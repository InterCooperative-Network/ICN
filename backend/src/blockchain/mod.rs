// src/blockchain/mod.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::consensus::*;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::websocket::WebSocketHandler;
use crate::vm::{VM, Contract, ExecutionContext};

pub mod transaction;
use transaction::{Transaction, TransactionType};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        use sha2::{Sha256, Digest};
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let mut hasher = Sha256::new();
        let transaction_data = serde_json::to_string(&transactions).unwrap_or_default();
        hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, transaction_data));
        let hash = format!("{:x}", hasher.finalize());

        Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash,
        }
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
    pub fn new(
        identity_system: Arc<Mutex<IdentitySystem>>,
        reputation_system: Arc<Mutex<ReputationSystem>>,
        ws_handler: Arc<WebSocketHandler>
    ) -> Self {
        let consensus_config = ConsensusConfig::default();
        
        Blockchain {
            chain: vec![Block::new(0, String::from("0"), vec![])],
            pending_transactions: vec![],
            contracts: HashMap::new(),
            identity_system,
            reputation_system,
            consensus: ProofOfCooperation::new(consensus_config, ws_handler),
            current_block_number: 1,
        }
    }

    fn verify_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        let identity_system = self.identity_system.lock()
            .map_err(|_| "Failed to acquire identity system lock".to_string())?;
        
        if !identity_system.is_registered(&transaction.sender) {
            return Err("Invalid sender DID".to_string());
        }

        let reputation_system = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation system lock".to_string())?;
        
        let sender_reputation = reputation_system.get_reputation(&transaction.sender);
        if sender_reputation < 10 {
            return Err("Insufficient reputation".to_string());
        }

        match &transaction.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                println!(
                    "Processing transfer of {} from {} to {}",
                    amount, transaction.sender, receiver
                );
                Ok(())
            }
            TransactionType::ContractExecution { contract_id, input_data: _ } => {
                let contract = self.contracts.get(contract_id)
                    .ok_or_else(|| format!("Contract {} not found", contract_id))?;

                let permissions = identity_system.get_permissions(&transaction.sender);
                let execution_context = ExecutionContext {
                    caller_did: transaction.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: transaction.timestamp as u64,
                    block_number: self.current_block_number,
                    reputation_score: sender_reputation,
                    permissions,
                };

                let mut vm = VM::new(1000, reputation_system.get_reputation_context());
                vm.set_execution_context(execution_context);
                vm.execute_contract(contract)
            }
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        self.verify_transaction(&transaction)?;
        self.pending_transactions.push(transaction);
        
        if self.pending_transactions.len() >= 10 {
            self.finalize_block()?;
        }
        
        Ok(())
    }

    pub fn finalize_block(&mut self) -> Result<(), String> {
        self.consensus.start_round()?;

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

        self.consensus.propose_block(&validators[0], new_block.clone())?;

        for validator in validators {
            self.consensus.submit_vote(&validator, true)?;
        }

        match self.consensus.finalize_round() {
            Ok(finalized_block) => {
                self.chain.push(finalized_block);
                self.pending_transactions.clear();
                
                let reputation_updates = self.consensus.get_reputation_updates();
                if let Ok(mut reputation_system) = self.reputation_system.lock() {
                    for (did, change) in reputation_updates {
                        reputation_system.increase_reputation(did, *change);
                    }
                }

                self.current_block_number += 1;
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    }

    fn get_active_validators(&self) -> Vec<String> {
        vec![
            "did:icn:1".to_string(),
            "did:icn:2".to_string(),
            "did:icn:3".to_string(),
        ]
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