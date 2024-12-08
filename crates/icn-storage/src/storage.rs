use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use sqlx::{postgres::{PgPool, PgPoolOptions}, Row};
use tracing::{debug, error, info, warn};

use crate::error::{StorageError, StorageResult};
use crate::metrics::{MetricsManager, MetricType, OperationTimer};
use icn_types::{Block, Transaction, NetworkState};
use crate::backup::{BackupManager, BackupConfig, BackupInfo};

/// LRU cache size for frequently accessed data
const CACHE_SIZE: usize = 1000;
/// Maximum number of retries for database operations
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (milliseconds)
const BASE_DELAY_MS: u64 = 100;

/// Configuration for the storage system
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to enable query logging
    pub enable_logging: bool,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Metrics configuration
    pub metrics_config: MetricsConfig,
    /// Backup configuration
    pub backup_config: BackupConfig,
}

/// Cache configuration options
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub size: usize,
    pub ttl_seconds: u64,
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub flush_interval: Duration,
    pub retention_days: u32,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            flush_interval: Duration::from_secs(60),
            retention_days: 30,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: CACHE_SIZE,
            ttl_seconds: 3600,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/icn".to_string(),
            max_connections: 20,
            timeout_seconds: 30,
            enable_logging: false,
            cache_config: CacheConfig::default(),
            metrics_config: MetricsConfig::default(),
            backup_config: BackupConfig::default(),
        }
    }
}

/// Manages persistent storage, caching, metrics, and backups for the ICN network
pub struct StorageManager {
    pool: PgPool,
    cache: Arc<RwLock<lru::LruCache<String, Vec<u8>>>>,
    config: StorageConfig,
    metrics: MetricsManager,
    backup: BackupManager,
}

impl StorageManager {
    /// Create a new storage manager instance
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let pool = Self::create_connection_pool(&config).await?;
        let cache = Arc::new(RwLock::new(lru::LruCache::new(config.cache_config.size)));
        let metrics = MetricsManager::new(pool.clone());
        let backup = BackupManager::new(pool.clone(), config.backup_config.clone(), metrics.clone());

        if config.metrics_config.enabled {
            metrics.start_collection().await?;
        }
        
        let manager = Self {
            pool,
            cache,
            config,
            metrics,
            backup,
        };

        // Verify database connection and schema
        manager.verify_database().await?;
        
        Ok(manager)
    }

    /// Create and configure the database connection pool
    async fn create_connection_pool(config: &StorageConfig) -> StorageResult<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.timeout_seconds))
            .connect(&config.database_url)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        Ok(pool)
    }

    /// Verify database connection and schema
    async fn verify_database(&self) -> StorageResult<()> {
        let _timer = OperationTimer::new(
            MetricType::QueryExecutionTime, 
            self.metrics.clone(),
            Some(HashMap::from([
                ("operation".to_string(), "verify_database".to_string())
            ]))
        );

        // Test query to verify connection
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        // Verify required tables exist
        let tables = ["blocks", "transactions", "state", "metrics"];
        for table in tables {
            self.verify_table_exists(table).await?;
        }

        Ok(())
    }

    /// Store a new block with retry logic and metrics
    pub async fn store_block(&self, block: &Block) -> StorageResult<()> {
        let _timer = OperationTimer::new(
            MetricType::BlockWriteTime,
            self.metrics.clone(),
            Some(HashMap::from([
                ("height".to_string(), block.height.to_string())
            ]))
        );

        let mut retries = 0;
        let mut last_error = None;

        while retries < MAX_RETRIES {
            match self.store_block_internal(block).await {
                Ok(()) => {
                    self.invalidate_block_cache(block).await;
                    self.metrics.record_metric(
                        MetricType::BlockWriteTime,
                        0.0, // Success metric
                        Some(HashMap::from([
                            ("status".to_string(), "success".to_string()),
                            ("retries".to_string(), retries.to_string())
                        ]))
                    ).await?;
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    retries += 1;
                    let delay = BASE_DELAY_MS * 2u64.pow(retries);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }

        // Record failure metric
        self.metrics.record_metric(
            MetricType::BlockWriteTime,
            -1.0, // Failure metric
            Some(HashMap::from([
                ("status".to_string(), "failure".to_string()),
                ("retries".to_string(), retries.to_string())
            ]))
        ).await?;

        Err(last_error.unwrap_or_else(|| StorageError::MaxRetriesExceeded))
    }

    /// Internal block storage implementation with transaction support
    async fn store_block_internal(&self, block: &Block) -> StorageResult<()> {
        let mut tx = self.pool
            .begin()
            .await
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        // Store block
        sqlx::query(
            "INSERT INTO blocks (height, hash, previous_hash, timestamp, data, merkle_root, validator_signature)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(block.height)
        .bind(&block.hash)
        .bind(&block.previous_hash)
        .bind(block.timestamp)
        .bind(serde_json::to_value(block).map_err(|e| StorageError::SerializationError(e.to_string()))?)
        .bind(&block.merkle_root)
        .bind(&block.validator_signature)
        .execute(&mut *tx)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;

        // Store transactions
        for transaction in &block.transactions {
            sqlx::query(
                "INSERT INTO transactions (
                    hash, block_height, sender, recipient, transaction_type, 
                    amount, data, timestamp, signature
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            )
            .bind(&transaction.hash)
            .bind(block.height)
            .bind(&transaction.sender)
            .bind(&transaction.recipient)
            .bind(&transaction.transaction_type)
            .bind(transaction.amount)
            .bind(serde_json::to_value(transaction).map_err(|e| StorageError::SerializationError(e.to_string()))?)
            .bind(transaction.timestamp)
            .bind(&transaction.signature)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::QueryError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        Ok(())
    }

    /// Get block with metrics tracking
    pub async fn get_block(&self, hash: &str) -> StorageResult<Block> {
        let _timer = OperationTimer::new(
            MetricType::BlockReadTime,
            self.metrics.clone(),
            Some(HashMap::from([
                ("hash".to_string(), hash.to_string())
            ]))
        );

        // Check cache first
        if self.config.cache_config.enabled {
            if let Some(cached_data) = self.cache.read().await.get(hash) {
                if let Ok(block) = serde_json::from_slice(cached_data) {
                    self.metrics.record_metric(
                        MetricType::CacheHitRate,
                        1.0,
                        Some(HashMap::from([("operation".to_string(), "get_block".to_string())]))
                    ).await?;
                    return Ok(block);
                }
            }
        }

        // Cache miss, query database
        self.metrics.record_metric(
            MetricType::CacheMissRate,
            1.0,
            Some(HashMap::from([("operation".to_string(), "get_block".to_string())]))
        ).await?;

        let row = sqlx::query(
            "SELECT b.*, array_agg(t.*) as transactions
             FROM blocks b
             LEFT JOIN transactions t ON b.height = t.block_height
             WHERE b.hash = $1
             GROUP BY b.height, b.hash"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?
        .ok_or_else(|| StorageError::NotFound(format!("Block not found: {}", hash)))?;

        let block: Block = self.deserialize_block_row(row)?;

        // Update cache
        if self.config.cache_config.enabled {
            if let Ok(cached_data) = serde_json::to_vec(&block) {
                self.cache.write().await.put(hash.to_string(), cached_data);
            }
        }

        Ok(block)
    }

    /// Helper method to deserialize a block row with its transactions
    fn deserialize_block_row(&self, row: sqlx::postgres::PgRow) -> StorageResult<Block> {
        // Implementation details for deserializing the row into a Block
        // This would include handling the aggregated transactions
        todo!("Implement block row deserialization")
    }

    /// Get the latest block height with metrics
    pub async fn get_latest_block_height(&self) -> StorageResult<i64> {
        let _timer = OperationTimer::new(
            MetricType::QueryExecutionTime,
            self.metrics.clone(),
            Some(HashMap::from([
                ("operation".to_string(), "get_latest_block_height".to_string())
            ]))
        );

        let row = sqlx::query("SELECT MAX(height) as height FROM blocks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::QueryError(e.to_string()))?;

        Ok(row.get::<Option<i64>, _>("height").unwrap_or(0))
    }

    /// Run database migrations with metrics
    pub async fn run_migrations(&self) -> StorageResult<()> {
        let _timer = OperationTimer::new(
            MetricType::QueryExecutionTime,
            self.metrics.clone(),
            Some(HashMap::from([
                ("operation".to_string(), "run_migrations".to_string())
            ]))
        );

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| StorageError::MigrationError(e.to_string()))?;

        Ok(())
    }

    /// Perform maintenance operations
    pub async fn perform_maintenance(&self) -> StorageResult<()> {
        // Clean up old metrics
        let retention_timestamp = chrono::Utc::now()
            .timestamp() - (self.config.metrics_config.retention_days as i64 * 86400);
        
        let deleted_count = self.metrics.cleanup_old_metrics(retention_timestamp).await?;
        info!("Cleaned up {} old metric records", deleted_count);

        // Analyze tables
        let tables = ["blocks", "transactions", "state", "metrics"];
        for table in tables {
            sqlx::query(&format!("ANALYZE {}", table))
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::QueryError(e.to_string()))?;
        }

        Ok(())
    }

    /// Create a new backup
    pub async fn create_backup(&self) -> StorageResult<BackupInfo> {
        let _timer = OperationTimer::new(
            MetricType::BackupOperation,
            self.metrics.clone(),
            Some(HashMap::from([
                ("operation".to_string(), "create_backup".to_string())
            ]))
        );

        self.backup.create_backup().await
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_info: &BackupInfo) -> StorageResult<()> {
        let _timer = OperationTimer::new(
            MetricType::BackupOperation,
            self.metrics.clone(),
            Some(HashMap::from([
                ("operation".to_string(), "restore_backup".to_string()),
                ("backup_id".to_string(), backup_info.id.clone())
            ]))
        );

        // Clear cache before restore
        self.cache.write().await.clear();
        
        self.backup.restore_backup(backup_info).await?;

        // Verify database state after restore
        self.verify_database().await?;

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> StorageResult<Vec<BackupInfo>> {
        self.backup.list_backups().await
    }

    /// Get information about the latest backup
    pub async fn get_latest_backup(&self) -> StorageResult<Option<BackupInfo>> {
        let backups = self.list_backups().await?;
        Ok(backups.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_storage_initialization() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();
        assert!(storage.verify_database().await.is_ok());
    }

    #[tokio::test]
    async fn test_block_storage_and_retrieval() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            merkle_root: "merkle_root".to_string(),
            validator_signature: "signature".to_string(),
            transactions: vec![],
        };

        assert!(storage.store_block(&block).await.is_ok());
        let retrieved = storage.get_block(&block.hash).await.unwrap();
        assert_eq!(retrieved.hash, block.hash);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        // Perform some operations to generate metrics
        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            merkle_root: "merkle_root".to_string(),
            validator_signature: "signature".to_string(),
            transactions: vec![],
        };

        storage.store_block(&block).await.unwrap();
        storage.get_block(&block.hash).await.unwrap();

        // Allow metrics to be flushed
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check metrics were recorded
        let now = chrono::Utc::now().timestamp();
        let metrics = storage.metrics.get_metrics(
            MetricType::BlockWriteTime,
            now - 3600,
            now
        ).await.unwrap();

        assert!(!metrics.is_empty());
        
        // Test cache metrics
        let cache_hits = storage.metrics.get_metrics(
            MetricType::CacheHitRate,
            now - 3600,
            now
        ).await.unwrap();

        let cache_misses = storage.metrics.get_metrics(
            MetricType::CacheMissRate,
            now - 3600,
            now
        ).await.unwrap();

        // First access should be a miss, second a hit
        assert_eq!(cache_hits.len(), 1);
        assert_eq!(cache_misses.len(), 1);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        // Test not found error
        let result = storage.get_block("nonexistent_hash").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));

        // Test invalid hash error
        let result = storage.get_block("").await;
        assert!(matches!(result, Err(StorageError::QueryError(_))));
    }

    #[tokio::test]
    async fn test_maintenance() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        // Generate some test metrics
        for i in 0..10 {
            storage.metrics.record_metric(
                MetricType::BlockWriteTime,
                i as f64,
                None
            ).await.unwrap();
        }

        // Allow metrics to be flushed
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Run maintenance
        assert!(storage.perform_maintenance().await.is_ok());
    }

    #[tokio::test]
    async fn test_backup_integration() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        // Create some test data
        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            merkle_root: "merkle_root".to_string(),
            validator_signature: "signature".to_string(),
            transactions: vec![],
        };
        storage.store_block(&block).await.unwrap();

        // Create backup
        let backup_info = storage.create_backup().await.unwrap();
        assert!(backup_info.path.exists());

        // Modify data
        let block2 = Block {
            height: 2,
            hash: "test_hash2".to_string(),
            previous_hash: block.hash.clone(),
            timestamp: 12346,
            merkle_root: "merkle_root2".to_string(),
            validator_signature: "signature2".to_string(),
            transactions: vec![],
        };
        storage.store_block(&block2).await.unwrap();

        // Restore backup
        storage.restore_backup(&backup_info).await.unwrap();

        // Verify restored state
        let restored_block = storage.get_block(&block.hash).await.unwrap();
        assert_eq!(restored_block.hash, block.hash);

        // Verify block2 is gone after restore
        let result = storage.get_block(&block2.hash).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
}