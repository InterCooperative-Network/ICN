// File: crates/icn-core/src/error.rs

use std::fmt;
use thiserror::Error;

/// Core result type
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for the ICN system
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Consensus error: {0}")]
    Consensus(#[from] icn_consensus::ConsensusError),

    #[error("Storage error: {0}")]
    Storage(#[from] icn_storage::StorageError),

    #[error("Network error: {0}")]
    Network(#[from] icn_p2p::NetworkError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] icn_runtime::RuntimeError),

    #[error("System error: {0}")]
    System(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Error metadata for enhanced error handling
#[derive(Debug, Clone)]
pub struct ErrorMetadata {
    /// Unique error code
    pub code: String,
    
    /// Error severity level
    pub severity: ErrorSeverity,
    
    /// Whether the error is recoverable
    pub recoverable: bool,
    
    /// Additional context
    pub context: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Minor issues that don't affect functionality
    Low,
    
    /// Issues that degrade but don't prevent functionality
    Medium,
    
    /// Serious issues that prevent some functionality
    High,
    
    /// Critical issues that prevent core functionality
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl Error {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }

    /// Create a new system error
    pub fn system<S: Into<String>>(msg: S) -> Self {
        Error::System(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Error::Validation(msg.into())
    }

    /// Create a new invalid state error
    pub fn invalid_state<S: Into<String>>(msg: S) -> Self {
        Error::InvalidState(msg.into())
    }

    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Error::NotFound(msg.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Error::Timeout(msg.into())
    }

    /// Get error metadata
    pub fn metadata(&self) -> ErrorMetadata {
        match self {
            Error::Config(_) => ErrorMetadata {
                code: "CONFIG_ERROR".into(),
                severity: ErrorSeverity::High,
                recoverable: true,
                context: None,
            },
            Error::Consensus(_) => ErrorMetadata {
                code: "CONSENSUS_ERROR".into(),
                severity: ErrorSeverity::Critical,
                recoverable: false,
                context: None,
            },
            Error::Storage(_) => ErrorMetadata {
                code: "STORAGE_ERROR".into(),
                severity: ErrorSeverity::High,
                recoverable: true,
                context: None,
            },
            Error::Network(_) => ErrorMetadata {
                code: "NETWORK_ERROR".into(),
                severity: ErrorSeverity::Medium,
                recoverable: true,
                context: None,
            },
            Error::Runtime(_) => ErrorMetadata {
                code: "RUNTIME_ERROR".into(),
                severity: ErrorSeverity::High,
                recoverable: false,
                context: None,
            },
            Error::System(_) => ErrorMetadata {
                code: "SYSTEM_ERROR".into(),
                severity: ErrorSeverity::Critical,
                recoverable: false,
                context: None,
            },
            Error::Validation(_) => ErrorMetadata {
                code: "VALIDATION_ERROR".into(),
                severity: ErrorSeverity::Low,
                recoverable: true,
                context: None,
            },
            Error::Database(_) => ErrorMetadata {
                code: "DATABASE_ERROR".into(),
                severity: ErrorSeverity::High,
                recoverable: true,
                context: None,
            },
            Error::Io(_) => ErrorMetadata {
                code: "IO_ERROR".into(),
                severity: ErrorSeverity::Medium,
                recoverable: true,
                context: None,
            },
            Error::Serialization(_) => ErrorMetadata {
                code: "SERIALIZATION_ERROR".into(),
                severity: ErrorSeverity::Low,
                recoverable: true,
                context: None,
            },
            Error::InvalidState(_) => ErrorMetadata {
                code: "INVALID_STATE_ERROR".into(),
                severity: ErrorSeverity::High,
                recoverable: false,
                context: None,
            },
            Error::NotFound(_) => ErrorMetadata {
                code: "NOT_FOUND_ERROR".into(),
                severity: ErrorSeverity::Low,
                recoverable: true,
                context: None,
            },
            Error::Timeout(_) => ErrorMetadata {
                code: "TIMEOUT_ERROR".into(),
                severity: ErrorSeverity::Medium,
                recoverable: true,
                context: None,
            },
        }
    }

    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        self.metadata().severity
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.metadata().recoverable
    }

    /// Get the error code
    pub fn code(&self) -> String {
        self.metadata().code
    }

    /// Add context to an error
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        let mut metadata = self.metadata();
        metadata.context = Some(context.into());
        self
    }
}

/// Helper trait for adding context to Result types
pub trait ResultExt<T, E> {
    /// Add context to an error result
    fn with_context<S: Into<String>>(self, context: S) -> Result<T>;
}

impl<T, E: Into<Error>> ResultExt<T, E> for std::result::Result<T, E> {
    fn with_context<S: Into<String>>(self, context: S) -> Result<T> {
        self.map_err(|e| e.into().with_context(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_metadata() {
        let error = Error::config("test error");
        let metadata = error.metadata();
        
        assert_eq!(metadata.code, "CONFIG_ERROR");
        assert_eq!(metadata.severity, ErrorSeverity::High);
        assert!(metadata.recoverable);
    }

    #[test]
    fn test_error_with_context() {
        let error = Error::system("base error")
            .with_context("additional context");
            
        let metadata = error.metadata();
        assert_eq!(metadata.context.unwrap(), "additional context");
    }

    #[test]
    fn test_result_context_extension() {
        let result: Result<()> = Err(Error::not_found("test"))
            .with_context("custom context");
            
        assert!(matches!(result, Err(Error::NotFound(_))));
        
        if let Err(e) = result {
            assert_eq!(e.metadata().context.unwrap(), "custom context");
        }
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "io error"
        );
        let error: Error = io_error.into();
        
        assert!(matches!(error, Error::Io(_)));
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_display() {
        let error = Error::validation("invalid input");
        assert_eq!(
            error.to_string(),
            "Validation error: invalid input"
        );
    }
}

// Implement conversion from specific error types
impl From<icn_types::CoreError> for Error {
    fn from(err: icn_types::CoreError) -> Self {
        Error::System(err.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::System(format!("Task join error: {}", err))
    }
}

impl From<tokio::sync::broadcast::error::SendError<crate::SystemEvent>> for Error {
    fn from(err: tokio::sync::broadcast::error::SendError<crate::SystemEvent>) -> Self {
        Error::System(format!("Broadcast send error: {}", err))
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(err.to_string())
    }
}

// Add additional conversions for other error types as needed
