// src/state/mod.rs
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use crate::error::{StorageError, StorageResult};
use crate::storage::StorageManager;

mod merkle_tree;
pub mod migrations;
mod persistence;
mod validation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub version: u64,
    pub root_hash: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct StateManager {
    storage: Arc<StorageManager>
}

impl StateManager {
    pub async fn new() -> Self {
        let storage = StorageManager::new(None).await
            .expect("Failed to initialize storage");
        
        Self {
            storage: Arc::new(storage)
        }
    }

    pub async fn get_metadata(&self) -> StorageResult<StateMetadata> {
        match self.storage.retrieve::<StateMetadata>("state_metadata").await {
            Ok(metadata) => Ok(metadata),
            Err(StorageError::KeyNotFound(_)) => {
                let metadata = StateMetadata {
                    version: 0,
                    root_hash: String::new(),
                    last_updated: chrono::Utc::now(),
                };
                
                self.storage.store("state_metadata", &metadata).await?;
                Ok(metadata)
            },
            Err(e) => Err(e),
        }
    }

    pub async fn store<T: Serialize>(&self, key: &str, value: &T) -> StorageResult<()> {
        self.storage.store(key, value).await
    }

    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> StorageResult<T> {
        self.storage.retrieve(key).await
    }

    pub async fn update_root_hash(&self) -> StorageResult<String> {
        let mut hasher = Sha256::new();
        
        // Get all keys and sort them for consistent hashing
        let mut keys = self.storage.list_keys().await?;
        keys.sort();
        
        // Hash all key-value pairs
        for key in keys {
            if let Ok(value) = self.storage.retrieve::<String>(&key).await {
                hasher.update(key.as_bytes());
                hasher.update(value.as_bytes());
            }
        }
        
        let hash = format!("{:x}", hasher.finalize());
        
        // Update metadata with new hash
        let mut metadata = self.get_metadata().await?;
        metadata.root_hash = hash.clone();
        metadata.last_updated = chrono::Utc::now();
        
        self.storage.store("state_metadata", &metadata).await?;
        
        Ok(hash)
    }
}