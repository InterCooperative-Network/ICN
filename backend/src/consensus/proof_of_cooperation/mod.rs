// src/consensus/proof_of_cooperation/mod.rs

pub mod core;
pub mod event;
pub mod metrics;
pub mod round;
pub mod validator;

// Re-export key components
pub use core::ProofOfCooperation;
pub use event::ConsensusEvent;
pub use validator::ValidatorManager;
pub use round::RoundManager;

// Re-export the Events type from the correct location
use crate::consensus::types::ConsensusEvent as TypesConsensusEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::types::ConsensusConfig;
    use crate::websocket::WebSocketHandler;
    use std::sync::Arc;

    async fn setup_test_consensus() -> ProofOfCooperation {
        let ws_handler = Arc::new(WebSocketHandler::new());
        let config = ConsensusConfig::default();
        ProofOfCooperation::new(config, ws_handler)
    }

    #[tokio::test]
    async fn test_consensus_initialization() {
        let consensus = setup_test_consensus().await;
        assert!(consensus.get_current_round().is_none());
    }
}