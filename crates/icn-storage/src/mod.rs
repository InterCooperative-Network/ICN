// src/state/mod.rs

use crate::storage::{StorageManager, StorageError, StorageResult};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Represents a state migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tracks the current state and handles migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub current_version: u32,
    pub last_block_height: u64,
    pub state_root: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub applied_migrations: Vec<Migration>,
}

/// Manages blockchain state and batched operations
pub struct StateManager {
    storage: Arc<StorageManager>,
    metadata: Arc<RwLock<StateMetadata>>,
    batch_buffer: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl StateManager {
    /// Create a new state manager
    pub async fn new(storage: Arc<StorageManager>) -> StorageResult<Self> {
        // Try to load existing metadata or create new
        let metadata = match storage.retrieve::<StateMetadata>("state_metadata").await {
            Ok(meta) => meta,
            Err(StorageError::NotFound(_)) => StateMetadata {
                current_version: 0,
                last_block_height: 0,
                state_root: "0".repeat(64),
                last_updated: chrono::Utc::now(),
                applied_migrations: Vec::new(),
            },
            Err(e) => return Err(e),
        };

        Ok(Self {
            storage,
            metadata: Arc::new(RwLock::new(metadata)),
            batch_buffer: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start a new batch operation
    pub async fn begin_batch(&self) -> StorageResult<()> {
        let mut buffer = self.batch_buffer.write().await;
        buffer.clear();
        Ok(())
    }

    /// Add an operation to the current batch
    pub async fn batch_store<T: Serialize>(&self, key: &str, value: &T) -> StorageResult<()> {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
        let mut buffer = self.batch_buffer.write().await;
        buffer.insert(key.to_string(), serialized);
        Ok(())
    }

    /// Commit the current batch atomically
    pub async fn commit_batch(&self) -> StorageResult<String> {
        let buffer = self.batch_buffer.read().await;
        
        // Calculate new state root
        let mut hasher = Sha256::new();
        for (key, value) in buffer.iter() {
            hasher.update(key.as_bytes());
            hasher.update(value);
        }
        let new_state_root = format!("{:x}", hasher.finalize());

        // Store all items
        for (key, value) in buffer.iter() {
            self.storage.store(key, value).await?;
        }

        // Update metadata
        let mut metadata = self.metadata.write().await;
        metadata.state_root = new_state_root.clone();
        metadata.last_updated = chrono::Utc::now();
        
        // Store updated metadata
        self.storage.store("state_metadata", &*metadata).await?;

        Ok(new_state_root)
    }

    /// Roll back the current batch
    pub async fn rollback_batch(&self) -> StorageResult<()> {
        let mut buffer = self.batch_buffer.write().await;
        buffer.clear();
        Ok(())
    }

    /// Apply a new migration
    pub async fn apply_migration(&self, migration: Migration) -> StorageResult<()> {
        let mut metadata = self.metadata.write().await;
        
        // Verify migration version
        if migration.version <= metadata.current_version {
            return Err(StorageError::InvalidData(
                format!("Migration version {} has already been applied", migration.version)
            ));
        }

        // Store migration in metadata
        metadata.applied_migrations.push(migration.clone());
        metadata.current_version = migration.version;
        metadata.last_updated = chrono::Utc::now();

        // Persist updated metadata
        self.storage.store("state_metadata", &*metadata).await
    }

    /// Get current state metadata
    pub async fn get_metadata(&self) -> StateMetadata {
        self.metadata.read().await.clone()
    }

    /// Verify state integrity
    pub async fn verify_state(&self) -> StorageResult<bool> {
        let metadata = self.metadata.read().await;
        let mut hasher = Sha256::new();

        // Verify each stored state item
        // Note: This is a simplified version - in practice we'd use a Merkle tree
        let keys = self.list_state_keys().await?;
        for key in keys {
            if let Ok(value) = self.storage.get(&key).await {
                hasher.update(key.as_bytes());
                hasher.update(&value);
            }
        }

        let calculated_root = format!("{:x}", hasher.finalize());
        Ok(calculated_root == metadata.state_root)
    }

    /// List all state keys (helper method)
    async fn list_state_keys(&self) -> StorageResult<Vec<String>> {
        // This would be implemented based on your storage backend capabilities
        // For now, we'll return an empty vec as it depends on the specific implementation
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::postgres::PostgresStorage;

    async fn setup_test_state() -> StateManager {
        let connection_str = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test".to_string());
            
        let backend = PostgresStorage::new(&connection_str)
            .await
            .expect("Failed to create PostgreSQL connection");
            
        let storage = Arc::new(StorageManager::new(Box::new(backend)));
        StateManager::new(storage)
            .await
            .expect("Failed to create state manager")
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let state = setup_test_state().await;

        // Test batch operations
        state.begin_batch().await.unwrap();
        
        #[derive(Debug, Serialize, Deserialize)]
        struct TestState {
            value: i32,
        }

        state.batch_store("test1", &TestState { value: 1 }).await.unwrap();
        state.batch_store("test2", &TestState { value: 2 }).await.unwrap();

        let root = state.commit_batch().await.unwrap();
        assert!(!root.is_empty());

        // Verify metadata was updated
        let metadata = state.get_metadata().await;
        assert_eq!(metadata.state_root, root);
    }

    #[tokio::test]
    async fn test_migrations() {
        let state = setup_test_state().await;

        let migration = Migration {
            version: 1,
            description: "Test migration".to_string(),
            timestamp: chrono::Utc::now(),
        };

        state.apply_migration(migration.clone()).await.unwrap();

        let metadata = state.get_metadata().await;
        assert_eq!(metadata.current_version, 1);
        assert_eq!(metadata.applied_migrations.len(), 1);
        assert_eq!(metadata.applied_migrations[0].version, 1);
    }
}