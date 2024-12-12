// crates/icn-consensus/src/error.rs

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

// crates/icn-consensus/src/engine.rs

use tokio::sync::RwLock;
use std::sync::Arc;
use crate::error::{ConsensusError, ConsensusResult};
use crate::proof_of_cooperation::ProofOfCooperation;
use crate::state::StateManager;

/// Core consensus engine implementation
pub struct ConsensusEngine {
    consensus: Arc<RwLock<ProofOfCooperation>>,
    state: Arc<StateManager>,
    initialized: bool,
}

impl ConsensusEngine {
    /// Creates a new consensus engine instance
    pub async fn new(config: crate::ConsensusConfig) -> ConsensusResult<Self> {
        let state = Arc::new(StateManager::new().await?);
        let (consensus, _) = ProofOfCooperation::new(config);
        
        Ok(Self {
            consensus: Arc::new(RwLock::new(consensus)),
            state,
            initialized: true,
        })
    }

    /// Returns whether the engine is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Starts the consensus engine
    pub async fn start(&self) -> ConsensusResult<()> {
        let mut consensus = self.consensus.write().await;
        consensus.start_round().await
    }

    /// Stops the consensus engine
    pub async fn stop(&self) -> ConsensusResult<()> {
        let mut consensus = self.consensus.write().await;
        if let Some(round) = consensus.get_current_round().await {
            consensus.fail_round(format!("Engine shutdown")).await?;
        }
        Ok(())
    }
}