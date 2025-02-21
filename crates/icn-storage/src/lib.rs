use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use icn_types::Block;
use std::time::Duration;

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
}

impl StorageManager {
    /// Create a new storage manager with the given backend
    pub fn new(backend: Box<dyn StorageBackend>, cache_size: usize, cache_ttl: Duration) -> Self {
        Self {
            backend: Arc::new(Mutex::new(backend)),
            cache: Arc::new(StorageCache::new(cache_size, cache_ttl)),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    // Mock storage backend for testing
    struct MockStorage {
        data: HashMap<String, Vec<u8>>,
    }
    
    #[async_trait::async_trait]
    impl StorageBackend for MockStorage {
        async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
            self.data.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        
        async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
            self.data.get(key)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(key.to_string()))
        }
        
        async fn delete(&self, key: &str) -> StorageResult<()> {
            self.data.remove(key);
            Ok(())
        }
        
        async fn exists(&self, key: &str) -> StorageResult<bool> {
            Ok(self.data.contains_key(key))
        }
    }
    
    #[tokio::test]
    async fn test_basic_storage_operations() {
        // Test implementation here
        // Will add comprehensive tests as we develop
    }
}
