// src/consensus/mod.rs

pub mod proof_of_cooperation;
pub mod types;

pub use proof_of_cooperation::ProofOfCooperation;
pub use types::{ConsensusRound, ConsensusConfig, RoundStatus}; 

use std::sync::{Arc, Mutex};
use crate::blockchain::{Block, Transaction};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub consensus: Arc<Mutex<ProofOfCooperation>>,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub current_block_number: u64,
    coordinator_did: String,
}

impl Blockchain {
    pub fn new(
        identity_system: Arc<Mutex<IdentitySystem>>, 
        reputation_system: Arc<Mutex<ReputationSystem>>,
        consensus: Arc<Mutex<ProofOfCooperation>>,
    ) -> Self {
        let coordinator_did = "did:icn:genesis".to_string();
        
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: vec![],
            consensus,
            identity_system,
            reputation_system,
            current_block_number: 1,
            coordinator_did,
        }
    }

    pub async fn finalize_block(&mut self) -> Result<(), String> {
        let previous_hash = self.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_default();

        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
            self.coordinator_did.clone(),
        );

        let mut consensus = self.consensus.lock()
            .map_err(|_| "Failed to acquire consensus lock".to_string())?;
        
        consensus.start_round().await?;

        let validators = self.get_active_validators();
        if validators.is_empty() {
            return Err("No active validators available".to_string());
        }

        drop(consensus);

        {
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            consensus.propose_block(&validators[0], new_block.clone()).await?;
        }

        for validator in validators {
            let signature = String::from("dummy_signature");
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            consensus.submit_vote(&validator, true, signature).await?;
        }

        let (finalized_block, reputation_updates) = {
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            let block = consensus.finalize_round().await?;
            let updates = consensus.get_reputation_updates().to_vec();
            (block, updates)
        };

        self.chain.push(finalized_block);
        self.pending_transactions.clear();
        
        {
            let mut reputation_system = self.reputation_system.lock()
                .map_err(|_| "Failed to acquire reputation lock".to_string())?;
            for (did, change) in reputation_updates {
                reputation_system.increase_reputation(&did, change);
            }
        }

        self.current_block_number += 1;
        Ok(())
    }

    fn get_active_validators(&self) -> Vec<String> {
        vec![
            "did:icn:validator1".to_string(),
            "did:icn:validator2".to_string(),
            "did:icn:validator3".to_string(),
        ]
    }

    pub fn get_current_round(&self) -> Option<ConsensusRound> {
        self.consensus.try_lock()
            .ok()
            .and_then(|consensus| consensus.get_current_round())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::WebSocketHandler;

    #[tokio::test]
    async fn test_blockchain_new() {
        let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
        let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());
        
        let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
            ConsensusConfig::default(),
            ws_handler,
        )));

        let blockchain = Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
            consensus,
        );

        assert_eq!(blockchain.current_block_number, 1);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 0);
        assert_eq!(blockchain.coordinator_did, "did:icn:genesis");
    }
}