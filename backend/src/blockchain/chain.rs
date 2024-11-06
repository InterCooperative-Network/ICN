// src/blockchain/chain.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::consensus::ProofOfCooperation;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::vm::event::Event;
use super::{Block, Transaction, TransactionType};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub contracts: HashMap<String, Contract>,
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
            contracts: HashMap::new(),
            consensus,
            identity_system,
            reputation_system,
            current_block_number: 1,
        }
    }

    pub fn create_contract(&mut self, contract: Contract) {
        self.contracts.insert(contract.id.clone(), contract);
    }

    pub async fn process_transaction(&mut self, transaction: &Transaction) -> Result<(), String> {
        match &transaction.transaction_type {
            TransactionType::Transfer { amount, receiver } => {
                self.validate_transaction(transaction)?;
                println!(
                    "Processing transfer of {} from {} to {}",
                    amount, transaction.sender, receiver
                );
                self.pending_transactions.push(transaction.clone());
                Ok(())
            }
            TransactionType::ContractExecution { contract_id, input_data: _ } => {
                // Get contract first to avoid holding locks during VM execution
                let contract = self.get_contract(contract_id)?;
                
                // Get required data under separate lock scopes
                let reputation_context = {
                    let reputation_system = self.reputation_system.lock()
                        .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                    reputation_system.get_reputation_context()
                };
                
                // Get permissions and reputation score
                let (permissions, reputation_score) = {
                    let identity_system = self.identity_system.lock()
                        .map_err(|_| "Failed to acquire identity lock".to_string())?;
                    let perms = identity_system.get_permissions(&transaction.sender);
                    
                    let reputation_system = self.reputation_system.lock()
                        .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                    let score = reputation_system.get_reputation(&transaction.sender);
                    
                    (perms, score)
                };

                // Execute contract
                let mut vm = VM::new(1000, reputation_context);
                let execution_context = ExecutionContext {
                    caller_did: transaction.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: transaction.timestamp as u64,
                    block_number: self.current_block_number,
                    reputation_score,
                    permissions,
                };

                vm.set_execution_context(execution_context);

                match vm.execute_contract(contract) {
                    Ok(_) => {
                        // Process VM events
                        self.handle_vm_events(&vm.get_events());
                        
                        // Update reputations
                        {
                            let mut reputation_system = self.reputation_system.lock()
                                .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                            reputation_system.update_reputations(&vm.get_reputation_context());
                        }
                        
                        self.pending_transactions.push(transaction.clone());
                        Ok(())
                    }
                    Err(e) => Err(format!("Contract execution failed: {}", e)),
                }
            }
        }
    }

    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // Check DID validity
        let identity_valid = {
            let identity_system = self.identity_system.lock()
                .map_err(|_| "Failed to acquire identity system lock".to_string())?;
            identity_system.is_registered(&transaction.sender)
        };

        if !identity_valid {
            return Err("Invalid sender DID".to_string());
        }

        // Check reputation
        let reputation_valid = {
            let reputation_system = self.reputation_system.lock()
                .map_err(|_| "Failed to acquire reputation system lock".to_string())?;
            reputation_system.get_reputation(&transaction.sender) >= 10
        };

        if !reputation_valid {
            return Err("Insufficient reputation".to_string());
        }

        Ok(())
    }

    fn get_contract(&self, contract_id: &str) -> Result<&Contract, String> {
        self.contracts.get(contract_id)
            .ok_or_else(|| format!("Contract {} not found", contract_id))
    }

    fn handle_vm_events(&self, events: &[Event]) {
        for event in events {
            match event.event_type.as_str() {
                "CooperativeCreated" => {
                    println!("Cooperative created: {:?}", event.data);
                }
                "ProposalCreated" => {
                    println!("Proposal created: {:?}", event.data);
                }
                _ => {
                    println!("Unknown event type: {}", event.event_type);
                }
            }
        }
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        self.validate_transaction(&transaction)?;
        self.pending_transactions.push(transaction.clone());
        
        if self.pending_transactions.len() >= 10 {
            self.finalize_block().await?;
        }
        
        Ok(())
    }

    pub async fn finalize_block(&mut self) -> Result<(), String> {
        let previous_hash = self.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_default();

        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        // Start consensus round
        let consensus_guard = self.consensus.lock()
            .map_err(|_| "Failed to acquire consensus lock".to_string())?;
        let mut consensus = consensus_guard;
        
        consensus.start_round().await?;
        
        // Get validators
        let validators = self.get_active_validators();
        if validators.is_empty() {
            return Err("No active validators available".to_string());
        }

        // Propose block
        consensus.propose_block(&validators[0], new_block.clone()).await?;

        // Collect votes
        for validator in &validators {
            let signature = String::from("dummy_signature");
            consensus.submit_vote(validator, true, signature).await?;
        }

        // Finalize round
        let block = consensus.finalize_round().await?;
        let updates = consensus.get_reputation_updates().to_vec();
        
        // Drop consensus lock before updating other state
        drop(consensus);

        // Update chain
        self.chain.push(block);
        self.pending_transactions.clear();
        
        // Apply reputation updates
        let mut reputation_system = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
        for (did, change) in updates {
            reputation_system.increase_reputation(&did, change);
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
    use crate::websocket::WebSocketHandler;
    use crate::identity::DID;

    fn setup_test_blockchain() -> Blockchain {
        let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
        let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());
        
        let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
            ConsensusConfig::default(),
            ws_handler,
        )));

        Blockchain::new(
            identity_system,
            reputation_system,
            consensus,
        )
    }

    #[tokio::test]
    async fn test_add_transaction() {
        let mut blockchain = setup_test_blockchain();
        let transaction = Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        );

        // Register the test DID
        {
            let mut identity = blockchain.identity_system.lock().unwrap();
            identity.register_did(
                DID {
                    id: "did:icn:test".to_string(),
                    public_key: vec![],
                },
                vec!["transfer".to_string()],
            );
        }

        // Set initial reputation
        {
            let mut reputation = blockchain.reputation_system.lock().unwrap();
            reputation.increase_reputation("did:icn:test", 100);
        }

        assert!(blockchain.add_transaction(transaction).await.is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }
}