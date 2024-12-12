// crates/icn-consensus/src/lib.rs

//! ICN Consensus implementation
//! 
//! This crate provides the consensus mechanism for the Inter-Cooperative Network,
//! implementing a custom Proof of Cooperation protocol.

mod crypto;
mod error;
mod state;
mod proof_of_cooperation;
mod engine;

pub use crypto::{CryptoManager, CryptoError, CryptoResult};
pub use error::{ConsensusError, ConsensusResult};
pub use state::{StateManager, ConsensusState};
pub use proof_of_cooperation::{ProofOfCooperation, ConsensusEvent, ConsensusConfig, RoundStatus};
pub use engine::ConsensusEngine;

use std::collections::HashMap;

/// Core validator information tracking
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Unique DID of the validator
    pub did: String,
    /// Current reputation score
    pub reputation: i64,
    /// Number of consecutive rounds as coordinator
    pub consecutive_rounds: usize,
}

/// Type alias for the validator set
pub type ValidatorSet = HashMap<String, ValidatorInfo>;

/// Initialize the consensus system with the given configuration
pub async fn init(config: ConsensusConfig) -> ConsensusResult<ConsensusEngine> {
    ConsensusEngine::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_consensus_initialization() {
        let config = ConsensusConfig::default();
        let engine = init(config).await.unwrap();
        assert!(engine.is_initialized());
    }
}
