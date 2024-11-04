pub mod transaction;

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use self::transaction::{Transaction, TransactionType};
use std::collections::HashMap;

use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::vm::{VM, Contract, ExecutionContext};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>, // Vector to hold transactions
    pub hash: String,
}

impl Block {
    /// Creates a new block with a list of transactions and calculates its hash.
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

    /// Calculates a hash for the block based on its contents.
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
    pub difficulty: usize,
    pub identity_system: IdentitySystem,
    pub reputation_system: ReputationSystem,
    pub contracts: HashMap<String, Contract>,
    pub current_block_number: u64,
}

impl Blockchain {
    /// Initializes a new blockchain with a genesis block.
    pub fn new(identity_system: IdentitySystem, reputation_system: ReputationSystem) -> Self {
        let genesis_block = Block::new(0, String::from("0"), vec![]);
        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: vec![],
            difficulty: 2,
            identity_system,
            reputation_system,
            contracts: HashMap::new(),
            current_block_number: 1,
        }
    }

    /// Adds a new transaction to the list of pending transactions.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    /// Finalizes a new block with pending transactions.
    pub fn finalize_block(&mut self) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        self.chain.push(new_block);
        self.pending_transactions.clear();
        self.current_block_number += 1;
    }

    /// Processes a transaction
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<(), String> {
        match &transaction.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                // Handle transfer logic (simplified for this example)
                println!(
                    "Processed transfer of {} from {} to {}",
                    amount, transaction.sender, receiver
                );
                Ok(())
            }
            TransactionType::ContractExecution { contract_id, input_data } => {
                // Fetch the contract
                let contract = self.get_contract(contract_id)?;
                
                // Set up the VM
                let mut vm = VM::new(
                    1000, // Instruction limit
                    self.reputation_system.get_reputation_context(),
                );

                // Create ExecutionContext
                let execution_context = ExecutionContext {
                    caller_did: transaction.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: transaction.timestamp as u64,
                    block_number: self.current_block_number,
                    reputation_score: self.reputation_system.get_reputation(&transaction.sender),
                    permissions: self.identity_system.get_permissions(&transaction.sender),
                };

                vm.set_execution_context(execution_context);

                // Execute the contract
                let result = vm.execute_contract(&contract);

                // Handle result
                match result {
                    Ok(_) => {
                        // Update state based on vm.memory or vm.events
                        self.handle_vm_events(vm.get_events());
                        self.reputation_system.update_reputations(vm.get_reputation_context());
                        Ok(())
                    }
                    Err(e) => Err(format!("Contract execution failed: {}", e)),
                }
            }
            // ... handle other transaction types
        }
    }

    /// Handle VM events emitted during contract execution
    fn handle_vm_events(&mut self, events: &Vec<crate::vm::Event>) {
        for event in events {
            match event.event_type.as_str() {
                "CooperativeCreated" => {
                    // Update blockchain state to include the new cooperative
                    println!("Event: CooperativeCreated - {}", event.data.get("creator").unwrap());
                }
                "ProposalCreated" => {
                    // Add the proposal to the governance module
                    println!("Event: ProposalCreated");
                }
                // Handle other event types...
                _ => {
                    // Log or ignore unknown events
                    println!("Unknown event type: {}", event.event_type);
                }
            }
        }
    }

    /// Get a contract by ID
    fn get_contract(&self, contract_id: &str) -> Result<&Contract, String> {
        self.contracts.get(contract_id).ok_or_else(|| "Contract not found".to_string())
    }
}
