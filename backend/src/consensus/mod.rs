// src/consensus/mod.rs

pub mod proof_of_cooperation;
pub mod types;

// Re-export key types and modules
pub use proof_of_cooperation::{
    core::ProofOfCooperation,
    event::ConsensusEvent,
    validator::ValidatorManager,
    round::RoundManager
};

// Re-export from types module for convenience
pub use types::{
    ConsensusConfig,
    ConsensusRound,
    ConsensusError,
    ValidatorInfo,
    WeightedVote,
    RoundStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio;
    use crate::websocket::WebSocketHandler;

    #[tokio::test]
    async fn test_consensus_integration() {
        let ws_handler = Arc::new(WebSocketHandler::new());
        let config = ConsensusConfig::default();
        
        let mut consensus = ProofOfCooperation::new(config, ws_handler);
        
        // Register test validators
        consensus.register_validator("did:icn:test1".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test2".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test3".to_string(), 1000).unwrap();
        
        assert!(consensus.start_round().await.is_ok());
    }
}