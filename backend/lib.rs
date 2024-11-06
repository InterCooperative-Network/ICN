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
use crate::consensus::types::ConsensusRound;
use crate::vm::{VM, Contract, ExecutionContext};
use crate::vm::opcode::OpCode;
use crate::vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact};
use crate::websocket::WebSocketHandler;

pub struct ICNCore {
    blockchain: Arc<Mutex<Blockchain>>,
    identity_system: Arc<Mutex<IdentitySystem>>,
    reputation_system: Arc<Mutex<ReputationSystem>>,
    ws_handler: Arc<WebSocketHandler>,
    vm: Arc<Mutex<VM>>,
    event_bus: broadcast::Sender<SystemEvent>,
}

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

        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
        )));

        let vm = VM::new(1000, reputation_system.lock().unwrap().get_reputation_context());
        let vm = Arc::new(Mutex::new(vm));

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
        let identity = self.identity_system.lock()
            .map_err(|_| "Failed to acquire identity lock".to_string())?;
        if !identity.is_registered(&creator_did) {
            return Err("Creator DID not registered".to_string());
        }
        drop(identity);

        let reputation = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
        if reputation.get_reputation(&creator_did) < 100 {
            return Err("Insufficient reputation to create cooperative".to_string());
        }
        let reputation_score = reputation.get_reputation(&creator_did);
        drop(reputation);

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

        {
            let mut vm = self.vm.lock()
                .map_err(|_| "Failed to acquire VM lock".to_string())?;
            vm.set_execution_context(context);
            vm.execute_contract(&contract)?;
        }

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

        let event = SystemEvent::CooperativeCreated {
            id: contract.id.clone(),
            creator: creator_did,
        };
        let _ = self.event_bus.send(event);

        Ok(contract.id)
    }

    pub async fn submit_proposal(&self, creator_did: String, proposal: Proposal) -> Result<u64, String> {
        let reputation = self.reputation_system.lock()
            .map_err(|_| "Failed to acquire reputation lock".to_string())?;
        if reputation.get_reputation(&creator_did) < proposal.required_reputation {
            return Err("Insufficient reputation to create proposal".to_string());
        }
        drop(reputation);

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

        let event = SystemEvent::ProposalSubmitted(proposal.clone());
        let _ = self.event_bus.send(event);

        Ok(proposal.id)
    }

    pub async fn start_consensus_round(&self) -> Result<(), String> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| "Failed to acquire blockchain lock".to_string())?;
        
        if let Some(round) = blockchain.get_current_round() {
            let consensus_round = ConsensusRound {
                round_number: round.round_number,
                coordinator: round.coordinator.clone(),
                start_time: round.start_time,
                timeout: round.timeout,
                status: round.status.clone(),
                proposed_block: round.proposed_block.clone(),
                votes: round.votes.clone(),
                stats: round.stats.clone(),
            };
            drop(blockchain);
            
            let event = SystemEvent::ConsensusStarted(consensus_round);
            let _ = self.event_bus.send(event);
        }

        Ok(())
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_bus.subscribe()
    }
}

pub struct CooperativeManager {
    core: Arc<ICNCore>,
}

impl CooperativeManager {
    pub fn new(core: Arc<ICNCore>) -> Self {
        CooperativeManager { core }
    }

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

    pub async fn join_cooperative(&self, cooperative_id: String, member_did: String) -> Result<(), String> {
        let event = SystemEvent::CooperativeJoined {
            id: cooperative_id,
            member: member_did,
        };
        let _ = self.core.event_bus.send(event);
        Ok(())
    }
}

pub struct ICNNode {
    core: Arc<ICNCore>,
    node_id: String,
    peers: Arc<Mutex<HashMap<String, String>>>,
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
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(&self, peer_id: String, address: String) -> Result<(), String> {
        self.peers.lock()
            .map_err(|_| "Failed to acquire peers lock".to_string())?
            .insert(peer_id, address);
        Ok(())
    }
}
