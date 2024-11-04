// src/consensus/types.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub did: String,
    pub reputation: i64,
    pub voting_power: f64,
    pub last_active_round: u64,
    pub consecutive_missed_rounds: u32,
    pub total_blocks_validated: u64,
    pub performance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRoundStats {
    pub total_voting_power: f64,
    pub participation_rate: f64,
    pub approval_rate: f64,
    pub round_duration_ms: u64,
    pub validator_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedVote {
    pub validator: String,
    pub approve: bool,
    pub voting_power: f64,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,
    pub start_time: DateTime<Utc>,
    pub timeout: DateTime<Utc>,
    pub status: RoundStatus,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, WeightedVote>,
    pub stats: ConsensusRoundStats,
}

impl ConsensusRound {
    pub fn duration_ms(&self) -> i64 {
        (Utc::now() - self.start_time).num_milliseconds()
    }

    pub fn is_timed_out(&self) -> bool {
        Utc::now() > self.timeout
    }

    pub fn get_participation_rate(&self, total_voting_power: f64) -> f64 {
        let votes_power: f64 = self.votes.values()
            .map(|v| v.voting_power)
            .sum();
        votes_power / total_voting_power
    }

    pub fn get_approval_rate(&self) -> f64 {
        let total_votes_power: f64 = self.votes.values()
            .map(|v| v.voting_power)
            .sum();
        
        if total_votes_power <= 0.0 {
            return 0.0;
        }

        let approval_power: f64 = self.votes.values()
            .filter(|v| v.approve)
            .map(|v| v.voting_power)
            .sum();

        approval_power / total_votes_power
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Proposing,
    Voting,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_validator_reputation: i64,
    pub max_voting_power: f64,
    pub min_participation_rate: f64,
    pub min_approval_rate: f64,
    pub round_timeout_ms: u64,
    pub base_reward: i64,
    pub penalty_factor: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            min_validator_reputation: 100,
            max_voting_power: 0.1,
            min_participation_rate: 0.67,
            min_approval_rate: 0.67,
            round_timeout_ms: 30_000,
            base_reward: 10,
            penalty_factor: 1.5,
        }
    }
}

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

impl From<ConsensusError> for String {
    fn from(error: ConsensusError) -> String {
        error.to_string()
    }
}

// Helper functions for consensus calculations
pub mod utils {
    pub fn calculate_voting_power(reputation: i64, total_reputation: i64, max_power: f64) -> f64 {
        let raw_power = reputation as f64 / total_reputation as f64;
        raw_power.min(max_power)
    }

    pub fn calculate_penalty(base_penalty: i64, consecutive_misses: u32, factor: f64) -> i64 {
        -(base_penalty as f64 * factor * consecutive_misses as f64) as i64
    }
}