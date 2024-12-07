use crate::ConsensusConfig;
use crate::metrics::ConsensusMetrics;
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

    pub async fn start_round(&mut self) -> anyhow::Result<()> {
        debug!("Starting new consensus round");
        Ok(())
    }

    pub async fn add_validator(&mut self, did: DID, initial_reputation: f64) {
        self.validators.insert(did, initial_reputation);
        self.metrics.active_validators.inc();
    }
}

fn compute_initial_state_root() -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"initial_state");
    hex::encode(hasher.finalize())
}
