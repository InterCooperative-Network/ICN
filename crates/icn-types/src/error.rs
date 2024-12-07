// crates/icn-types/src/error.rs

use thiserror::Error;
use std::fmt;

/// Core error types for the ICN system
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Consensus error: {0}")]
    Consensus(#[from] ConsensusError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Storage-specific errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("State sync error: {0}")]
    StateSync(String),
}

/// Consensus-specific errors
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Not enough validators: {0}")]
    InsufficientValidators(String),

    #[error("Round timeout: {0}")]
    RoundTimeout(String),

    #[error("Invalid vote: {0}")]
    InvalidVote(String),

    #[error("Byzantine behavior detected: {0}")]
    ByzantineBehavior(String),
}

/// Network-specific errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Message timeout: {0}")]
    MessageTimeout(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Peer error: {0}")]
    PeerError(String),
}

/// Identity-specific errors
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Invalid DID: {0}")]
    InvalidDID(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Common result types
pub type CoreResult<T> = Result<T, CoreError>;
pub type StorageResult<T> = Result<T, StorageError>;
pub type ConsensusResult<T> = Result<T, ConsensusError>;
pub type NetworkResult<T> = Result<T, NetworkError>;
pub type IdentityResult<T> = Result<T, IdentityError>;

// Implement conversion between error types
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

// Add helper functions for common error scenarios
impl CoreError {
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        CoreError::Validation(msg.into())
    }

    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        CoreError::Configuration(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        CoreError::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let storage_err = StorageError::KeyNotFound("test_key".to_string());
        let core_err: CoreError = storage_err.into();
        assert!(matches!(core_err, CoreError::Storage(_)));
    }

    #[test]
    fn test_error_helper_functions() {
        let err = CoreError::validation("invalid input");
        assert!(matches!(err, CoreError::Validation(_)));
        
        let err = CoreError::configuration("missing config");
        assert!(matches!(err, CoreError::Configuration(_)));
        
        let err = CoreError::internal("unexpected error");
        assert!(matches!(err, CoreError::Internal(_)));
    }
}