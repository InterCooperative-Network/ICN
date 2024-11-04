// backend/src/consensus/mod.rs

pub mod proof_of_cooperation;
pub use proof_of_cooperation::{
    ProofOfCooperation,
    ConsensusConfig,
    RoundStatus
};
use std::sync::{Arc, Mutex};
use crate::blockchain::{Block, transaction::Transaction};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub consensus: ProofOfCooperation,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub current_block_number: u64,
}

impl Blockchain {
    pub fn new(identity_system: Arc<Mutex<IdentitySystem>>, 
               reputation_system: Arc<Mutex<ReputationSystem>>) -> Self {
        // Create consensus config
        let consensus_config = ConsensusConfig::default();

        Blockchain {
            chain: vec![Block::new(0, String::from("0"), vec![])],
            pending_transactions: vec![],
            consensus: ProofOfCooperation::new(consensus_config),
            identity_system,
            reputation_system,
            current_block_number: 1,
        }
    }

    pub fn finalize_block(&mut self) -> Result<(), String> {
        // Start consensus round
        self.consensus.start_round()?;

        // Create new block with pending transactions
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        // Get the active validators
        let validators = self.get_active_validators();
        if validators.is_empty() {
            return Err("No active validators available".to_string());
        }

        // Propose the block using the first validator as coordinator
        match self.consensus.get_current_round() {
            Some(round) => {
                // Propose the block
                self.consensus.propose_block(&validators[0], new_block.clone())?;

                // Simulate votes from validators
                for validator in validators {
                    self.consensus.submit_vote(&validator, true)?;
                }

                // Finalize the round and get the block
                match self.consensus.finalize_round() {
                    Ok(finalized_block) => {
                        // Add the finalized block to the chain
                        self.chain.push(finalized_block);
                        
                        // Clear pending transactions
                        self.pending_transactions.clear();
                        
                        // Update reputation scores
                        let reputation_updates = self.consensus.get_reputation_updates();
                        let mut reputation_system = self.reputation_system.lock().unwrap();
                        for (ref did, change) in reputation_updates {
                            reputation_system.increase_reputation(did, *change);
                        }

                        self.current_block_number += 1;
                        Ok(())
                    },
                    Err(e) => Err(e)
                }
            },
            None => Err("No active consensus round".to_string())
        }
    }

    fn get_active_validators(&self) -> Vec<String> {
        vec![
            "did:icn:1".to_string(),
            "did:icn:2".to_string(),
            "did:icn:3".to_string(),
        ]
    }
}