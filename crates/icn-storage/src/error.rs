use thiserror::Error;
use sqlx::error::Error as SqlxError;

/// Custom error types for the storage system
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Pool error: {0}")]
    PoolError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

impl From<SqlxError> for StorageError {
    fn from(err: SqlxError) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError(err.to_string())
    }
}