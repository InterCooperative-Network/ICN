// src/error.rs

use thiserror::Error;

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Errors that can occur during consensus operations
#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    #[error("Not enough active validators (required: {required}, actual: {actual})")]
    InsufficientValidators {
        required: usize,
        actual: usize,
    },

    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),

    #[error("Validator {0} is already registered")]
    ValidatorAlreadyRegistered(String),

    #[error("Unauthorized proposer: {0}")]
    UnauthorizedProposer(String),

    #[error("Invalid block height: expected {expected}, got {actual}")]
    InvalidBlockHeight {
        expected: u64,
        actual: u64,
    },

    #[error("Invalid block timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Round timeout after {duration_secs} seconds")]
    RoundTimeout {
        duration_secs: u64,
    },

    #[error("Insufficient voting power (required: {required}, actual: {actual})")]
    InsufficientVotingPower {
        required: f64,
        actual: f64,
    },

    #[error("Invalid vote: {0}")]
    InvalidVote(String),

    #[error("Consensus failed: {0}")]
    ConsensusFailed(String),

    #[error("Event channel error: {0}")]
    EventError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ConsensusError {
    /// Returns true if the error represents a timeout
    pub fn is_timeout(&self) -> bool {
        matches!(self, ConsensusError::RoundTimeout { .. })
    }

    /// Returns true if the error is related to validator issues
    pub fn is_validator_error(&self) -> bool {
        matches!(
            self,
            ConsensusError::ValidatorNotFound(_) |
            ConsensusError::ValidatorAlreadyRegistered(_) |
            ConsensusError::UnauthorizedProposer(_) |
            ConsensusError::InsufficientValidators { .. } |
            ConsensusError::InsufficientVotingPower { .. }
        )
    }

    /// Returns true if the error is related to block validation
    pub fn is_block_error(&self) -> bool {
        matches!(
            self,
            ConsensusError::InvalidBlock(_) |
            ConsensusError::InvalidBlockHeight { .. } |
            ConsensusError::InvalidTimestamp(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let timeout_err = ConsensusError::RoundTimeout { duration_secs: 30 };
        assert!(timeout_err.is_timeout());
        assert!(!timeout_err.is_validator_error());
        assert!(!timeout_err.is_block_error());

        let validator_err = ConsensusError::ValidatorNotFound("did:icn:test".into());
        assert!(!validator_err.is_timeout());
        assert!(validator_err.is_validator_error());
        assert!(!validator_err.is_block_error());

        let block_err = ConsensusError::InvalidBlockHeight { 
            expected: 1,
            actual: 2,
        };
        assert!(!block_err.is_timeout());
        assert!(!block_err.is_validator_error());
        assert!(block_err.is_block_error());
    }

    #[test]
    fn test_error_messages() {
        let err = ConsensusError::InsufficientValidators {
            required: 4,
            actual: 2,
        };
        assert_eq!(
            err.to_string(),
            "Not enough active validators (required: 4, actual: 2)"
        );

        let err = ConsensusError::InvalidBlock("missing signatures".into());
        assert_eq!(err.to_string(), "Invalid block: missing signatures");
    }
}