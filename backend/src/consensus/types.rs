// src/consensus/types.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

/// Events emitted during consensus process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusEvent {
    /// Round started event
    RoundStarted {
        round: u64,
        coordinator: String,
        timeout: u64,
    },
    
    /// Block proposed event
    BlockProposed {
        round: u64,
        proposer: String,
        block_hash: String,
        transactions: usize,
    },
    
    /// Vote received event
    VoteReceived {
        round: u64,
        validator: String,
        approve: bool,
        voting_power: f64,
    },
    
    /// Round completed event
    RoundCompleted {
        round: u64,
        block_hash: String,
        validators: Vec<String>,
        duration_ms: u64,
    },
    
    /// Round failed event
    RoundFailed {
        round: u64,
        reason: String,
    },
    
    /// Validator status update
    ValidatorUpdate {
        did: String,
        reputation: i64,
        voting_power: f64,
        performance_score: f64,
    },
}

/// Information about a validator in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Decentralized Identifier of the validator
    pub did: String,
    
    /// Current reputation score
    pub reputation: i64,
    
    /// Calculated voting power (0.0 to 1.0)
    pub voting_power: f64,
    
    /// Last consensus round the validator participated in
    pub last_active_round: u64,
    
    /// Number of consecutive rounds missed
    pub consecutive_missed_rounds: u32,
    
    /// Total number of blocks validated
    pub total_blocks_validated: u64,
    
    /// Performance score based on historical participation (0.0 to 1.0)
    pub performance_score: f64,
}

/// Statistics for a consensus round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRoundStats {
    /// Total available voting power in the round
    pub total_voting_power: f64,
    
    /// Percentage of validators who participated
    pub participation_rate: f64,
    
    /// Percentage of voting power that approved the block
    pub approval_rate: f64,
    
    /// Duration of the round in milliseconds
    pub round_duration_ms: u64,
    
    /// Number of validators eligible to participate
    pub validator_count: usize,
}

/// A vote cast by a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedVote {
    /// DID of the validator casting the vote
    pub validator: String,
    
    /// Whether the validator approves the block
    pub approve: bool,
    
    /// Voting power of the validator
    pub voting_power: f64,
    
    /// When the vote was cast
    pub timestamp: DateTime<Utc>,
    
    /// Cryptographic signature of the vote
    pub signature: String,
}

/// State of a consensus round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    /// Sequential number of this round
    pub round_number: u64,
    
    /// DID of the coordinator for this round
    pub coordinator: String,
    
    /// When the round started
    pub start_time: DateTime<Utc>,
    
    /// When the round will timeout
    pub timeout: DateTime<Utc>,
    
    /// Current status of the round
    pub status: RoundStatus,
    
    /// The block being proposed (if any)
    pub proposed_block: Option<Block>,
    
    /// Votes received, keyed by validator DID
    pub votes: HashMap<String, WeightedVote>,
    
    /// Statistics for the round
    pub stats: ConsensusRoundStats,
}

/// Status of a consensus round
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    /// Waiting for block proposal
    Proposing,
    
    /// Collecting votes from validators
    Voting,
    
    /// Preparing to commit the block
    Finalizing,
    
    /// Round successfully completed
    Completed,
    
    /// Round failed to reach consensus
    Failed,
}

/// Configuration parameters for the consensus mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum reputation required to be a validator
    pub min_validator_reputation: i64,
    
    /// Maximum voting power any validator can have
    pub max_voting_power: f64,
    
    /// Minimum participation required for valid consensus
    pub min_participation_rate: f64,
    
    /// Minimum approval required for valid consensus
    pub min_approval_rate: f64,
    
    /// How long before a round times out (milliseconds)
    pub round_timeout_ms: u64,
    
    /// Base reputation reward for participation
    pub base_reward: i64,
    
    /// Multiplier for consecutive missed rounds
    pub penalty_factor: f64,
    
    /// Minimum number of validators needed
    pub min_validators: usize,
    
    /// Maximum consecutive missed rounds before ejection
    pub max_missed_rounds: u32,
    
    /// Minimum performance score to remain eligible 
    pub min_performance_score: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            min_validator_reputation: 100,
            max_voting_power: 0.1,
            min_participation_rate: 0.67,  // 2/3 participation required
            min_approval_rate: 0.67,       // 2/3 approval required
            round_timeout_ms: 30_000,      // 30 second timeout
            base_reward: 10,
            penalty_factor: 1.5,
            min_validators: 3,
            max_missed_rounds: 5,
            min_performance_score: 0.5,
        }
    }
}

/// Possible consensus-related errors
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
    // Add missing variants
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
            ConsensusError::Custom(msg) => 
                write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConsensusError {}

impl From<ConsensusError> for String {
    fn from(error: ConsensusError) -> String {
        error.to_string()
    }
}

/// Utility functions for consensus calculations
pub mod utils {
    /// Calculate voting power based on reputation
    pub fn calculate_voting_power(reputation: i64, total_reputation: i64, max_power: f64) -> f64 {
        let raw_power = reputation as f64 / total_reputation as f64;
        raw_power.min(max_power)
    }

    /// Calculate penalty for missed rounds
    pub fn calculate_penalty(base_penalty: i64, consecutive_misses: u32, factor: f64) -> i64 {
        -(base_penalty as f64 * factor * consecutive_misses as f64) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voting_power_calculation() {
        let power = utils::calculate_voting_power(500, 1000, 0.1);
        assert!(power <= 0.1);
        assert!(power > 0.0);
        assert_eq!(power, 0.1); // Should be capped at max_power
        
        let power = utils::calculate_voting_power(100, 1000, 0.1);
        assert_eq!(power, 0.1);
    }

    #[test]
    fn test_penalty_calculation() {
        let penalty = utils::calculate_penalty(10, 3, 1.5);
        assert_eq!(penalty, -45); // -10 * 1.5 * 3

        let penalty1 = utils::calculate_penalty(10, 1, 1.5);
        let penalty2 = utils::calculate_penalty(10, 2, 1.5);
        assert!(penalty2 < penalty1); 
    }

    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.min_validator_reputation, 100);
        assert_eq!(config.max_voting_power, 0.1);
        assert_eq!(config.min_participation_rate, 0.67);
        assert_eq!(config.min_approval_rate, 0.67);
        assert_eq!(config.round_timeout_ms, 30_000);
    }
}