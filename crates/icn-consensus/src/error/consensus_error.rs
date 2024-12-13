use thiserror::Error;
use std::result;

/// Custom error types for consensus operations
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Insufficient active validators (required: {required}, current: {current})")]
    InsufficientValidators { required: usize, current: usize },

    #[error("No active consensus round")]
    NoActiveRound,

    #[error("Invalid block height")]
    InvalidBlockHeight,

    #[error("Invalid previous block hash")]
    InvalidPreviousHash,

    #[error("Invalid block timestamp")]
    InvalidTimestamp,

    #[error("Unauthorized block proposer")]
    UnauthorizedProposer,

    #[error("Duplicate vote from validator")]
    DuplicateVote,

    #[error("Unknown validator")]
    UnknownValidator,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid DID format")]
    InvalidDID,

    #[error("Insufficient reputation (required: {required}, current: {current})")]
    InsufficientReputation { required: i64, current: i64 },

    #[error("No eligible coordinator available")]
    NoEligibleCoordinator,

    #[error("Round timeout")]
    RoundTimeout,

    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = result::Result<T, ConsensusError>;
