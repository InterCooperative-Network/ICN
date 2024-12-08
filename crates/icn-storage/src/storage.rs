// crates/icn-storage/src/storage.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::{Pool, Postgres};
use crate::error::{StorageError, StorageResult};  // Using our local error types
use icn_types::{Block, Transaction, Relationship};
use serde_json::Value;

/// Configuration for the storage manager
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_pool_size: u32,
    pub timeout_seconds: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_db".to_string()),
            max_pool_size: 5,
            timeout_seconds: 30,
        }
    }
}

/// Manages persistent storage and state management for the ICN system
pub struct StorageManager {
    pool: Pool<Postgres>,
    cache: Arc<RwLock<lru::LruCache<String, Vec<u8>>>>,
}

impl StorageManager {
    /// Create a new storage manager instance
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        // Initialize database connection pool
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_pool_size)
            .connect_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .connect(&config.database_url)
            .await
            .map_err(|e| StorageError::PoolError(e.to_string()))?;

        // Initialize LRU cache for frequently accessed data
        let cache = Arc::new(RwLock::new(lru::LruCache::new(1000)));

        Ok(Self { pool, cache })
    }

    /// Store a block in the database
    pub async fn store_block(&self, block: &Block) -> StorageResult<()> {
        let data = serde_json::to_value(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO blocks (height, hash, previous_hash, timestamp, proposer, data)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            block.height as i64,
            block.hash,
            block.previous_hash,
            block.timestamp as i64,
            block.proposer,
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

        // If not in cache, query database
        let record = sqlx::query!(
            r#"
            SELECT data FROM blocks WHERE hash = $1
            "#,
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let block = match record {
            Some(record) => {
                serde_json::from_value(record.data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?
            }
            None => return Err(StorageError::KeyNotFound(format!("Block not found: {}", hash))),
        };

        // Update cache
        let block_data = serde_json::to_vec(&block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let mut cache = self.cache.write().await;
        cache.put(cache_key, block_data);

        Ok(block)
    }

    /// Store a batch of transactions
    pub async fn store_transactions(&self, transactions: &[Transaction]) -> StorageResult<()> {
        for transaction in transactions {
            let data = serde_json::to_value(transaction)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            sqlx::query!(
                r#"
                INSERT INTO transactions (hash, block_height, sender, transaction_type, data, timestamp)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                transaction.hash,
                transaction.block_height as i64,
                transaction.sender,
                transaction.transaction_type.to_string(),
                data,
                transaction.timestamp as i64
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
            SELECT data FROM transactions WHERE sender = $1 ORDER BY timestamp DESC
            "#,
            sender
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let transactions = records
            .into_iter()
            .map(|record| {
                serde_json::from_value(record.data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Get the latest block height
    pub async fn get_latest_block_height(&self) -> StorageResult<i64> {
        let record = sqlx::query!(
            r#"
            SELECT MAX(height) as "max_height?" FROM blocks
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(record.max_height.unwrap_or(0))
    }

    /// Clean up old data before specified timestamp
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

    /// Create or update a relationship between two DIDs
    pub async fn upsert_relationship(
        &self,
        source_did: &str,
        target_did: &str,
        relationship_type: &str,
        metadata: Option<Value>,
    ) -> StorageResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO relationships (source_did, target_did, relationship_type, metadata)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (source_did, target_did, relationship_type) 
            DO UPDATE SET 
                metadata = EXCLUDED.metadata,
                updated_at = CURRENT_TIMESTAMP
            "#,
            source_did,
            target_did,
            relationship_type,
            metadata,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get all relationships for a given DID
    pub async fn get_relationships_for_did(&self, did: &str) -> StorageResult<Vec<Relationship>> {
        let records = sqlx::query!(
            r#"
            SELECT source_did, target_did, relationship_type, metadata, created_at, updated_at
            FROM relationships
            WHERE source_did = $1 OR target_did = $1
            "#,
            did
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let relationships = records
            .into_iter()
            .map(|r| Relationship {
                source_did: r.source_did,
                target_did: r.target_did,
                relationship_type: r.relationship_type,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(relationships)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_types::{Block, Transaction};

    async fn create_test_storage() -> StorageResult<StorageManager> {
        let config = StorageConfig {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test_db".to_string()),
            max_pool_size: 2,
            timeout_seconds: 5,
        };
        StorageManager::new(config).await
    }

    #[tokio::test]
    async fn test_block_storage_and_retrieval() -> StorageResult<()> {
        let storage = create_test_storage().await?;
        
        // Create test block
        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            proposer: "test_proposer".to_string(),
            data: serde_json::json!({"test": "data"}),
        };

        // Store block
        storage.store_block(&block).await?;

        // Retrieve block
        let retrieved = storage.get_block(&block.hash).await?;
        assert_eq!(block.hash, retrieved.hash);
        assert_eq!(block.height, retrieved.height);

        Ok(())
    }

    #[tokio::test]
    async fn test_latest_block_height() -> StorageResult<()> {
        let storage = create_test_storage().await?;
        let height = storage.get_latest_block_height().await?;
        assert!(height >= 0);
        Ok(())
    }
}