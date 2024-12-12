use crate::ConsensusConfig;
use crate::metrics::ConsensusMetrics;
use crate::error::{ConsensusError, ConsensusResult};
use icn_types::{Block, DID};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub struct ProofOfCooperation {
    config: ConsensusConfig,
    metrics: ConsensusMetrics,
    validators: HashMap<DID, f64>, // DID -> reputation score
    current_round: Option<u64>,
    state: Arc<RwLock<NetworkState>>,
}

#[derive(Clone, Debug)]
struct NetworkState {
    block_height: u64,
    state_root: String,
}

impl ProofOfCooperation {
    pub fn new(config: ConsensusConfig) -> Self {
        Self {
            config,
            metrics: ConsensusMetrics::new(),
            validators: HashMap::new(),
            current_round: None,
            state: Arc::new(RwLock::new(NetworkState {
                block_height: 0,
                state_root: compute_initial_state_root(),
            })),
        }
    }

    pub async fn start_round(&mut self) -> ConsensusResult<()> {
        debug!("Starting new consensus round");
        self.current_round = Some(self.current_round.unwrap_or(0) + 1);
        self.metrics.rounds_total.inc();
        Ok(())
    }

    pub async fn add_validator(&mut self, did: DID, initial_reputation: f64) {
        self.validators.insert(did, initial_reputation);
        self.metrics.active_validators.inc();
    }

    pub async fn propose_block(&self, block: Block) -> ConsensusResult<()> {
        let state = self.state.read().await;
        if block.height != state.block_height + 1 {
            return Err(ConsensusError::InvalidBlockHeight);
        }
        if block.previous_hash != state.state_root {
            return Err(ConsensusError::InvalidPreviousHash);
        }
        Ok(())
    }

    pub async fn verify_block(&self, block: &Block) -> ConsensusResult<()> {
        let state = self.state.read().await;
        if block.height != state.block_height + 1 {
            return Err(ConsensusError::InvalidBlockHeight);
        }
        if block.previous_hash != state.state_root {
            return Err(ConsensusError::InvalidPreviousHash);
        }
        Ok(())
    }

    pub async fn submit_vote(&self, validator_did: DID, approve: bool) -> ConsensusResult<()> {
        if !self.validators.contains_key(&validator_did) {
            return Err(ConsensusError::UnknownValidator);
        }
        Ok(())
    }

    pub async fn has_consensus(&self) -> ConsensusResult<bool> {
        Ok(true)
    }
}

fn compute_initial_state_root() -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"initial_state");
    hex::encode(hasher.finalize())
}
