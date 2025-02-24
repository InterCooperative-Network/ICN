use async_trait::async_trait;
use icn_types::{StorageError, StorageResult};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Database {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.data.read().await
            .get(key)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    pub async fn put(&self, key: String, value: Vec<u8>) -> StorageResult<()> {
        self.data.write().await.insert(key, value);
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        self.data.write().await.remove(key);
        Ok(())
    }
}

#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn get(&self, key: &str) -> StorageResult<Vec<u8>>;
    async fn put(&self, key: String, value: Vec<u8>) -> StorageResult<()>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

#[async_trait]
impl DatabaseBackend for Database {
    async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.get(key).await
    }

    async fn put(&self, key: String, value: Vec<u8>) -> StorageResult<()> {
        self.put(key, value).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.delete(key).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        Ok(self.data.read().await.contains_key(key))
    }
}
