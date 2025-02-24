use std::sync::Arc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use icn_types::{Block, Transaction, FederationOperation};

// Module declarations
pub mod blockchain;
pub mod core;
pub mod db;
pub mod identity;
pub mod reputation;
pub mod storage;
pub mod vm;
pub mod networking;
pub mod models;
pub mod telemetry;

// Re-export main components
pub use self::{
    core::Core,
    storage::{StorageManager, StorageInterface},
    networking::{NetworkManager, NetworkInterface},
    identity::{IdentityManager, IdentityInterface},
    reputation::{ReputationManager, ReputationInterface},
    telemetry::{TelemetryManager, PrometheusMetrics, Logger, TracingSystem},
    models::*,
};

use icn_consensus::ProofOfCooperation;
use tokio::time::{sleep, Duration};
use zk_snarks::verify_proof; // Import zk-SNARK verification function

pub struct Core {
    consensus: Arc<dyn ConsensusEngine>,
    storage: Arc<dyn StorageManager>,
    network: Arc<dyn NetworkManager>,
    runtime: Arc<dyn RuntimeManager>,
    telemetry: Arc<TelemetryManager>,
    identity: Arc<dyn IdentityManager>,
    reputation: Arc<dyn ReputationManager>,
    federation_manager: Arc<FederationManager>,
    resource_system: Arc<ResourceAllocationSystem>,
}

impl Core {
    pub fn new(
        storage: Arc<dyn StorageManager>,
        network: Arc<dyn NetworkManager>,
        runtime: Arc<dyn RuntimeManager>,
        telemetry: Arc<TelemetryManager>,
        identity: Arc<dyn IdentityManager>,
        reputation: Arc<dyn ReputationManager>,
    ) -> Self {
        let resource_system = Arc::new(ResourceAllocationSystem::new());
        let federation_manager = Arc::new(FederationManager::new(resource_system.clone()));
        let consensus = Arc::new(ProofOfCooperation::new(reputation.clone()));

        Core {
            consensus,
            storage,
            network,
            runtime,
            telemetry,
            identity,
            reputation,
            federation_manager,
            resource_system,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        self.telemetry.log("Starting Core...");
        if let Err(e) = self.consensus.start().await {
            return Err(format!("Failed to start consensus: {}", e));
        }
        if let Err(e) = self.network.start().await {
            return Err(format!("Failed to start network: {}", e));
        }
        if let Err(e) = self.runtime.start().await {
            return Err(format!("Failed to start runtime: {}", e));
        }
        if let Err(e) = self.identity.start().await {
            return Err(format!("Failed to start identity: {}", e));
        }
        if let Err(e) = self.reputation.start().await {
            return Err(format!("Failed to start reputation: {}", e));
        }

        // Start real-time reputation recalibration
        let reputation_system = self.reputation.clone();
        tokio::spawn(async move {
            loop {
                reputation_system.dynamic_adjustment("did:icn:test", 10).await;
                reputation_system.apply_decay("did:icn:test", 0.1).await;
                sleep(Duration::from_secs(10)).await;
            }
        });

        self.telemetry.log("Core started.");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.telemetry.log("Stopping Core...");
        if let Err(e) = self.runtime.stop().await {
            return Err(format!("Failed to stop runtime: {}", e));
        }
        if let Err(e) = self.network.stop().await {
            return Err(format!("Failed to stop network: {}", e));
        }
        if let Err(e) = self.consensus.stop().await {
            return Err(format!("Failed to stop consensus: {}", e));
        }
        if let Err(e) = self.identity.stop().await {
            return Err(format!("Failed to stop identity: {}", e));
        }
        if let Err(e) = self.reputation.stop().await {
            return Err(format!("Failed to stop reputation: {}", e));
        }
        self.telemetry.log("Core stopped.");
        Ok(())
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> Result<(), String> {
        self.telemetry.log("Processing transaction...");
        self.runtime.execute_transaction(transaction).await;
        self.telemetry.log("Transaction processed.");
        Ok(())
    }

    pub async fn add_block(&self, block: Block) -> Result<(), String> {
        self.telemetry.log("Adding block...");
        self.storage.store_block(block).await;
        self.telemetry.log("Block added.");
        Ok(())
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry.log("Creating proposal...");
        // Logic to handle proposal creation
        self.telemetry.log("Proposal created.");
        Ok(())
    }

    pub async fn vote_on_proposal(&self, vote: Vote) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry.log("Voting on proposal...");
        // Logic to handle voting on a proposal
        self.telemetry.log("Vote cast.");
        Ok(())
    }

    pub async fn handle_federation_operation(&self, operation: FederationOperation) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry.log("Handling federation operation...");
        
        match operation {
            FederationOperation::InitiateFederation { federation_type, partner_id, terms } => {
                self.federation_manager.create_federation(
                    format!("Federation with {}", partner_id),
                    federation_type,
                    terms,
                    partner_id,
                ).await?;
            }
            FederationOperation::JoinFederation { federation_id, commitment } => {
                // Get the requesting member's DID from context
                let member_did = "did:icn:requesting_member"; // This should come from auth context
                self.federation_manager.join_federation(&federation_id, member_did, commitment).await?;
            }
            // ... handle other federation operations ...
        }

        self.telemetry.log("Federation operation handled.");
        Ok(())
    }

    pub async fn allocate_resource(&self, request: ResourceAllocation) -> Result<String, Box<dyn std::error::Error>> {
        self.telemetry.log("Allocating resources...");
        let allocation_id = self.resource_system.allocate(
            &request.resource_type,
            request.recipient,
            request.amount,
        ).await?;
        self.telemetry.log("Resources allocated.");
        Ok(allocation_id)
    }

    pub async fn load_cooperative_rules(&self, dsl_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse DSL code
        let ast = icn_dsl::CoopLangAST::parse(dsl_code)
            .map_err(|e| format!("Failed to parse DSL: {}", e))?;
        
        // Compile to ICVM bytecode
        let bytecode = icn_dsl::compile_to_icvm(&ast);
        
        // Load into VM
        self.runtime.load_bytecode(&bytecode).await?;
        
        Ok(())
    }

    pub async fn handle_mutual_credit_transaction_with_proof(&self, sender: &str, receiver: &str, amount: f64, proof: &str) -> Result<(), String> {
        self.telemetry.log("Handling mutual credit transaction with zk-SNARK proof...");
        if !verify_proof(proof) {
            return Err("Invalid zk-SNARK proof".to_string());
        }
        // Placeholder logic for mutual credit transaction
        self.telemetry.log("Mutual credit transaction with zk-SNARK proof completed.");
        Ok(())
    }
}

#[async_trait]
pub trait ConsensusEngine: Send + Sync {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn get_reputation(&self, did: String, category: String) -> Result<i64, Box<dyn std::error::Error>>;
    async fn is_eligible(&self, did: String, min_reputation: i64, category: String) -> Result<bool, Box<dyn std::error::Error>>;
    async fn dynamic_adjustment(&self, did: String, contribution: i64) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub votes_for: i64,
    pub votes_against: i64,
    pub created_by: String,
    pub ends_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub approve: bool,
}

pub mod governance;   // <-- new module export
