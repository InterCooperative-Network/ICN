// crates/icn-storage/src/storage.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::{Pool, Postgres};
use icn_types::{StorageError, StorageResult, Block, Transaction, Relationship, ReputationScore};
use serde_json::Value;
use chrono::{DateTime, Utc};

/// Configuration for the storage manager
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_pool_size: u32,
    pub timeout_seconds: u64,
    pub username: String,
    pub password: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/icn".to_string(),
            max_pool_size: 5,
            timeout_seconds: 30,
            username: "matt".to_string(),
            password: String::new(), // Should be set via environment or config file
        }
    }
}

impl StorageConfig {
    /// Creates a new StorageConfig with full connection string
    pub fn with_credentials(username: &str, password: &str, database: &str) -> Self {
        Self {
            database_url: format!("postgresql://{}:{}@localhost/{}", username, password, database),
            max_pool_size: 5,
            timeout_seconds: 30,
            username: username.to_string(),
            password: password.to_string(),
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
            .map_err(|e| StorageError::Pool(e.to_string()))?;

        // Initialize LRU cache for frequently accessed data
        let cache = Arc::new(RwLock::new(lru::LruCache::new(1000)));

        Ok(Self { pool, cache })
    }

    /// Store a block in the database
    pub async fn store_block(&self, block: &Block) -> StorageResult<()> {
        let query = sqlx::query!(
            r#"
            INSERT INTO blocks (height, hash, previous_hash, timestamp, data)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            block.index as i64,
            block.hash,
            block.previous_hash,
            block.timestamp as i64,
            serde_json::to_value(block).map_err(|e| StorageError::Serialization(e.to_string()))?,
        );

        query
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Update cache
        let cache_key = format!("block:{}", block.hash);
        let block_data = serde_json::to_vec(block)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
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
                .map_err(|e| StorageError::Serialization(e.to_string()));
        }

        // If not in cache, query database
        let record = sqlx::query!(
            "SELECT data FROM blocks WHERE hash = $1",
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        let block = match record {
            Some(record) => {
                let block: Block = serde_json::from_value(record.data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                // Update cache
                let block_data = serde_json::to_vec(&block)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                let mut cache = self.cache.write().await;
                cache.put(cache_key, block_data);

                block
            }
            None => return Err(StorageError::KeyNotFound(format!("Block not found: {}", hash))),
        };

        Ok(block)
    }

    /// Store a batch of transactions
    pub async fn store_transactions(&self, transactions: &[Transaction]) -> StorageResult<()> {
        for transaction in transactions {
            let query = sqlx::query!(
                r#"
                INSERT INTO transactions (hash, sender, transaction_type, data, timestamp)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                transaction.hash,
                transaction.sender,
                transaction.transaction_type.to_string(),
                serde_json::to_value(transaction).map_err(|e| StorageError::Serialization(e.to_string()))?,
                transaction.timestamp as i64,
            );

            query
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Get transactions by sender
    pub async fn get_transactions_by_sender(&self, sender: &str) -> StorageResult<Vec<Transaction>> {
        let records = sqlx::query!(
            "SELECT data FROM transactions WHERE sender = $1 ORDER BY timestamp DESC",
            sender
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        let transactions = records
            .into_iter()
            .map(|record| {
                serde_json::from_value(record.data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))
            })
            .collect::<Result<Vec<Transaction>, _>>()?;

        Ok(transactions)
    }

    /// Get the latest block height
    pub async fn get_latest_block_height(&self) -> StorageResult<i64> {
        let record = sqlx::query!("SELECT MAX(height) as height FROM blocks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(record.height.unwrap_or(0))
    }

    /// Clean up old data (can be called periodically)
    pub async fn cleanup_old_data(&self, before_timestamp: i64) -> StorageResult<()> {
        // Remove old transactions while keeping genesis block and recent history
        sqlx::query!(
            "DELETE FROM transactions WHERE timestamp < $1",
            before_timestamp
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

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
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get all relationships for a given DID (either as source or target)
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
        .map_err(|e| StorageError::Database(e.to_string()))?;

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

    /// Update reputation score for a DID in a given context
    pub async fn update_reputation(
        &self,
        did: &str,
        context: &str,
        score_delta: i32,
    ) -> StorageResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO reputation (did, context, score, history)
            VALUES ($1, $2, $3, '[]'::jsonb)
            ON CONFLICT (did) 
            DO UPDATE SET 
                score = reputation.score + $3,
                history = jsonb_build_array(
                    jsonb_build_object(
                        'timestamp', extract(epoch from current_timestamp)::bigint,
                        'delta', $3,
                        'new_score', reputation.score + $3
                    )
                ) || reputation.history,
                last_updated = CURRENT_TIMESTAMP
            WHERE reputation.context = $2
            "#,
            did,
            context,
            score_delta,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get reputation score and history for a DID in a specific context
    pub async fn get_reputation(
        &self,
        did: &str,
        context: &str,
    ) -> StorageResult<Option<ReputationScore>> {
        let record = sqlx::query!(
            r#"
            SELECT score, history, last_updated
            FROM reputation
            WHERE did = $1 AND context = $2
            "#,
            did,
            context
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(record.map(|r| ReputationScore {
            score: r.score,
            history: r.history,
            last_updated: r.last_updated,
        }))
    }

    /// Get top reputation scores in a specific context
    pub async fn get_top_reputation_scores(
        &self,
        context: &str,
        limit: i64
    ) -> StorageResult<Vec<(String, i32)>> {
        let records = sqlx::query!(
            r#"
            SELECT did, score
            FROM reputation
            WHERE context = $1
            ORDER BY score DESC
            LIMIT $2
            "#,
            context,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(records.into_iter().map(|r| (r.did, r.score)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_types::{Block, Transaction, TransactionType};

    #[tokio::test]
    async fn test_store_and_retrieve_block() {
        // Initialize test database connection
        let config = StorageConfig {
            database_url: "postgresql://localhost/icn_test".to_string(),
            ..Default::default()
        };
        let storage = StorageManager::new(config).await.unwrap();

        // Create test block
        let block = Block {
            index: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: 12345,
            // ... other fields
        };

        // Store block
        storage.store_block(&block).await.unwrap();

        // Retrieve block
        let retrieved = storage.get_block("test_hash").await.unwrap();
        assert_eq!(retrieved.hash, block.hash);
        assert_eq!(retrieved.index, block.index);
    }
}