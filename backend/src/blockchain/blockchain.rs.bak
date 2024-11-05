use super::transaction::{Transaction, TransactionType};
use super::block::Block;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::vm::{VM, ExecutionContext};
use crate::vm::contract::Contract;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub contracts: HashMap<String, Contract>,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub current_block_number: u64,
}

impl Blockchain {
    pub fn new(
        identity_system: Arc<Mutex<IdentitySystem>>,
        reputation_system: Arc<Mutex<ReputationSystem>>,
    ) -> Self {
        Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            contracts: HashMap::new(),
            identity_system,
            reputation_system,
            current_block_number: 0,
        }
    }

    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<(), String> {
        match &transaction.transaction_type {
            TransactionType::Transfer { amount, receiver } => {
                // Handle standard transfer transaction
                self.validate_transaction(transaction)?;
                // Update balances, etc.
                self.pending_transactions.push(transaction.clone());
                Ok(())
            }
            TransactionType::ContractExecution { contract_id, input_data } => {
                // Fetch the contract
                let contract = self.get_contract(contract_id)?;
                // Set up the VM
                let reputation_context = self.reputation_system.lock().unwrap().get_reputation_context();
                let mut vm = VM::new(1000, reputation_context);

                // Create ExecutionContext
                let identity_system = self.identity_system.lock().unwrap();
                let permissions = identity_system.get_permissions(&transaction.sender);
                let reputation_score = self.reputation_system.lock().unwrap().get_reputation(&transaction.sender);

                let execution_context = ExecutionContext {
                    caller_did: transaction.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: transaction.timestamp,
                    block_number: self.current_block_number,
                    reputation_score,
                    permissions,
                };

                vm.set_execution_context(execution_context);

                // Execute the contract
                let result = vm.execute_contract(&contract);

                // Handle result
                match result {
                    Ok(_) => {
                        // Update state based on vm.memory or vm.events
                        self.handle_vm_events(vm.get_events());
                        self.reputation_system.lock().unwrap().update_reputations(vm.reputation_context);
                        self.pending_transactions.push(transaction.clone());
                        Ok(())
                    }
                    Err(e) => Err(format!("Contract execution failed: {}", e)),
                }
            }
            // Handle other transaction types...
        }
    }

    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // Validate transaction signature, balances, etc.
        Ok(())
    }

    fn get_contract(&self, contract_id: &str) -> Result<Contract, String> {
        self.contracts.get(contract_id).cloned().ok_or(format!("Contract {} not found", contract_id))
    }

    fn handle_vm_events(&mut self, events: &Vec<crate::vm::event::Event>) {
        for event in events {
            match event.event_type.as_str() {
                "CooperativeCreated" => {
                    // Update blockchain state to include the new cooperative
                    println!("Cooperative created: {:?}", event.data);
                }
                "ProposalCreated" => {
                    // Add the proposal to the governance module
                    println!("Proposal created: {:?}", event.data);
                }
                // Handle other event types...
                _ => {
                    // Log or ignore unknown events
                    println!("Unknown event: {:?}", event);
                }
            }
        }
    }

    pub fn create_contract(&mut self, contract: Contract) {
        self.contracts.insert(contract.id.clone(), contract);
    }
}
