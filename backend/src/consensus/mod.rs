// src/consensus/mod.rs

pub mod proof_of_cooperation;
pub mod types;

pub use proof_of_cooperation::ProofOfCooperation;
pub use types::{
    ConsensusConfig,
    RoundStatus,
    ConsensusRound,
    ValidatorInfo,
    ConsensusError
};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::blockchain::{Block, transaction::{Transaction, TransactionType}};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::websocket::WebSocketHandler;
use crate::vm::{Contract, VM, ExecutionContext};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub contracts: HashMap<String, Contract>,
    pub consensus: ProofOfCooperation,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
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
            consensus: ProofOfCooperation::new(consensus_config, ws_handler),
            identity_system,
            reputation_system,
            current_block_number: 1,
        }
    }

    pub fn finalize_block(&mut self) -> Result<(), String> {
        self.consensus.start_round()?;

        let previous_hash = self.chain.last().unwrap().hash.clone();
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
                let mut reputation_system = self.reputation_system.lock().unwrap();
                for (ref did, change) in reputation_updates {
                    reputation_system.increase_reputation(did, *change);
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

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        self.verify_transaction(&transaction)?;
        self.pending_transactions.push(transaction);
        
        if self.pending_transactions.len() >= 10 {
            self.finalize_block()?;
        }
        
        Ok(())
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
                let contract = self.get_contract(contract_id)?;
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
                vm.execute_contract(&contract)
            }
        }
    }

    fn get_contract(&self, contract_id: &str) -> Result<&Contract, String> {
        self.contracts
            .get(contract_id)
            .ok_or_else(|| format!("Contract {} not found", contract_id))
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