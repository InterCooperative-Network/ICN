use std::sync::Arc;
use tokio::sync::broadcast;
use crate::websocket::WebSocketHandler;
use crate::blockchain::Block;
use crate::consensus::types::{ConsensusConfig, ConsensusError, ConsensusRound};
use super::{
    validator::ValidatorManager,
    round::RoundManager,
    events::ConsensusEvent,
};
use crate::ICNCore;

pub struct ProofOfCooperation {
    // Make these public to fix the access error
    pub validator_manager: ValidatorManager,
    pub round_manager: RoundManager,
    
    config: ConsensusConfig,
    reputation_updates: Vec<(String, i64)>,
    ws_handler: Arc<WebSocketHandler>,
    event_tx: broadcast::Sender<ConsensusEvent>,
    icn_core: Arc<ICNCore>,
}

impl ProofOfCooperation {
    pub fn new(config: ConsensusConfig, ws_handler: Arc<WebSocketHandler>, icn_core: Arc<ICNCore>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        ProofOfCooperation {
            validator_manager: ValidatorManager::new(config.clone()),
            round_manager: RoundManager::new(config.clone()),
            config,
            reputation_updates: Vec::new(),
            ws_handler,
            event_tx,
            icn_core,
        }
    }

    pub async fn start_round(&mut self) -> Result<(), ConsensusError> {
        // Clean up inactive validators periodically
        self.validator_manager.cleanup_inactive_validators();

        // Get active validators meeting reputation threshold
        let active_validators: Vec<_> = self.validator_manager.get_validators().values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation &&
                      v.performance_score >= self.config.min_performance_score)
            .collect();

        if active_validators.len() < self.config.min_validators {
            return Err(ConsensusError::InsufficientValidators);
        }

        // Select coordinator
        let coordinator = self.validator_manager.select_coordinator(&active_validators)?;

        // Calculate total voting power
        let total_voting_power: f64 = active_validators.iter()
            .map(|v| v.voting_power)
            .sum();

        // Start new round
        let event = self.round_manager.start_round(
            self.get_next_round_number(),
            coordinator.did.clone(),
            total_voting_power,
            active_validators.len(),
        )?;

        // Broadcast updates
        if let Some(round) = self.round_manager.get_current_round() {
            self.ws_handler.broadcast_consensus_update(round);
        }
        let _ = self.event_tx.send(event);

        Ok(())
    }

    pub async fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), ConsensusError> {
        // Validate proposer
        let validator = self.validator_manager.get_validator(proposer_did)
            .ok_or(ConsensusError::NotValidator)?;

        if validator.reputation < self.config.min_validator_reputation {
            return Err(ConsensusError::InsufficientReputation);
        }

        // Process proposal
        let event = self.round_manager.propose_block(proposer_did, block.clone())?;

        // Broadcast updates
        self.ws_handler.broadcast_block_finalized(&block);
        let _ = self.event_tx.send(event);

        Ok(())
    }

    pub async fn submit_vote(
        &mut self,
        validator_did: &str,
        approve: bool,
        signature: String,
    ) -> Result<(), ConsensusError> {
        // Validate validator
        let validator = self.validator_manager.get_validator(validator_did)
            .ok_or(ConsensusError::NotValidator)?;

        if validator.reputation < self.config.min_validator_reputation {
            return Err(ConsensusError::InsufficientReputation);
        }

        // Submit vote
        let event = self.round_manager.submit_vote(
            validator_did.to_string(),
            approve,
            validator.voting_power,
            signature,
        )?;

        // Broadcast updates
        if let Some(round) = self.round_manager.get_current_round() {
            self.ws_handler.broadcast_consensus_update(round);
        }
        let _ = self.event_tx.send(event);

        Ok(())
    }

    pub async fn finalize_round(&mut self) -> Result<Block, ConsensusError> {
        // Finalize the round
        let (block, stats) = self.round_manager.finalize_round()?;

        // Update validator statistics
        let round = self.round_manager.get_current_round()
            .ok_or(ConsensusError::NoActiveRound)?;

        self.validator_manager.update_validator_stats(
            round.round_number,
            &round.votes.iter().map(|(k, v)| (k.clone(), v.approve)).collect(),
            &round.coordinator,
        );

        // Create round completed event
        let event = ConsensusEvent::RoundCompleted {
            round: round.round_number,
            block_hash: block.hash.clone(),
            validators: round.votes.keys().cloned().collect(),
            duration_ms: stats.round_duration_ms,
        };

        // Broadcast completion
        self.ws_handler.broadcast_block_finalized(&block);
        let _ = self.event_tx.send(event);

        Ok(block)
    }

    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), ConsensusError> {
        self.validator_manager.register_validator(did, initial_reputation)
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.round_manager.get_current_round()
    }

    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ConsensusEvent> {
        self.event_tx.subscribe()
    }

    fn get_next_round_number(&self) -> u64 {
        self.round_manager.get_round_history().len() as u64 + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_consensus() -> ProofOfCooperation {
        let ws_handler = Arc::new(WebSocketHandler::new());
        let config = ConsensusConfig::default();
        let icn_core = Arc::new(ICNCore::new());
        ProofOfCooperation::new(config, ws_handler, icn_core)
    }

    #[tokio::test]
    async fn test_start_round() {
        let mut consensus = setup_test_consensus().await;
        
        // Add test validators
        consensus.register_validator("did:icn:test1".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test2".to_string(), 1000).unwrap();
        consensus.register_validator("did:icn:test3".to_string(), 1000).unwrap();
        
        assert!(consensus.start_round().await.is_ok());
        assert!(consensus.get_current_round().is_some());
    }

    #[tokio::test]
    async fn test_full_consensus_cycle() {
        let mut consensus = setup_test_consensus().await;
        
        // Set up validators
        for i in 1..=3 {
            consensus.register_validator(format!("did:icn:test{}", i), 1000).unwrap();
        }
        
        // Start round
        consensus.start_round().await.unwrap();
        let coordinator_did = consensus.get_current_round().unwrap().coordinator.clone();
        
        // Propose block
        let block = Block::new(1, "prev_hash".to_string(), vec![], coordinator_did.clone());
        consensus.propose_block(&coordinator_did, block).await.unwrap();
        
        // Submit votes
        for i in 1..=3 {
            consensus.submit_vote(
                &format!("did:icn:test{}", i),
                true,
                "signature".to_string()
            ).await.unwrap();
        }
        
        // Finalize
        let result = consensus.finalize_round().await;
        assert!(result.is_ok());
    }
}
