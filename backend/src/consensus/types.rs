use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusError {
    InsufficientValidators,
    InvalidCoordinator,
    RoundInProgress,
    NoActiveRound,
    InvalidRoundState,
    TimedOut,
    ValidationFailed,
    NotValidator,
    InsufficientReputation,
    InsufficientSignatures,
    InvalidBlockIndex,
    InvalidPreviousHash,
    InvalidTimestamp,
    InvalidStateTransition,
    InvalidBlockHash,
    InvalidValidatorUpdate,
    Custom(String),
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::InsufficientValidators => 
                write!(f, "Insufficient number of active validators"),
            ConsensusError::InvalidCoordinator => 
                write!(f, "Invalid coordinator for this round"),
            ConsensusError::RoundInProgress => 
                write!(f, "Consensus round already in progress"),
            ConsensusError::NoActiveRound => 
                write!(f, "No active consensus round"),
            ConsensusError::InvalidRoundState => 
                write!(f, "Invalid round state for requested operation"),
            ConsensusError::TimedOut => 
                write!(f, "Consensus round timed out"),
            ConsensusError::ValidationFailed => 
                write!(f, "Block validation failed"),
            ConsensusError::NotValidator => 
                write!(f, "Not a registered validator"),
            ConsensusError::InsufficientReputation => 
                write!(f, "Insufficient reputation for operation"),
            ConsensusError::InsufficientSignatures =>
                write!(f, "Insufficient validator signatures"),
            ConsensusError::InvalidBlockIndex =>
                write!(f, "Invalid block index"),
            ConsensusError::InvalidPreviousHash =>
                write!(f, "Invalid previous block hash"),
            ConsensusError::InvalidTimestamp =>
                write!(f, "Invalid block timestamp"),
            ConsensusError::InvalidStateTransition =>
                write!(f, "Invalid state transition"),
            ConsensusError::InvalidBlockHash =>
                write!(f, "Invalid block hash"),
            ConsensusError::InvalidValidatorUpdate =>
                write!(f, "Invalid validator update"),
            ConsensusError::Custom(msg) => write!(f, "{}", msg),
        }
    }
