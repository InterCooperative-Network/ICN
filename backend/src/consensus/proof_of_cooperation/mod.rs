pub mod core;
pub mod events;
pub mod metrics;
pub mod round;
pub mod validator;

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
        
        let mut consensus = core::ProofOfCooperation::new(config, ws_handler);
        
        // Register test validators
        consensus.register_validator("did:icn:test1".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test2".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test3".to_string(), 1000).unwrap();
        
        assert!(consensus.start_round().await.is_ok());
    }
}
