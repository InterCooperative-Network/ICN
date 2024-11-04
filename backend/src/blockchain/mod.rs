// backend/src/blockchain/mod.rs

pub mod transaction;
use transaction::{Transaction, TransactionType};

use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::collections::HashMap;

use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::consensus::{ProofOfCooperation, ConsensusConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = Self::calculate_hash(index, &previous_hash, timestamp, &transactions);

        Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash,
        }
    }

    fn calculate_hash(index: u64, previous_hash: &str, timestamp: u128, transactions: &Vec<Transaction>) -> String {
        let mut hasher = Sha256::new();
        let transaction_data = serde_json::to_string(transactions).expect("Failed to serialize transactions");
        hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, transaction_data));
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub contracts: HashMap<String, Contract>,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub consensus: ProofOfCooperation,
    pub current_block_number: u64,
}

impl Blockchain {
    pub fn new(identity_system: Arc<Mutex<IdentitySystem>>, 
               reputation_system: Arc<Mutex<ReputationSystem>>) -> Self {
        let consensus_config = ConsensusConfig::default();

        Blockchain {
            chain: vec![Block::new(0, String::from("0"), vec![])],
            pending_transactions: vec![],
            contracts: HashMap::new(),
            identity_system,
            reputation_system,
            consensus: ProofOfCooperation::new(consensus_config),
            current_block_number: 1,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // Verify transaction
        self.verify_transaction(&transaction)?;
        
        // Add to pending transactions
        self.pending_transactions.push(transaction);
        
        // If we have enough transactions, try to finalize a block
        if self.pending_transactions.len() >= 10 {
            self.finalize_block()?;
        }
        
        Ok(())
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
                
                // Update reputation scores based on consensus participation
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
    }

    fn verify_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // Verify sender's DID
        let identity_system = self.identity_system.lock().unwrap();
        if !identity_system.is_registered(&transaction.sender) {
            return Err("Invalid sender DID".to_string());
        }

        // Verify sender has sufficient reputation for the transaction
        let reputation_system = self.reputation_system.lock().unwrap();
        let sender_reputation = reputation_system.get_reputation(&transaction.sender);
        
        // Require minimum reputation for transactions
        if sender_reputation < 10 {
            return Err("Insufficient reputation".to_string());
        }

        // Process transaction based on type
        match &transaction.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                // Handle transfer logic
                println!(
                    "Processing transfer of {} from {} to {}",
                    amount, transaction.sender, receiver
                );
                Ok(())
            }
            TransactionType::ContractExecution { contract_id, input_data: _ } => {
                // Fetch and validate the contract
                let contract = self.get_contract(contract_id)?;
                
                // Set up the VM
                let mut vm = VM::new(
                    1000, // Instruction limit
                    reputation_system.get_reputation_context(),
                );

                // Create ExecutionContext
                let execution_context = ExecutionContext {
                    caller_did: transaction.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: transaction.timestamp as u64,
                    block_number: self.current_block_number,
                    reputation_score: sender_reputation,
                    permissions: identity_system.get_permissions(&transaction.sender),
                };

                vm.set_execution_context(execution_context);
                
                // Execute the contract
                vm.execute_contract(&contract)
            }
        }
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    fn get_contract(&self, contract_id: &str) -> Result<&Contract, String> {
        self.contracts
            .get(contract_id)
            .ok_or_else(|| format!("Contract {} not found", contract_id))
    }

    pub fn deploy_contract(&mut self, contract: Contract) -> Result<(), String> {
        if self.contracts.contains_key(&contract.id) {
            return Err("Contract ID already exists".to_string());
        }
        self.contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    fn get_active_validators(&self) -> Vec<String> {
        vec![
            "did:icn:1".to_string(),
            "did:icn:2".to_string(),
            "did:icn:3".to_string(),
        ]
    }

    pub fn get_transaction_count(&self) -> usize {
        self.chain.iter().map(|block| block.transactions.len()).sum()
    }

    pub fn get_block_count(&self) -> usize {
        self.chain.len()
    }
}