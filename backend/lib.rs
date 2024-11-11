// lib.rs
//
// Core module for the Inter-Cooperative Network (ICN)
// This implementation combines blockchain, identity, governance and relationship systems
// to create a cooperative-centric network focused on human relationships and mutual aid.

// Module declarations
pub mod blockchain;
pub mod identity;
pub mod reputation;
pub mod governance;
pub mod utils;
pub mod vm;
pub mod websocket;
pub mod consensus;
pub mod network;
pub mod relationship;  // New module for relationship management

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use chrono;
use uuid;

// Core system imports
use crate::blockchain::{Block, Blockchain, Transaction, TransactionType};
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::governance::Proposal;
use crate::consensus::types::ConsensusRound;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::vm::opcode::OpCode;
use crate::vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact};
use crate::websocket::WebSocketHandler;
use crate::relationship::{
    RelationshipSystem,
    Contribution,
    MutualAidInteraction,
    Relationship
};

/// Core ICN system managing all subsystems and their interactions
pub struct ICNCore {
    /// Blockchain for immutable record-keeping
    blockchain: Arc<Mutex<Blockchain>>,
    /// Identity system for DID management
    identity_system: Arc<Mutex<IdentitySystem>>,
    /// Reputation system for tracking cooperative engagement
    reputation_system: Arc<Mutex<ReputationSystem>>,
    /// Relationship system for managing cooperative bonds and mutual aid
    relationship_system: Arc<Mutex<RelationshipSystem>>,
    /// WebSocket handler for real-time updates
    ws_handler: Arc<WebSocketHandler>,
    /// Virtual Machine for executing cooperative contracts
    vm: Arc<Mutex<VM>>,
    /// Event bus for system-wide notifications
    event_bus: broadcast::Sender<SystemEvent>,
}

/// Events that can occur within the ICN system
#[derive(Clone, Debug)]
pub enum SystemEvent {
    // Blockchain events
    BlockCreated(Block),
    
    // Governance events
    ProposalSubmitted(Proposal),
    VoteCast { proposal_id: u64, voter: String, vote: bool },
    
    // Reputation events
    ReputationChanged { did: String, change: i64, reason: String },
    
    // Consensus events
    ConsensusStarted(ConsensusRound),
    ConsensusFinished(Block),
    
    // Cooperative events
    CooperativeCreated { id: String, creator: String },
    CooperativeJoined { id: String, member: String },
    
    // Relationship events
    ContributionRecorded { 
        contribution: Contribution 
    },
    MutualAidProvided { 
        interaction: MutualAidInteraction 
    },
    RelationshipUpdated { 
        member_one: String,
        member_two: String,
        update_type: String 
    },
}

impl ICNCore {
    /// Creates a new instance of the ICN core system
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        // Initialize core systems
        let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
        let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
        let relationship_system = Arc::new(Mutex::new(RelationshipSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());

        // Initialize blockchain with required systems
        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
        )));

        // Initialize VM with reputation context
        let reputation_context = reputation_system.lock()
            .unwrap()
            .get_reputation_context();
        let vm = VM::new(1000, reputation_context);
        let vm = Arc::new(Mutex::new(vm));

        ICNCore {
            blockchain,
            identity_system,
            reputation_system,
            relationship_system,
            ws_handler,
            vm,
            event_bus: event_tx,
        }
    }

    /// Creates a new cooperative within the network
    pub async fn create_cooperative(
        &self, 
        creator_did: String, 
        metadata: CooperativeMetadata
    ) -> Result<String, String> {
        // Verify creator's identity
        let identity = self.identity_system.lock()
            .map_err(|_| "Failed to acquire identity lock".to_string())?;
        if !identity.is_registered(&creator_did) {
            return Err("Creator DID not registered".to_string());
        }
        drop(identity);

        // Check reputation requirements
        let reputation = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
        if reputation.get_reputation(&creator_did) < 100 {
            return Err("Insufficient reputation to create cooperative".to_string());
        }
        let reputation_score = reputation.get_reputation(&creator_did);
        drop(reputation);

        // Create and configure the contract
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

        // Set up execution context
        let context = ExecutionContext {
            caller_did: creator_did.clone(),
            cooperative_id: contract.id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            block_number: {
                let blockchain = self.blockchain.lock()
                    .map_err(|_| "Failed to acquire blockchain lock".to_string())?;
                blockchain.current_block_number
            },
            reputation_score,
            permissions: vec!["cooperative.create".to_string()],
        };

        // Execute contract
        {
            let mut vm = self.vm.lock()
                .map_err(|_| "Failed to acquire VM lock".to_string())?;
            vm.set_execution_context(context);
            vm.execute_contract(&contract)?;
        }

        // Create and add transaction
        let transaction = Transaction::new(
            creator_did.clone(),
            TransactionType::ContractExecution {
                contract_id: contract.id.clone(),
                input_data: HashMap::new(),
            },
        );

        {
            let mut blockchain = self.blockchain.lock()
                .map_err(|_| "Failed to acquire blockchain lock".to_string())?;
            blockchain.add_transaction(transaction).await?;
        }

        // Emit creation event
        let event = SystemEvent::CooperativeCreated {
            id: contract.id.clone(),
            creator: creator_did,
        };
        let _ = self.event_bus.send(event);

        Ok(contract.id)
    }

    /// Submit a new proposal to the governance system
    pub async fn submit_proposal(
        &self, 
        creator_did: String, 
        proposal: Proposal
    ) -> Result<u64, String> {
        // Check reputation requirements
        let reputation = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
        if reputation.get_reputation(&creator_did) < proposal.required_reputation {
            return Err("Insufficient reputation to create proposal".to_string());
        }
        drop(reputation);

        // Create and add transaction
        let transaction = Transaction::new(
            creator_did,
            TransactionType::ContractExecution {
                contract_id: "governance".to_string(),
                input_data: HashMap::new(),
            },
        );

        {
            let mut blockchain = self.blockchain.lock()
                .map_err(|_| "Failed to acquire blockchain lock".to_string())?;
            blockchain.add_transaction(transaction).await?;
        }

        // Emit proposal event
        let event = SystemEvent::ProposalSubmitted(proposal.clone());
        let _ = self.event_bus.send(event);

        Ok(proposal.id)
    }

    /// Start a new consensus round
    pub async fn start_consensus_round(&self) -> Result<(), String> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| "Failed to acquire blockchain lock".to_string())?;
        
        if let Some(round) = blockchain.get_current_round() {
            drop(blockchain);
            
            let event = SystemEvent::ConsensusStarted(round);
            let _ = self.event_bus.send(event);
        }

        Ok(())
    }

    /// Record a new contribution to the community
    pub async fn record_contribution(&self, contribution: Contribution) -> Result<(), String> {
        let mut relationship_system = self.relationship_system.lock()
            .map_err(|_| "Failed to acquire relationship lock".to_string())?;

        // Record the contribution
        relationship_system.record_contribution(contribution.clone())?;

        // Emit contribution event
        let event = SystemEvent::ContributionRecorded { contribution };
        let _ = self.event_bus.send(event);

        Ok(())
    }

    /// Record a mutual aid interaction between members
    pub async fn record_mutual_aid(&self, interaction: MutualAidInteraction) -> Result<(), String> {
        let mut relationship_system = self.relationship_system.lock()
            .map_err(|_| "Failed to acquire relationship lock".to_string())?;

        // Record the mutual aid interaction
        relationship_system.record_mutual_aid(interaction.clone())?;

        // Emit mutual aid event
        let event = SystemEvent::MutualAidProvided { interaction };
        let _ = self.event_bus.send(event);

        Ok(())
    }

    /// Get all relationships for a member
    pub async fn get_member_relationships(&self, did: &str) -> Result<Vec<Relationship>, String> {
        let relationship_system = self.relationship_system.lock()
            .map_err(|_| "Failed to acquire relationship lock".to_string())?;

        Ok(relationship_system.get_member_relationships(did).to_vec())
    }

    /// Subscribe to system events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_bus.subscribe()
    }
}

/// Manager for cooperative-specific operations
pub struct CooperativeManager {
    core: Arc<ICNCore>,
}

impl CooperativeManager {
    /// Create a new cooperative manager
    pub fn new(core: Arc<ICNCore>) -> Self {
        CooperativeManager { core }
    }

    /// Create a new cooperative with specified metadata
    pub async fn create_cooperative(
        &self, 
        creator_did: String, 
        _name: String,
        purpose: String
    ) -> Result<String, String> {
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

    /// Add a member to an existing cooperative
    pub async fn join_cooperative(
        &self, 
        cooperative_id: String, 
        member_did: String
    ) -> Result<(), String> {
        let event = SystemEvent::CooperativeJoined {
            id: cooperative_id,
            member: member_did,
        };
        let _ = self.core.event_bus.send(event);
        Ok(())
    }
}

/// Node implementation for the ICN network
pub struct ICNNode {
    core: Arc<ICNCore>,
    node_id: String,
    peers: Arc<Mutex<HashMap<String, String>>>,
}

impl ICNNode {
    /// Create a new ICN node
    pub fn new(core: Arc<ICNCore>, node_id: String) -> Self {
        ICNNode {
            core,
            node_id,
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the node and begin processing events
    pub async fn start(&self) -> Result<(), String> {
        let mut event_rx = self.core.subscribe_to_events();
        self.core.start_consensus_round().await?;

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    SystemEvent::BlockCreated(block) => {
                        println!("New block created: {}", block.index);
                    }
                    SystemEvent::ConsensusStarted(round) => {
                        println!("New consensus round started: {}", round.round_number);
                    }
                    SystemEvent::ContributionRecorded { contribution } => {
                        println!("New contribution recorded: {}", contribution.description);
                    }
                    SystemEvent::MutualAidProvided { interaction } => {
                        println!("Mutual aid provided: {}", interaction.description);
                    }
                    SystemEvent::RelationshipUpdated { member_one, member_two, update_type } => {
                        println!("Relationship updated between {} and {}: {}", 
                            member_one, member_two, update_type);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Connect to a peer node
    pub async fn connect_to_peer(&self, peer_id: String, address: String) -> Result<(), String> {
        self.peers.lock()
            .map_err(|_| "Failed to acquire peers lock".to_string())?
            .insert(peer_id, address);
        Ok(())
    }
}