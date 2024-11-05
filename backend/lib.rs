// src/lib.rs

pub mod blockchain;
pub mod identity;
pub mod reputation;
pub mod governance;
pub mod utils;
pub mod vm;
pub mod websocket;
pub mod consensus;
pub mod network;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use chrono;
use uuid;

use crate::blockchain::{Block, Blockchain, Transaction, TransactionType};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::governance::Proposal;
use crate::consensus::types::ConsensusRound as ConsensusRoundType;
use crate::websocket::WebSocketHandler;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::vm::opcode::OpCode;
use crate::vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact};
use crate::consensus::ProofOfCooperation;
use crate::consensus::types::ConsensusConfig;

// Core system integration
pub struct ICNCore {
    blockchain: Arc<Mutex<Blockchain>>,
    identity_system: Arc<Mutex<IdentitySystem>>,
    reputation_system: Arc<Mutex<ReputationSystem>>,
    ws_handler: Arc<WebSocketHandler>,
    vm: VM,
    event_bus: broadcast::Sender<SystemEvent>,
}

// System-wide events
#[derive(Clone, Debug)]
pub enum SystemEvent {
    BlockCreated(Block),
    ProposalSubmitted(Proposal),
    VoteCast { proposal_id: u64, voter: String, vote: bool },
    ReputationChanged { did: String, change: i64, reason: String },
    ConsensusStarted(ConsensusRoundType),
    ConsensusFinished(Block),
    CooperativeCreated { id: String, creator: String },
    CooperativeJoined { id: String, member: String },
}

impl ICNCore {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
        let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());

        // Create ProofOfCooperation instance first
        let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
            ConsensusConfig::default(),
            ws_handler.clone(),
        )));
        
        // Create blockchain with proper consensus instance
        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
            consensus, // Pass the consensus Arc<Mutex<>>
        )));

        let vm = VM::new(1000, reputation_system.lock().unwrap().get_reputation_context());

        ICNCore {
            blockchain,
            identity_system,
            reputation_system,
            ws_handler,
            vm,
            event_bus: event_tx,
        }
    }

    pub async fn create_cooperative(&self, creator_did: String, metadata: CooperativeMetadata) -> Result<String, String> {
        // Verify creator's identity
        let identity = self.identity_system.lock().unwrap();
        if !identity.is_registered(&creator_did) {
            return Err("Creator DID not registered".to_string());
        }

        // Check reputation requirements
        let reputation = self.reputation_system.lock().unwrap();
        if reputation.get_reputation(&creator_did) < 100 {
            return Err("Insufficient reputation to create cooperative".to_string());
        }

        // Create cooperative contract
        let contract = Contract {
            id: uuid::Uuid::new_v4().to_string(),
            code: vec![OpCode::CreateCooperative],
            state: HashMap::new(),
            required_reputation: 100,
            cooperative_metadata: metadata.clone(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec!["cooperative.create".to_string()],
        };

        // Execute contract in VM
        let execution_context = ExecutionContext {
            caller_did: creator_did.clone(),
            cooperative_id: contract.id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            block_number: self.blockchain.lock().unwrap().current_block_number,
            reputation_score: reputation.get_reputation(&creator_did),
            permissions: vec!["cooperative.create".to_string()],
        };

        self.vm.set_execution_context(execution_context);
        self.vm.execute_contract(&contract)?;

        // Create and submit transaction
        let transaction = Transaction::new(
            creator_did.clone(),
            TransactionType::ContractExecution {
                contract_id: contract.id.clone(),
                input_data: HashMap::new(),
            },
        );

        // Add transaction to blockchain with proper await
        {
            let mut blockchain = self.blockchain.lock().unwrap();
            blockchain.add_transaction(transaction).await?;
        }

        // Broadcast event
        let event = SystemEvent::CooperativeCreated {
            id: contract.id.clone(),
            creator: creator_did,
        };
        let _ = self.event_bus.send(event);

        Ok(contract.id)
    }

    pub async fn submit_proposal(&self, creator_did: String, proposal: Proposal) -> Result<u64, String> {
        // Check reputation requirement with fixed threshold
        let reputation = self.reputation_system.lock().unwrap().get_reputation(&creator_did);
        if reputation < 50 {
            return Err("Insufficient reputation to create proposal".to_string());
        }

        // Create proposal transaction
        let transaction = Transaction::new(
            creator_did,
            TransactionType::ContractExecution {
                contract_id: "governance".to_string(),
                input_data: HashMap::new(),
            },
        );

        // Add to blockchain with proper await
        {
            let mut blockchain = self.blockchain.lock().unwrap();
            blockchain.add_transaction(transaction).await?;
        }

        // Broadcast event
        let event = SystemEvent::ProposalSubmitted(proposal.clone());
        let _ = self.event_bus.send(event);

        Ok(proposal.id)
    }

    pub async fn start_consensus_round(&self) -> Result<(), String> {
        let blockchain = self.blockchain.lock().unwrap();
        
        // Access consensus through mutex
        {
            let mut consensus = blockchain.consensus.lock()
                .map_err(|_| "Failed to acquire consensus lock".to_string())?;
            
            consensus.start_round().await?;

            // Get current round if available
            if let Some(round) = consensus.get_current_round() {
                let round_type = ConsensusRoundType::from(round.clone());
                drop(consensus); // Drop consensus lock before event broadcast
                let event = SystemEvent::ConsensusStarted(round_type);
                let _ = self.event_bus.send(event);
            }
        }

        Ok(())
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_bus.subscribe()
    }
}

// Implement Display for SystemEvent for better logging
impl std::fmt::Display for SystemEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemEvent::BlockCreated(block) => write!(f, "Block created: {}", block.index),
            SystemEvent::ProposalSubmitted(proposal) => write!(f, "Proposal submitted: {}", proposal.id),
            SystemEvent::VoteCast { proposal_id, voter, vote } => 
                write!(f, "Vote cast on proposal {}: {} voted {}", proposal_id, voter, vote),
            SystemEvent::ReputationChanged { did, change, reason } => 
                write!(f, "Reputation changed for {}: {} ({})", did, change, reason),
            SystemEvent::ConsensusStarted(round) => 
                write!(f, "Consensus round {} started", round.round_number),
            SystemEvent::ConsensusFinished(block) => 
                write!(f, "Consensus finished with block {}", block.index),
            SystemEvent::CooperativeCreated { id, creator } => 
                write!(f, "Cooperative {} created by {}", id, creator),
            SystemEvent::CooperativeJoined { id, member } => 
                write!(f, "Member {} joined cooperative {}", member, id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_core_initialization() {
        let core = ICNCore::new();
        assert!(core.subscribe_to_events().receiver_count() > 0);
    }

    #[tokio::test]
    async fn test_submit_proposal() {
        let core = ICNCore::new();
        
        // Register test DID
        {
            let mut identity = core.identity_system.lock().unwrap();
            identity.register_did(
                crate::identity::DID {
                    id: "did:icn:test".to_string(),
                    public_key: Vec::new(),
                },
                vec!["proposal.create".to_string()],
            );
        }

        // Set initial reputation
        {
            let mut reputation = core.reputation_system.lock().unwrap();
            reputation.increase_reputation("did:icn:test", 100);
        }

        let proposal = Proposal {
            id: 1,
            proposal_type: crate::governance::ProposalType::Funding,
            description: "Test proposal".to_string(),
            resource_amount: None,
            duration: 60,
            status: crate::governance::ProposalStatus::Open,
        };

        let result = core.submit_proposal("did:icn:test".to_string(), proposal).await;
        assert!(result.is_ok());
    }
}