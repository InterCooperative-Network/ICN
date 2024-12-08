use thiserror::Error;

/// Storage system error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database query error: {0}")]
    QueryError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Data not found: {0}")]
    NotFound(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    #[error("Invalid configuration: {0}")]
    ConfigurationError(String),

    #[error("Backup error: {0}")]
    BackupError(String),

    #[error("Restore error: {0}")]
    RestoreError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

impl From<sqlx::Error> for StorageError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::Database(e) => StorageError::QueryError(e.to_string()),
            sqlx::Error::RowNotFound => StorageError::NotFound("Row not found".to_string()),
            sqlx::Error::PoolTimedOut => StorageError::ConnectionError("Connection pool timeout".to_string()),
            _ => StorageError::QueryError(error.to_string()),
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        StorageError::SerializationError(error.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        StorageError::BackupError(error.to_string())
    }
}

impl From<flate2::Error> for StorageError {
    fn from(error: flate2::Error) -> Self {
        StorageError::CompressionError(error.to_string())
    }
}