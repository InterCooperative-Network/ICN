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
    
    #[error("Pool error: {0}")]
    PoolError(String),
}

pub type StorageResult<T> = Result<T, StorageError>;