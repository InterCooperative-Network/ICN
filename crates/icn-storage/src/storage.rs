use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;
use tracing::{debug, error, info};

use crate::error::{StorageError, StorageResult};
use icn_types::{Block, Transaction, NetworkState};

/// Configuration for the storage system
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub cache_size: usize,
    pub timeout_seconds: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/icn".to_string(),
            max_connections: 5,
            cache_size: 1000,
            timeout_seconds: 30,
        }
    }
}

/// Main storage manager handling persistence and caching
pub struct StorageManager {
    pool: PgPool,
    cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager instance
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .connect(&config.database_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let cache = Arc::new(RwLock::new(LruCache::new(config.cache_size)));

        Ok(Self {
            pool,
            cache,
            config,
        })
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> StorageResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Store a new block in the database
    pub async fn store_block(&self, block: &Block) -> StorageResult<()> {
        // Serialize block data
        let data = serde_json::to_value(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO blocks (height, hash, previous_hash, timestamp, data)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            block.height as i64,
            block.hash,
            block.previous_hash,
            block.timestamp as i64,
            data
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Update cache
        let cache_key = format!("block:{}", block.hash);
        let block_data = serde_json::to_vec(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        let mut cache = self.cache.write().await;
        cache.put(cache_key, block_data);

        Ok(())
    }

    /// Retrieve a block by its hash
    pub async fn get_block(&self, hash: &str) -> StorageResult<Block> {
        // Check cache first
        let cache_key = format!("block:{}", hash);
        if let Some(block_data) = self.cache.read().await.get(&cache_key) {
            return serde_json::from_slice(block_data)
                .map_err(|e| StorageError::SerializationError(e.to_string()));
        }

        // Query database
        let record = sqlx::query!(
            r#"
            SELECT data FROM blocks WHERE hash = $1
            "#,
            hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::KeyNotFound(e.to_string()))?;

        let block: Block = serde_json::from_value(record.data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Update cache
        let block_data = serde_json::to_vec(&block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        let mut cache = self.cache.write().await;
        cache.put(cache_key, block_data);

        Ok(block)
    }

    /// Get the latest block height
    pub async fn get_latest_block_height(&self) -> StorageResult<i64> {
        let record = sqlx::query!(
            r#"
            SELECT MAX(height) as height FROM blocks
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(record.height.unwrap_or(0))
    }

    /// Store a batch of transactions
    pub async fn store_transactions(&self, transactions: &[Transaction]) -> StorageResult<()> {
        for tx in transactions {
            let data = serde_json::to_value(tx)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            sqlx::query!(
                r#"
                INSERT INTO transactions (hash, block_height, sender, transaction_type, data, timestamp)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                tx.hash,
                tx.block_height as i64,
                tx.sender,
                tx.transaction_type.to_string(),
                data,
                tx.timestamp as i64
            )
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Get transactions by sender
    pub async fn get_transactions_by_sender(&self, sender: &str) -> StorageResult<Vec<Transaction>> {
        let records = sqlx::query!(
            r#"
            SELECT data FROM transactions WHERE sender = $1
            "#,
            sender
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let transactions = records
            .into_iter()
            .map(|r| {
                serde_json::from_value(r.data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            })
            .collect::<Result<Vec<Transaction>, StorageError>>()?;

        Ok(transactions)
    }

    /// Get the current network state
    pub async fn get_network_state(&self) -> StorageResult<NetworkState> {
        let state = sqlx::query!(
            r#"
            SELECT data FROM network_state ORDER BY timestamp DESC LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match state {
            Some(record) => {
                serde_json::from_value(record.data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            None => Ok(NetworkState::default())
        }
    }

    /// Update the network state
    pub async fn update_network_state(&self, state: &NetworkState) -> StorageResult<()> {
        let data = serde_json::to_value(state)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO network_state (timestamp, data)
            VALUES ($1, $2)
            "#,
            state.timestamp as i64,
            data
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Clean up old data beyond retention period
    pub async fn cleanup_old_data(&self, before_timestamp: i64) -> StorageResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM transactions WHERE timestamp < $1
            "#,
            before_timestamp
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
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
        assert!(storage.get_latest_block_height().await.unwrap() >= 0);
    }

    #[tokio::test]
    async fn test_block_storage() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            transactions: vec![],
        };

        storage.store_block(&block).await.unwrap();
        let retrieved = storage.get_block("test_hash").await.unwrap();
        assert_eq!(block.hash, retrieved.hash);
    }

    #[tokio::test]
    async fn test_transaction_storage() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config).await.unwrap();

        let tx = Transaction {
            hash: "tx_hash".to_string(),
            block_height: 1,
            sender: "sender".to_string(),
            transaction_type: "transfer".to_string(),
            timestamp: 12345,
        };

        storage.store_transactions(&[tx.clone()]).await.unwrap();
        let transactions = storage.get_transactions_by_sender("sender").await.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].hash, tx.hash);
    }
}