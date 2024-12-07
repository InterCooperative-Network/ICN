// src/consensus/mod.rs

pub mod proof_of_cooperation;
pub mod types;

// Re-export key types and modules
pub use types::ConsensusRound;
pub use proof_of_cooperation::core::ProofOfCooperation;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio;
    use crate::websocket::WebSocketHandler;
    use crate::consensus::types::ConsensusConfig;

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