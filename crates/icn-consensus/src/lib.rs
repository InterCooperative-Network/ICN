// src/lib.rs

//! ICN Consensus Library
//! 
//! This library implements the Proof of Cooperation consensus mechanism for the
//! Inter-Cooperative Network (ICN). It provides a Byzantine fault-tolerant consensus
//! system that uses reputation-weighted voting and energy-aware metrics.

mod config;
mod error;
mod metrics;
mod proof_of_cooperation;

use std::time::Duration;

pub use config::ConsensusConfig;
pub use error::{ConsensusError, ConsensusResult};
pub use metrics::ConsensusMetrics;
pub use proof_of_cooperation::{ProofOfCooperation, ConsensusEvent};

/// Trait defining the core consensus engine interface
#[async_trait::async_trait]
pub trait ConsensusEngine: Send + Sync {
    /// Start a new consensus round
    /// 
    /// This selects a coordinator and initializes the round state.
    /// Returns an error if there aren't enough active validators.
    async fn start_round(&mut self) -> ConsensusResult<()>;

    /// Propose a new block for the current round
    /// 
    /// The block must be proposed by the current round's coordinator.
    /// Returns an error if the block is invalid or the proposer isn't authorized.
    async fn propose_block(&mut self, block: icn_types::Block) -> ConsensusResult<()>;

    /// Verify a proposed block
    /// 
    /// Checks the block's validity including:
    /// - Block structure and signatures
    /// - Height continuity
    /// - Timestamp validity
    /// - Transaction validity
    async fn verify_block(&self, block: &icn_types::Block) -> ConsensusResult<()>;

    /// Submit a vote for the current round
    /// 
    /// Validators can vote to approve or reject the proposed block.
    /// Returns an error if the validator isn't authorized or has already voted.
    async fn submit_vote(&mut self, validator_did: String, approve: bool) -> ConsensusResult<()>;

    /// Check if the round has reached consensus
    /// 
    /// Returns true if enough weighted votes have been collected to reach
    /// the consensus threshold.
    async fn has_consensus(&self) -> ConsensusResult<bool>;
}

/// Configuration for the consensus system
#[derive(Debug, Clone)]
pub struct ConsensusOpts {
    /// Minimum number of validators required for consensus
    pub min_validators: usize,
    
    /// Timeout duration for consensus rounds
    pub round_timeout: Duration,
    
    /// Required threshold of weighted votes to reach consensus (0.0-1.0)
    pub consensus_threshold: f64,
    
    /// Maximum time difference allowed for block timestamps
    pub max_timestamp_diff: Duration,
    
    /// Maximum size of the event broadcast channel
    pub event_channel_size: usize,
}

impl Default for ConsensusOpts {
    fn default() -> Self {
        Self {
            min_validators: 4,
            round_timeout: Duration::from_secs(30),
            consensus_threshold: 0.66,
            max_timestamp_diff: Duration::from_secs(60),
            event_channel_size: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_consensus_opts_defaults() {
        let opts = ConsensusOpts::default();
        assert!(opts.min_validators >= 4, "Should require enough validators for BFT");
        assert!(opts.consensus_threshold > 0.66, "Should require >2/3 consensus");
        assert!(opts.round_timeout >= Duration::from_secs(30), "Should allow enough time for consensus");
    }
}