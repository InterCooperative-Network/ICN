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
use crate::consensus::{ConsensusRound};
use crate::websocket::WebSocketHandler;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::vm::opcode::OpCode;  // Corrected import for OpCode
use crate::consensus::types::{CooperativeMetadata, ResourceImpact};  // Corrected imports for CooperativeMetadata and ResourceImpact

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
    ConsensusStarted(ConsensusRound),
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
        
        // Initialize Blockchain with consensus handling
        let consensus = Arc::new(Mutex::new(crate::consensus::proof_of_cooperation::ProofOfCooperation::new(
            crate::consensus::types::ConsensusConfig::default(),
            ws_handler.clone(),
        )));
        
        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
            consensus,  // Updated to pass consensus
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

    // Core functionality for managing cooperatives
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

        // Broadcast event
        let event = SystemEvent::CooperativeCreated {
            id: contract.id.clone(),
            creator: creator_did,
        };
        let _ = self.event_bus.send(event);

        Ok(contract.id)
    }

    // Governance functionality
    pub async fn submit_proposal(&self, creator_did: String, proposal: Proposal) -> Result<u64, String> {
        let reputation = self.reputation_system.lock().unwrap();
        if reputation.get_reputation(&creator_did) < 100 {
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

        // Add to blockchain
        self.blockchain.lock().unwrap().add_transaction(transaction).await?;

        // Broadcast event
        let event = SystemEvent::ProposalSubmitted(proposal.clone());
        let _ = self.event_bus.send(event);

        Ok(proposal.id)
    }

    // Consensus functionality
    pub async fn start_consensus_round(&self) -> Result<(), String> {
        let mut blockchain = self.blockchain.lock().unwrap();
        
        // Start new consensus round
        blockchain.consensus.lock().unwrap().start_round().await?;
        
        if let Some(round) = blockchain.consensus.lock().unwrap().get_current_round() {
            // Broadcast event
            let event = SystemEvent::ConsensusStarted(round.clone());
            let _ = self.event_bus.send(event);
        }

        Ok(())
    }

    // WebSocket event handling
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_bus.subscribe()
    }
}

// Cooperative management implementation
pub struct CooperativeManager {
    core: Arc<ICNCore>,
}

impl CooperativeManager {
    pub fn new(core: Arc<ICNCore>) -> Self {
        CooperativeManager { core }
    }

    pub async fn create_cooperative(&self, creator_did: String, name: String, purpose: String) -> Result<String, String> {
        let metadata = CooperativeMetadata {
            creator_did: creator_did.clone(),
            cooperative_id: uuid::Uuid::new_v4().to_string(),
            purpose,
            resource_impact: ResourceImpact {
                cpu_intensity: 1,
                memory_usage: 1,
                network_usage: 1,
                storage_usage: 1,
                bandwidth_usage: 1,
            },
            federation_id: None,
            creation_timestamp: chrono::Utc::now().timestamp() as u64,
            last_updated: chrono::Utc::now().timestamp() as u64,
            member_count: 1,
            resource_allocation: HashMap::new(),
        };

        self.core.create_cooperative(creator_did, metadata).await
    }

    pub async fn join_cooperative(&self, cooperative_id: String, member_did: String) -> Result<(), String> {
        let event = SystemEvent::CooperativeJoined {
            id: cooperative_id,
            member: member_did,
        };
        let _ = self.core.event_bus.send(event);
        Ok(())
    }
}

// Node implementation for network participation
pub struct ICNNode {
    core: Arc<ICNCore>,
    node_id: String,
    peers: Arc<Mutex<HashMap<String, String>>>, // peer_id -> address
}

impl ICNNode {
    pub fn new(core: Arc<ICNCore>, node_id: String) -> Self {
        ICNNode {
            core,
            node_id,
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        // Subscribe to system events
        let mut event_rx = self.core.subscribe_to_events();
        
        // Start consensus participation
        self.core.start_consensus_round().await?;

        // Handle events
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    SystemEvent::BlockCreated(block) => {
                        // Propagate block to peers
                        println!("New block created: {}", block.index);
                    }
                    SystemEvent::ConsensusStarted(round) => {
                        println!("New consensus round started: {}", round.round_number);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(&self, peer_id: String, address: String) -> Result<(), String> {
        self.peers.lock().unwrap().insert(peer_id, address);
        Ok(())
    }
}
