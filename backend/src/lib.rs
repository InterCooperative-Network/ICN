// src/lib.rs

pub mod blockchain;
pub mod claims;
pub mod identity;
pub mod reputation;
pub mod governance;
pub mod utils;
pub mod vm;
pub mod websocket;
pub mod consensus;
pub mod network;
pub mod monitoring;
pub mod relationship;
pub mod community;

pub use blockchain::{Block, Blockchain, Transaction, TransactionType};
pub use identity::IdentitySystem;
pub use reputation::ReputationSystem;
pub use governance::Proposal;
pub use consensus::{ProofOfCooperation, types::ConsensusConfig};
pub use consensus::types::ConsensusRound;
pub use vm::{VM, Contract, ExecutionContext};
pub use vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact};
pub use websocket::WebSocketHandler;
pub use monitoring::energy::{EnergyAware, EnergyMonitor};
pub use relationship::{Contribution, MutualAidInteraction, RelationshipSystem};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid;
use chrono::{DateTime, Utc};

/// Events emitted by the ICN system
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
    ContributionRecorded(Contribution),
    MutualAidProvided(MutualAidInteraction),
    RelationshipUpdated { member_one: String, member_two: String },
}

/// Core system for the Inter-Cooperative Network
pub struct ICNCore {
    blockchain: Arc<Mutex<Blockchain>>,
    identity_system: Arc<Mutex<IdentitySystem>>,
    reputation_system: Arc<Mutex<ReputationSystem>>,
    relationship_system: Arc<Mutex<RelationshipSystem>>,
    vm: Arc<Mutex<VM>>,
    event_bus: broadcast::Sender<SystemEvent>,
    start_time: DateTime<Utc>,
}

impl ICNCore {
    /// Creates a new instance of the ICN core system
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
        let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
        let relationship_system = Arc::new(Mutex::new(RelationshipSystem::new()));
        let ws_handler = Arc::new(WebSocketHandler::new());

        let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
            ConsensusConfig::default(),
            ws_handler.clone(),
        )));

        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            identity_system.clone(),
            reputation_system.clone(),
            relationship_system.clone(),
            consensus.clone()
        )));

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
            vm,
            event_bus: event_tx,
            start_time: Utc::now(),
        }
    }

    /// Creates a new cooperative with the given creator and metadata
    pub async fn create_cooperative(
        &self,
        creator_did: String,
        metadata: CooperativeMetadata
    ) -> Result<String, String> {
        // Validate creator's identity
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

        // Create and execute contract
        let contract = Contract {
            id: uuid::Uuid::new_v4().to_string(),
            code: vec![vm::opcode::OpCode::CreateCooperative {
                name: metadata.cooperative_id.clone(),
                description: metadata.purpose.clone(),
                resources: HashMap::new(),
                federation_id: metadata.federation_id.clone(),
            }],
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
            timestamp: Utc::now().timestamp() as u64,
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

        // Create and submit transaction
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

    /// Records a new contribution in the system
    pub async fn record_contribution(
        &self,
        contribution: Contribution
    ) -> Result<(), String> {
        let mut relationship_system = self.relationship_system.lock()
            .map_err(|_| "Failed to acquire relationship lock".to_string())?;
        
        relationship_system.record_contribution(contribution.clone())?;

        let event = SystemEvent::ContributionRecorded(contribution);
        let _ = self.event_bus.send(event);

        Ok(())
    }

    /// Records a mutual aid interaction between members
    pub async fn record_mutual_aid(
        &self,
        interaction: MutualAidInteraction
    ) -> Result<(), String> {
        let mut relationship_system = self.relationship_system.lock()
            .map_err(|_| "Failed to acquire relationship lock".to_string())?;
        
        relationship_system.record_mutual_aid(interaction.clone())?;

        let event = SystemEvent::MutualAidProvided(interaction);
        let _ = self.event_bus.send(event);

        Ok(())
    }

    /// Starts a new consensus round
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

    /// Submits a new proposal for voting
    pub async fn submit_proposal(
        &self,
        creator_did: String,
        proposal: Proposal
    ) -> Result<u64, String> {
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

    /// Subscribes to system events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_bus.subscribe()
    }

    /// Gets system uptime in seconds
    pub fn get_uptime(&self) -> i64 {
        (Utc::now() - self.start_time).num_seconds()
    }

    /// Gets system statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("uptime".to_string(), self.get_uptime().to_string());
        
        if let Ok(blockchain) = self.blockchain.lock() {
            stats.insert("block_count".to_string(), blockchain.get_block_count().to_string());
            stats.insert("tx_count".to_string(), blockchain.get_transaction_count().to_string());
        }

        stats
    }
}

/// Manager for cooperative-specific operations
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
            creation_timestamp: Utc::now().timestamp() as u64,
            last_updated: Utc::now().timestamp() as u64,
            member_count: 1,
            resource_allocation: HashMap::new(),
            energy_usage: HashMap::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icn_core_creation() {
        let core = ICNCore::new();
        let _receiver = core.subscribe_to_events();
        assert!(core.get_uptime() >= 0);
    }

    #[tokio::test]
    async fn test_cooperative_creation() {
        let core = Arc::new(ICNCore::new());
        let manager = CooperativeManager::new(core);
        
        let result = manager.create_cooperative(
            "did:icn:test".to_string(),
            "Test Cooperative".to_string(),
            "Test Purpose".to_string()
        ).await;
        
        assert!(result.is_err()); // Should fail because DID not registered
    }

    #[test]
    fn test_system_stats() {
        let core = ICNCore::new();
        let stats = core.get_stats();
        assert!(stats.contains_key("uptime"));
        assert!(stats.contains_key("block_count"));
        assert!(stats.contains_key("tx_count"));
    }
}