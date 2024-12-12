// crates/icn-consensus/src/state.rs

use serde::{Serialize, Deserialize};
use crate::error::{ConsensusError, ConsensusResult};

/// Represents the current consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub block_height: u64,
    pub last_block_hash: String,
    pub validator_set: crate::ValidatorSet,
    pub timestamp: i64,
}

/// Manages consensus state
pub struct StateManager {
    state: tokio::sync::RwLock<ConsensusState>,
}

impl StateManager {
    /// Creates a new state manager
    pub async fn new() -> ConsensusResult<Self> {
        Ok(Self {
            state: tokio::sync::RwLock::new(ConsensusState {
                block_height: 0,
                last_block_hash: String::new(),
                validator_set: crate::ValidatorSet::new(),
                timestamp: 0,
            }),
        })
    }

    /// Updates the consensus state
    pub async fn update_state(&self, new_state: ConsensusState) -> ConsensusResult<()> {
        let mut state = self.state.write().await;
        *state = new_state;
        Ok(())
    }

    /// Gets the current state
    pub async fn get_state(&self) -> ConsensusResult<ConsensusState> {
        Ok(self.state.read().await.clone())
    }
}