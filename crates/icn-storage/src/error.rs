// src/error.rs
use thiserror::Error;

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
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Pool error: {0}")]
    PoolError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Add conversions from common error types
impl From<sqlx::Error> for StorageError {
    fn from(err: sqlx::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError(err.to_string())
    }
}

// Type alias for Result with StorageError
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = StorageError::DatabaseError("connection failed".to_string());
        assert_eq!(err.to_string(), "Database error: connection failed");

        let err = StorageError::KeyNotFound("block_123".to_string());
        assert_eq!(err.to_string(), "Key not found: block_123");
    }

    #[test]
    fn test_error_conversions() {
        // Test SQLx error conversion
        let db_err = sqlx::Error::RowNotFound;
        let storage_err: StorageError = db_err.into();
        matches!(storage_err, StorageError::DatabaseError(_));

        // Test serde error conversion
        let serde_err = serde_json::Error::syntax(serde_json::error::ErrorCode::ExpectedSomeValue, 0, 0);
        let storage_err: StorageError = serde_err.into();
        matches!(storage_err, StorageError::SerializationError(_));
    }
}