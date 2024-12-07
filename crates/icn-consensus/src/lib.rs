mod proof_of_cooperation;
mod metrics;

use icn_types as types;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub struct ConsensusConfig {
    pub min_validators: usize,
    pub round_timeout: std::time::Duration,
    pub threshold: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 3,
            round_timeout: std::time::Duration::from_secs(30),
            threshold: 0.66,
        }
    }
}

pub use proof_of_cooperation::ProofOfCooperation;
pub use metrics::ConsensusMetrics;

#[async_trait]
pub trait ConsensusEngine: Send + Sync {
    async fn start_round(&mut self) -> anyhow::Result<()>;
    async fn propose_block(&mut self, block: types::Block) -> anyhow::Result<()>;
    async fn verify_block(&self, block: &types::Block) -> anyhow::Result<()>;
}
