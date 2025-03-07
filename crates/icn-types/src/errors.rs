use thiserror::Error;
use std::fmt;
use log::{error, warn};

#[derive(Debug, Error)]
pub enum IcnError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Federation error: {0}")]
    FederationError(String),
    
    #[error("Governance error: {0}")]
    GovernanceError(String),
    
    #[error("Identity error: {0}")]
    IdentityError(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Rate limiting: {0}")]
    RateLimitError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Resource not found: {0}")]
    NotFoundError(String),
    
    #[error("Request timeout: {0}")]
    TimeoutError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInputError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Block error: {0}")]
    BlockError(#[from] super::BlockError),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for ICN operations
pub type IcnResult<T> = Result<T, IcnError>;

/// Add a helper function for unified error logging
pub fn log_error(error: &IcnError, module: &str) {
    match error {
        IcnError::ValidationError(_) | 
        IcnError::RateLimitError(_) |
        IcnError::AuthorizationError(_) => {
            warn!("[{}] {}", module, error);
        },
        _ => {
            error!("[{}] {}", module, error);
        }
    }
}
