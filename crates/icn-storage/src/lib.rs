use thiserror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use icn_types::{StorageError as IcnStorageError, StorageResult as IcnStorageResult, StorageBackend as IcnStorageBackend, StorageConfig};
use std::time::Duration;
use ipfs_api_backend_actix::{IpfsClient, TryFromUri, IpfsApi};
use futures::TryStreamExt;
use std::io::Cursor;
use std::collections::HashMap;
use std::sync::RwLock;

mod cache;
use cache::StorageCache;

/// Errors that can occur in storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("IPFS error: {0}")]
    IpfsError(String),
    
    #[error("Reference already exists")]
    ReferenceAlreadyExists,
    
    #[error("Reference not found")]
    ReferenceNotFound,
}

/// Represents the result of storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Core storage interface for the system
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a value with the given key
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()>;
    
    /// Retrieve a value by key
    async fn get(&self, key: &str) -> StorageResult<Vec<u8>>;
    
    /// Delete a value by key
    async fn delete(&self, key: &str) -> StorageResult<()>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

/// Manages persistent storage for the system
pub struct StorageManager {
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    cache: Arc<StorageCache>,
    ipfs_client: IpfsClient,
}

impl StorageManager {
    /// Create a new storage manager with the given backend and configuration
    pub fn new(backend: Box<dyn StorageBackend>, config: StorageConfig) -> Self {
        Self {
            backend: Arc::new(Mutex::new(backend)),
            cache: Arc::new(StorageCache::new(
                config.cache_size,
                Duration::from_secs(config.cache_ttl_seconds)
            )),
            ipfs_client: IpfsClient::from_str(&config.ipfs_url).expect("Invalid IPFS URL"),
        }
    }
    
    /// Store a serializable value
    pub async fn store<T: Serialize>(&self, key: &str, value: &T) -> StorageResult<()> {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
        // Update backend
        let backend = self.backend.lock().await;
        backend.set(key, &serialized).await?;
        
        // Update cache
        self.cache.set(key.to_string(), serialized);
        
        Ok(())
    }
    
    /// Retrieve and deserialize a value
    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> StorageResult<T> {
        // Try cache first
        if let Some(cached_data) = self.cache.get(key) {
            return serde_json::from_slice(&cached_data)
                .map_err(|e| StorageError::SerializationError(e.to_string()));
        }

        // Fall back to backend
        let backend = self.backend.lock().await;
        let data = backend.get(key).await?;
        
        // Update cache
        self.cache.set(key.to_string(), data.clone());
        
        serde_json::from_slice(&data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }
    
    /// Delete a stored value
    pub async fn remove(&self, key: &str) -> StorageResult<()> {
        let backend = self.backend.lock().await;
        backend.delete(key).await
    }
    
    /// Check if a key exists in storage
    pub async fn has_key(&self, key: &str) -> StorageResult<bool> {
        let backend = self.backend.lock().await;
        backend.exists(key).await
    }

    /// Store data using IPFS
    pub async fn store_ipfs(&self, data: &[u8]) -> StorageResult<String> {
        // Create a cursor around the data to implement Read trait
        let cursor = Cursor::new(data.to_vec());
        let result = self.ipfs_client.add(cursor).await
            .map_err(|e| StorageError::IpfsError(e.to_string()))?;
        Ok(result.hash)
    }

    /// Retrieve data from IPFS
    pub async fn retrieve_ipfs(&self, hash: &str) -> StorageResult<Vec<u8>> {
        let data = self.ipfs_client.cat(hash).map_ok(|chunk| chunk.to_vec()).try_concat().await
            .map_err(|e| StorageError::IpfsError(e.to_string()))?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;
    
    // Mock storage backend for testing
    struct MockStorage {
        data: RwLock<HashMap<String, Vec<u8>>>,
    }
    
    #[async_trait::async_trait]
    impl StorageBackend for MockStorage {
        async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
            let mut data = self.data.write().unwrap();
            data.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        
        async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
            let data = self.data.read().unwrap();
            data.get(key)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(key.to_string()))
        }
        
        async fn delete(&self, key: &str) -> StorageResult<()> {
            let mut data = self.data.write().unwrap();
            data.remove(key);
            Ok(())
        }
        
        async fn exists(&self, key: &str) -> StorageResult<bool> {
            let data = self.data.read().unwrap();
            Ok(data.contains_key(key))
        }
    }
    
    #[tokio::test]
    async fn test_basic_storage_operations() {
        let config = StorageConfig {
            backend_type: "mock".to_string(),
            cache_size: 1000,
            cache_ttl_seconds: 300,
            ipfs_url: "http://localhost:5001".to_string(),
            database_url: None,
        };
        
        let storage = StorageManager::new(
            Box::new(MockStorage { data: RwLock::new(HashMap::new()) }),
            config
        );

        // Test store and retrieve
        let key = "test_key";
        let value = "test_value";
        storage.store(key, &value).await.unwrap();
        
        let retrieved: String = storage.retrieve(key).await.unwrap();
        assert_eq!(retrieved, value);
        
        // Test exists
        assert!(storage.has_key(key).await.unwrap());
        
        // Test delete
        storage.remove(key).await.unwrap();
        assert!(!storage.has_key(key).await.unwrap());
    }
}
