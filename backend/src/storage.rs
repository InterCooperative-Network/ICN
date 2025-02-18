use async_trait::async_trait;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[async_trait]
pub trait StorageBackend {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()>;
    async fn get(&self, key: &str) -> StorageResult<Vec<u8>>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

pub struct StorageManager {
    backend: Box<dyn StorageBackend + Send + Sync>,
}

impl StorageManager {
    pub fn new(backend: Box<dyn StorageBackend + Send + Sync>) -> Self {
        Self { backend }
    }
}
