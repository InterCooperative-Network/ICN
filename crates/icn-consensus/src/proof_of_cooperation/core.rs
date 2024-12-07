// src/consensus/proof_of_cooperation/core.rs

use std::sync::Arc;
use tokio::sync::broadcast;
use chrono::Utc;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

use crate::websocket::WebSocketHandler;
use crate::blockchain::{Block, Transaction};
use crate::consensus::types::{
    ConsensusConfig, ConsensusError, ConsensusRound, RoundStatus,
    ConsensusRoundStats, ValidatorInfo
};
use super::{
    validator::ValidatorManager,
    round::RoundManager,
    events::ConsensusEvent,
};

/// Represents the state of the cooperative network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkState {
    pub block_height: u64,
    pub state_root: String,
    pub validator_set: Vec<ValidatorInfo>,
    pub timestamp: i64,
    pub merkle_proof: Vec<String>,
}

pub struct ProofOfCooperation {
    pub validator_manager: ValidatorManager,
    pub round_manager: RoundManager,
    config: ConsensusConfig,
    reputation_updates: Vec<(String, i64)>,
    ws_handler: Arc<WebSocketHandler>,
    event_tx: broadcast::Sender<ConsensusEvent>,
    state: NetworkState,
}

impl ProofOfCooperation {
    pub fn new(config: ConsensusConfig, ws_handler: Arc<WebSocketHandler>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let initial_state = NetworkState {
            block_height: 0,
            state_root: "genesis".to_string(),
            validator_set: Vec::new(),
            timestamp: Utc::now().timestamp(),
            merkle_proof: Vec::new(),
        };
        
        ProofOfCooperation {
            validator_manager: ValidatorManager::new(config.clone()),
            round_manager: RoundManager::new(config.clone()),
            config,
            reputation_updates: Vec::new(),
            ws_handler,
            event_tx,
            state: initial_state,
        }
    }

    pub async fn start_round(&mut self) -> Result<(), ConsensusError> {
        // Clean up inactive validators
        self.validator_manager.cleanup_inactive_validators();

        // Get active validators meeting requirements
        let active_validators: Vec<_> = self.validator_manager.get_validators().values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation &&
                      v.performance_score >= self.config.min_performance_score)
            .collect();

        if active_validators.len() < self.config.min_validators {
            return Err(ConsensusError::InsufficientValidators);
        }

        // Select coordinator using weighted reputation
        let coordinator = self.validator_manager.select_coordinator(&active_validators)?;

        // Calculate total voting power
        let total_voting_power: f64 = active_validators.iter()
            .map(|v| v.voting_power)
            .sum();

        // Start new consensus round
        let event = self.round_manager.start_round(
            self.get_next_round_number(),
            coordinator.did.clone(),
            total_voting_power,
            active_validators.len(),
        )?;

        // Update state
        self.state.timestamp = Utc::now().timestamp();
        self.state.validator_set = active_validators.iter().map(|v| (*v).clone()).collect();

        // Broadcast updates
        if let Some(round) = self.round_manager.get_current_round() {
            self.ws_handler.broadcast_consensus_update(round);
        }
        let _ = self.event_tx.send(event);

        Ok(())
    }

    pub fn verify_block_signatures(&self, block: &Block, signatures: &[String]) -> Result<(), ConsensusError> {
        let block_hash = self.calculate_block_hash(block);
        
        // Verify we have enough valid signatures
        let valid_signatures = signatures.iter()
            .filter(|sig| self.verify_signature(&block_hash, sig))
            .count();
            
        let required_signatures = (self.config.min_validators as f64 * self.config.min_approval_rate) as usize;
        
        if valid_signatures < required_signatures {
            return Err(ConsensusError::InsufficientSignatures);
        }

        Ok(())
    }

    fn verify_signature(&self, message: &str, signature: &str) -> bool {
        // TODO: Implement actual signature verification
        // For now, return true for testing
        true
    }

    fn calculate_block_hash(&self, block: &Block) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", 
            block.index,
            block.previous_hash,
            block.timestamp
        ));
        format!("{:x}", hasher.finalize())
    }

    pub async fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), ConsensusError> {
        // Validate proposer
        let validator = self.validator_manager.get_validator(proposer_did)
            .ok_or(ConsensusError::NotValidator)?;

        if validator.reputation < self.config.min_validator_reputation {
            return Err(ConsensusError::InsufficientReputation);
        }

        // Validate block
        self.validate_block(&block)?;

        // Process proposal
        let event = self.round_manager.propose_block(proposer_did, block.clone())?;

        // Update state
        self.state.block_height = block.index;

        // Broadcast updates
        self.ws_handler.broadcast_block_finalized(&block);
        let _ = self.event_tx.send(event);

        Ok(())
    }

    fn validate_block(&self, block: &Block) -> Result<(), ConsensusError> {
        // Verify block index
        if block.index != self.state.block_height + 1 {
            return Err(ConsensusError::InvalidBlockIndex);
        }

        // Verify previous hash
        if block.previous_hash != self.state.state_root {
            return Err(ConsensusError::InvalidPreviousHash);
        }

        // Verify timestamp
        if block.timestamp <= self.state.timestamp {
            return Err(ConsensusError::InvalidTimestamp);
        }

        // Verify transactions
        for tx in &block.transactions {
            self.validate_transaction(tx)?;
        }

        Ok(())
    }

    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), ConsensusError> {
        // TODO: Implement full transaction validation
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

        // Update state root
        self.state.state_root = self.calculate_block_hash(&block);

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

    pub fn get_network_state(&self) -> &NetworkState {
        &self.state
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
        ProofOfCooperation::new(config, ws_handler)
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
    async fn test_block_validation() {
        let mut consensus = setup_test_consensus().await;
        let initial_state = consensus.get_network_state();

        let valid_block = Block::new(
            initial_state.block_height + 1,
            initial_state.state_root.clone(),
            vec![],
            "test_proposer".to_string(),
        );

        assert!(consensus.validate_block(&valid_block).is_ok());

        let invalid_block = Block::new(
            initial_state.block_height + 2, // Wrong index
            "invalid_hash".to_string(),     // Wrong previous hash
            vec![],
            "test_proposer".to_string(),
        );

        assert!(consensus.validate_block(&invalid_block).is_err());
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