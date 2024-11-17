// src/blockchain/chain.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::consensus::{ProofOfCooperation, ConsensusRound};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::vm::{VM, Contract, ExecutionContext, Event};
use crate::blockchain::{Block, Transaction};
use crate::blockchain::transaction::{TransactionType, ResourceAllocation};
use crate::relationship::{RelationshipSystem, Contribution, MutualAidInteraction, Relationship};
use crate::vm::operations::relationship::RelationType;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub consensus: Arc<Mutex<ProofOfCooperation>>,
    pub identity_system: Arc<Mutex<IdentitySystem>>,
    pub reputation_system: Arc<Mutex<ReputationSystem>>,
    pub relationship_system: Arc<Mutex<RelationshipSystem>>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    pub current_block_number: u64,
    coordinator_did: String,
}

impl Blockchain {
    pub fn new(
        identity_system: Arc<Mutex<IdentitySystem>>, 
        reputation_system: Arc<Mutex<ReputationSystem>>,
        relationship_system: Arc<Mutex<RelationshipSystem>>,
        consensus: Arc<Mutex<ProofOfCooperation>>,
    ) -> Self {
        let coordinator_did = "did:icn:genesis".to_string();
        
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: vec![],
            consensus,
            identity_system,
            reputation_system,
            relationship_system,
            resource_allocations: HashMap::new(),
            current_block_number: 1,
            coordinator_did,
        }
    }

    pub async fn process_transaction(&mut self, transaction: &Transaction) -> Result<(), String> {
        self.validate_transaction(transaction)?;
        self.check_and_update_resources(&transaction.sender)?;

        let tx = transaction.clone();
        let sender = tx.sender.clone();

        {
            let resource_allocation = self.resource_allocations.get_mut(&sender)
                .ok_or("Resource allocation not found")?;
            
            if !resource_allocation.can_afford(tx.resource_cost) {
                return Err("Insufficient resources".to_string());
            }

            resource_allocation.consume_resources(tx.resource_cost)?;
        }

        match &tx.transaction_type {
            TransactionType::Transfer { amount, receiver } => {
                println!(
                    "Processing transfer of {} from {} to {}",
                    amount, tx.sender, receiver
                );
                self.pending_transactions.push(tx);
                Ok(())
            },

            TransactionType::ContractExecution { contract_id, input_data: _ } => {
                let contract = self.get_contract(contract_id)?;
                let reputation_context = {
                    let reputation_system = self.reputation_system.lock()
                        .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                    reputation_system.get_reputation_context()
                };

                let (permissions, reputation_score) = {
                    let identity_system = self.identity_system.lock()
                        .map_err(|_| "Failed to acquire identity system lock".to_string())?;
                    let perms = identity_system.get_permissions(&tx.sender);
                    
                    let reputation_system = self.reputation_system.lock()
                        .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                    let score = reputation_system.get_reputation(&tx.sender);
                    
                    (perms, score)
                };

                let mut vm = VM::new(1000, reputation_context);
                let execution_context = ExecutionContext {
                    caller_did: tx.sender.clone(),
                    cooperative_id: contract.cooperative_metadata.cooperative_id.clone(),
                    timestamp: tx.timestamp as u64,
                    block_number: self.current_block_number,
                    reputation_score,
                    permissions,
                };

                vm.set_execution_context(execution_context);

                match vm.execute_contract(&contract) {
                    Ok(_) => {
                        self.handle_vm_events(vm.get_events());
                        let mut reputation_system = self.reputation_system.lock()
                            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
                        reputation_system.update_reputations(&vm.get_reputation_context());
                        self.pending_transactions.push(tx);
                        Ok(())
                    }
                    Err(e) => Err(format!("Contract execution failed: {}", e)),
                }
            },

            TransactionType::RecordContribution { description, impact_story, context, tags } => {
                let mut relationship_system = self.relationship_system.lock()
                    .map_err(|_| "Failed to acquire relationship lock".to_string())?;
                
                let contribution = Contribution {
                    contributor_did: tx.sender.clone(),
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    date: chrono::Utc::now(),
                    context: context.clone(),
                    witnesses: vec![],
                    feedback: vec![],
                    tags: tags.clone(),
                };
                
                relationship_system.record_contribution(contribution)?;
                self.pending_transactions.push(tx);
                Ok(())
            },

            TransactionType::RecordMutualAid { receiver, description, impact_story, reciprocity_notes, tags } => {
                let mut relationship_system = self.relationship_system.lock()
                    .map_err(|_| "Failed to acquire relationship lock".to_string())?;
                
                let interaction = MutualAidInteraction {
                    date: chrono::Utc::now(),
                    provider_did: tx.sender.clone(),
                    receiver_did: receiver.clone(),
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    reciprocity_notes: reciprocity_notes.clone(),
                    tags: tags.clone(),
                };
                
                relationship_system.record_mutual_aid(interaction)?;
                self.pending_transactions.push(tx);
                Ok(())
            },

            TransactionType::UpdateRelationship { member_two, relationship_type, story, interaction } => {
                let mut relationship_system = self.relationship_system.lock()
                    .map_err(|_| "Failed to acquire relationship lock".to_string())?;
                
                let relationship = Relationship {
                    member_one: tx.sender.clone(),
                    member_two: member_two.clone(),
                    relationship_type: RelationType::Custom(relationship_type.clone()),
                    started: chrono::Utc::now(),
                    story: story.clone(),
                    interactions: vec![],
                    mutual_endorsements: vec![],
                    notes: vec![],
                };
                
                if let Some(interaction_data) = interaction {
                    let _ = interaction_data;
                }
                
                relationship_system.update_relationship(relationship)?;
                self.pending_transactions.push(tx);
                Ok(())
            },

            TransactionType::AddEndorsement { to_did, content, context, skills } => {
                let mut relationship_system = self.relationship_system.lock()
                    .map_err(|_| "Failed to acquire relationship lock".to_string())?;
                
                let endorsement = crate::relationship::Endorsement {
                    from_did: tx.sender.clone(),
                    content: content.clone(),
                    date: chrono::Utc::now(),
                    context: context.clone(),
                    skills: skills.clone(),
                };
                
                relationship_system.add_endorsement(&tx.sender, to_did, endorsement)?;
                self.pending_transactions.push(tx);
                Ok(())
            },
        }
    }

    fn check_and_update_resources(&mut self, did: &str) -> Result<(), String> {
        let reputation_score = {
            let reputation_system = self.reputation_system.lock()
                .map_err(|_| "Failed to acquire reputation lock".to_string())?;
            reputation_system.get_reputation(did)
        };

        let resource_allocation = self.resource_allocations
            .entry(did.to_string())
            .or_insert_with(|| ResourceAllocation::new(reputation_score));

        resource_allocation.update_resources();
        Ok(())
    }

    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        let identity_valid = {
            let identity_system = self.identity_system.lock()
                .map_err(|_| "Failed to acquire identity system lock".to_string())?;
            identity_system.is_registered(&transaction.sender)
        };

        if !identity_valid {
            return Err("Invalid sender DID".to_string());
        }

        let reputation_valid = {
            let reputation_system = self.reputation_system.lock()
                .map_err(|_| "Failed to acquire reputation lock".to_string())?;
            reputation_system.get_reputation(&transaction.sender) >= 10
        };

        if !reputation_valid {
            return Err("Insufficient reputation".to_string());
        }

        Ok(())
    }

    fn get_contract(&self, _contract_id: &str) -> Result<Contract, String> {
        Err("Contract not found".to_string())
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
        self.process_transaction(&transaction).await?;
        
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
            self.coordinator_did.clone(),
        );

        let mut consensus_guard = self.consensus.lock()
            .map_err(|_| "Failed to acquire consensus lock".to_string())?;

        consensus_guard.start_round().await?;
        
        let validators = self.get_active_validators();
        if validators.is_empty() {
            return Err("No active validators available".to_string());
        }

        self.coordinator_did = validators[0].clone();

        consensus_guard.propose_block(&self.coordinator_did, new_block.clone()).await?;

        for validator in &validators {
            let signature = String::from("dummy_signature");
            consensus_guard.submit_vote(validator, true, signature).await?;
        }

        let block = consensus_guard.finalize_round().await?;
        let updates = consensus_guard.get_reputation_updates();

        let mut reputation_system = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
            
        for (did, change) in updates {
            reputation_system.increase_reputation(&did, *change);
        }

        self.chain.push(block);
        self.pending_transactions.clear();
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

    pub fn get_current_round(&self) -> Option<ConsensusRound> {
        self.consensus.try_lock()
            .ok()
            .and_then(|consensus| consensus.get_current_round().cloned())
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
        let relationship_system = Arc::new(Mutex::new(RelationshipSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());
        
        let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
            crate::consensus::types::ConsensusConfig::default(),
            ws_handler,
        )));

        let blockchain = Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
            relationship_system.clone(),
            consensus,
        );

        assert_eq!(blockchain.current_block_number, 1);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 0);
        assert_eq!(blockchain.coordinator_did, "did:icn:genesis");
    }
}