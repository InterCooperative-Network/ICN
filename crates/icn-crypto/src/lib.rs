// src/lib.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

mod config;
mod error;
mod metrics;
mod proof_of_cooperation;

pub use config::ConsensusConfig;
pub use error::{ConsensusError, ConsensusResult};
pub use metrics::ConsensusMetrics;
pub use proof_of_cooperation::{ProofOfCooperation, ConsensusEvent};

/// Atomic round counter for unique round identification
static ROUND_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Core consensus engine interface
#[async_trait::async_trait]
pub trait ConsensusEngine: Send + Sync {
    async fn start_round(&mut self) -> ConsensusResult<()>;
    async fn propose_block(&mut self, block: icn_types::Block) -> ConsensusResult<()>;
    async fn verify_block(&self, block: &icn_types::Block) -> ConsensusResult<()>;
    async fn submit_vote(&mut self, validator_did: String, approve: bool, signature: Vec<u8>) -> ConsensusResult<()>;
    async fn has_consensus(&self) -> ConsensusResult<bool>;
}

/// Vote tracking with signature verification
#[derive(Debug, Clone)]
struct Vote {
    validator: String,
    approve: bool,
    signature: Vec<u8>,
    timestamp: Instant,
}

/// Round state management
#[derive(Debug)]
pub struct ConsensusRound {
    id: u64,
    start_time: Instant,
    timeout: Duration,
    votes: RwLock<HashMap<String, Vote>>,
    proposed_block: Option<icn_types::Block>,
    finalized: bool,
}

impl ConsensusRound {
    fn new(timeout: Duration) -> Self {
        Self {
            id: ROUND_COUNTER.fetch_add(1, Ordering::SeqCst),
            start_time: Instant::now(),
            timeout,
            votes: RwLock::new(HashMap::new()),
            proposed_block: None,
            finalized: false,
        }
    }

    async fn add_vote(&self, vote: Vote) -> ConsensusResult<()> {
        let mut votes = self.votes.write().await;
        
        // Check for duplicate votes
        if votes.contains_key(&vote.validator) {
            return Err(ConsensusError::DuplicateVote);
        }
        
        // Verify vote signature
        if !self.verify_vote_signature(&vote).await? {
            return Err(ConsensusError::InvalidSignature);
        }
        
        votes.insert(vote.validator.clone(), vote);
        Ok(())
    }

    async fn verify_vote_signature(&self, vote: &Vote) -> ConsensusResult<bool> {
        // TODO: Implement actual signature verification
        Ok(true)
    }

    fn is_timed_out(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }
}

/// Configuration options
#[derive(Debug, Clone)]
pub struct ConsensusOpts {
    pub min_validators: usize,
    pub round_timeout: Duration,
    pub consensus_threshold: f64,
    pub max_timestamp_diff: Duration,
    pub event_channel_size: usize,
    pub signature_verification: bool,
}

impl Default for ConsensusOpts {
    fn default() -> Self {
        Self {
            min_validators: 4,
            round_timeout: Duration::from_secs(30),
            consensus_threshold: 0.66,
            max_timestamp_diff: Duration::from_secs(60),
            event_channel_size: 1000,
            signature_verification: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_consensus_round_voting() {
        let round = ConsensusRound::new(Duration::from_secs(30));
        
        let vote = Vote {
            validator: "test_validator".to_string(),
            approve: true,
            signature: vec![],
            timestamp: Instant::now(),
        };
        
        assert!(round.add_vote(vote.clone()).await.is_ok());
        assert!(round.add_vote(vote.clone()).await.is_err()); // Duplicate vote
    }

    #[tokio::test]
    async fn test_round_timeout() {
        let round = ConsensusRound::new(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(2)).await;
        assert!(round.is_timed_out());
    }
}