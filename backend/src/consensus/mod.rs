// src/consensus/mod.rs

pub mod proof_of_cooperation;
pub mod types;

pub use proof_of_cooperation::ProofOfCooperation;
pub use types::{ConsensusConfig, RoundStatus, ConsensusRound, ValidatorInfo};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::blockchain::{Block, Transaction};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::websocket::WebSocketHandler;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub consensus: Arc<Mutex<ProofOfCooperation>>,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub current_block_number: u64,
}

impl Blockchain {
    pub fn new(
        identity_system: Arc<Mutex<IdentitySystem>>, 
        reputation_system: Arc<Mutex<ReputationSystem>>,
        consensus: Arc<Mutex<ProofOfCooperation>>,
    ) -> Self {
        Blockchain {
            chain: vec![Block::new(0, String::from("0"), vec![])],
            pending_transactions: vec![],
            consensus,
            identity_system,
            reputation_system,
            current_block_number: 1,
        }
    }

    pub async fn finalize_block(&mut self) -> Result<(), String> {
        // Get consensus lock first
        let mut consensus = self.consensus.lock()
            .map_err(|_| "Failed to acquire consensus lock".to_string())?;

        // Start consensus round
        consensus.start_round().await?;

        let previous_hash = self.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_default();

        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        let validators = self.get_active_validators();
        if validators.is_empty() {
            return Err("No active validators available".to_string());
        }

        // Drop current consensus lock before proposing block
        drop(consensus);

        // Propose block with new lock
        {
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            consensus.propose_block(&validators[0], new_block.clone()).await?;
        }

        // Submit votes
        for validator in validators {
            let signature = String::from("dummy_signature");
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            consensus.submit_vote(&validator, true, signature).await?;
        }

        // Finalize round
        let (finalized_block, reputation_updates) = {
            let mut consensus = self.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            let block = consensus.finalize_round().await?;
            let updates = consensus.get_reputation_updates().to_vec();
            (block, updates)
        };

        // Update state
        self.chain.push(finalized_block);
        self.pending_transactions.clear();
        
        // Apply reputation updates with proper dereferencing
        {
            let mut reputation_system = self.reputation_system.lock()
                .map_err(|_| "Failed to acquire reputation lock".to_string())?;
            for (did, change) in reputation_updates {
                reputation_system.increase_reputation(&did, change); // No longer passing a reference
            }
        }

        self.current_block_number += 1;
        Ok(())
    }

    fn get_active_validators(&self) -> Vec<String> {
        vec![
            "did:icn:1".to_string(),
            "did:icn:2".to_string(),
            "did:icn:3".to_string(),
        ]
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        self.pending_transactions.push(transaction);
        if self.pending_transactions.len() >= 10 {
            self.finalize_block().await?;
        }
        Ok(())
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn get_transaction_count(&self) -> usize {
        self.chain.iter().map(|block| block.transactions.len()).sum()
    }

    pub fn get_block_count(&self) -> usize {
        self.chain.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            identity_system,
            reputation_system,
            consensus,
        );

        assert_eq!(blockchain.current_block_number, 1);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 0);
    }
}