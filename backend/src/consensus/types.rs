use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

/// Represents a validator node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// The validator's DID
    pub did: String,
    
    /// Current reputation score
    pub reputation: i64,
    
    /// Block number of the last block this validator proposed
    pub last_block_proposed: u64,
    
    /// Number of consecutive validation rounds missed
    pub consecutive_missed_validations: u32,
    
    /// Whether the validator is currently active
    pub is_active: bool,
    
    /// The validator's voting power in the current round
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voting_power: Option<f64>,
}

impl Validator {
    pub fn new(did: String, initial_reputation: i64) -> Self {
        Self {
            did,
            reputation: initial_reputation,
            last_block_proposed: 0,
            consecutive_missed_validations: 0,
            is_active: true,
            voting_power: None,
        }
    }

    pub fn calculate_voting_power(&mut self, total_reputation: i64) {
        self.voting_power = Some(self.reputation as f64 / total_reputation as f64);
    }

    pub fn is_eligible(&self, min_reputation: i64) -> bool {
        self.is_active && self.reputation >= min_reputation
    }
}

/// Represents the type of consensus message being sent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusMessage {
    /// Proposal for a new block
    BlockProposal(Block),
    
    /// Vote for a proposed block
    Vote {
        /// The voting validator's DID
        validator: String,
        
        /// The hash of the block being voted on
        block_hash: String,
        
        /// Whether the validator approves the block
        approve: bool,
    },
    
    /// Request to start a new round
    StartRound {
        round_number: u64,
    },
    
    /// Notification that a round has been finalized
    RoundFinalized {
        round_number: u64,
        block_hash: String,
    },
}

/// Status of a consensus round
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    /// Block is being proposed by coordinator
    Proposing,
    
    /// Validators are voting on proposed block
    Voting,
    
    /// Round is being finalized
    Finalizing,
    
    /// Round completed successfully
    Completed,
    
    /// Round failed (timeout or other error)
    Failed,
}

/// Represents a single round of consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    /// Unique round number
    pub round_number: u64,
    
    /// DID of the round's coordinator
    pub coordinator: String,
    
    /// The block being proposed (if any)
    pub proposed_block: Option<Block>,
    
    /// Map of validator DIDs to their votes
    pub votes: HashMap<String, bool>,
    
    /// Current status of the round
    pub status: RoundStatus,
    
    /// Timestamp when the round started
    pub start_time: u64,
    
    /// Maximum duration for the round in seconds
    pub timeout: u64,
}

impl ConsensusRound {
    pub fn new(round_number: u64, coordinator: String, timeout: u64, start_time: u64) -> Self {
        Self {
            round_number,
            coordinator,
            proposed_block: None,
            votes: HashMap::new(),
            status: RoundStatus::Proposing,
            start_time,
            timeout,
        }
    }

    pub fn has_voted(&self, validator_did: &str) -> bool {
        self.votes.contains_key(validator_did)
    }

    pub fn add_vote(&mut self, validator_did: String, approve: bool) {
        self.votes.insert(validator_did, approve);
    }

    pub fn get_approval_rate(&self) -> f64 {
        if self.votes.is_empty() {
            return 0.0;
        }
        
        let approve_count = self.votes.values().filter(|&&approve| approve).count();
        approve_count as f64 / self.votes.len() as f64
    }

    pub fn is_timed_out(&self, current_time: u64) -> bool {
        current_time - self.start_time > self.timeout
    }
}

/// Configuration for the consensus mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum number of validators required for consensus
    pub min_validators: usize,
    
    /// Threshold of votes required to approve a block (0.0 - 1.0)
    pub vote_threshold: f64,
    
    /// Round timeout in seconds
    pub round_timeout: u64,
    
    /// Minimum reputation required to be a validator
    pub min_reputation: i64,
    
    /// Base reward for participating in consensus
    pub participation_reward: i64,
    
    /// Extra reward for being the coordinator
    pub coordinator_reward: i64,
    
    /// Penalty for missing validation
    pub missed_validation_penalty: i64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 3,
            vote_threshold: 0.66,  // 66% approval required
            round_timeout: 60,     // 60 seconds
            min_reputation: 50,
            participation_reward: 1,
            coordinator_reward: 2,
            missed_validation_penalty: -1,
        }
    }
}

/// Result of a consensus round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round_number: u64,
    pub successful: bool,
    pub finalized_block: Option<Block>,
    pub reputation_updates: Vec<(String, i64)>,
    pub participating_validators: Vec<String>,
}

/// Error types specific to consensus operations
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