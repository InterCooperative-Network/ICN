# File: ./backend/src/consensus/mod.rs
mod proof_of_cooperation;
mod types;

pub use proof_of_cooperation::ProofOfCooperation;
pub use types::{ConsensusRound, RoundStatus, Validator};

# File: ./backend/src/consensus/types.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub did: String,
    pub reputation: i64,
    pub last_block_proposed: u64,
    pub consecutive_missed_validations: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,
    pub proposed_block: Option<crate::blockchain::Block>,
    pub votes: HashMap<String, bool>,
    pub status: RoundStatus,
    pub start_time: u64,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Proposing,
    Voting,
    Finalizing,
    Completed,
    Failed,
}

# File: ./backend/src/consensus/proof_of_cooperation.rs
// Copy the entire ProofOfCooperation implementation from the previous artifact here

# File: ./backend/src/blockchain/mod.rs
// Add this to the existing mod.rs
use crate::consensus::ProofOfCooperation;

pub struct Blockchain {
    // Add these fields to the existing Blockchain struct
    pub consensus: ProofOfCooperation,
    pub current_block_number: u64,
    // ... existing fields ...
}

impl Blockchain {
    pub fn new(identity_system: IdentitySystem, reputation_system: ReputationSystem) -> Self {
        Blockchain {
            chain: vec![Block::new(0, String::from("0"), vec![])],
            pending_transactions: vec![],
            difficulty: 2,
            identity_system,
            reputation_system,
            contracts: HashMap::new(),
            current_block_number: 1,
            // Initialize consensus with desired parameters
            consensus: ProofOfCooperation::new(
                3,      // min_validators
                0.66,   // vote_threshold (66%)
                60,     // round_timeout in seconds
                50,     // min_reputation
            ),
        }
    }

    // Add this method to handle block finalization through consensus
    pub fn finalize_block(&mut self) -> Result<(), String> {
        // Start a new consensus round
        self.consensus.start_round()?;

        // Create a new block with pending transactions
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        // Get the current round status and coordinator
        if let Some(round) = &self.consensus.current_round {
            // Propose the block (only coordinator can do this)
            self.consensus.propose_block(&round.coordinator, new_block)?;

            // In a real implementation, we would wait for votes here
            // For now, we'll assume the block is accepted

            // Finalize the round and get the accepted block
            let finalized_block = self.consensus.finalize_round()?;
            
            // Add the block to the chain
            self.chain.push(finalized_block);
            
            // Clear pending transactions
            self.pending_transactions.clear();
            
            // Update reputation based on consensus round
            let reputation_updates = self.consensus.get_reputation_updates();
            for (did, change) in reputation_updates {
                self.reputation_system.update_reputation(did, *change);
            }

            self.current_block_number += 1;
            Ok(())
        } else {
            Err("No active consensus round".to_string())
        }
    }
}