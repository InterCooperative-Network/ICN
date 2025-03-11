use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;
use serde_json;
use uuid::Uuid;
use icn_types::{FederationId, MemberId};
use std::collections::HashSet;
use chrono;
use chrono::{Utc};
use thiserror::Error;
use crate::resource_manager::ResourceProvider;
use icn_reputation::ReputationService;

pub mod federation;
pub mod governance;
pub mod dispute;
pub mod messaging;
pub mod treasury;
pub mod cross_federation;
pub mod resource_manager;
pub mod resource_sharing;

pub use federation::{
    Federation, FederationType, FederationTerms, FederationError as FederationModuleError, 
    FederationStatus, MemberStatus, MemberRole, ResourcePool, ResourceType,
    ProposalType, ProposalStatus, Vote, VoteDecision, MembershipAction,
    ResourceAllocationDetails, MemberInfo, ResourceAllocation
};

pub use governance::{
    GovernanceManager, GovernanceConfig, GovernanceProposal, 
    GovernanceError, GovernanceResult, VotingStrategy
};

pub use dispute::{
    DisputeManager, DisputeConfig, Dispute, DisputeError, DisputeResult,
    DisputeType, DisputeStatus, ResolutionMethod, ResolutionOutcome
};

use resource_manager::ResourceProvider as ResourceManagerTrait;

/// Result type for federation operations
pub type FederationResult<T> = Result<T, FederationError>;

/// Helper to convert a String to a FederationId
pub fn federation_id_from_string(id: String) -> FederationId {
    FederationId(id)
}

/// Helper to convert a FederationId to a String
pub fn federation_id_to_string(id: &FederationId) -> String {
    id.0.clone()
}

// Add SDP support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDPConfig {
    pub bind_address: String,
    pub enable_multipath: bool,
    pub enable_onion_routing: bool,
    pub message_priority: HashMap<String, u8>,
}

impl Default for SDPConfig {
    fn default() -> Self {
        let mut message_priority = HashMap::new();
        message_priority.insert("governance_vote".to_string(), 8);
        message_priority.insert("dispute_resolution".to_string(), 9);
        message_priority.insert("resource_allocation".to_string(), 6);
        message_priority.insert("member_update".to_string(), 5);
        
        Self {
            bind_address: "0.0.0.0:0".to_string(),
            enable_multipath: true,
            enable_onion_routing: false,
            message_priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    pub source_federation: String,
    pub target_federation: String,
    pub message_type: FederationMessageType,
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessageType {
    ProposalSubmission,
    Vote,
    DisputeInitiation,
    ResourceAllocation,
    MembershipUpdate,
}

impl ToString for FederationMessageType {
    fn to_string(&self) -> String {
        match self {
            Self::ProposalSubmission => "proposal_submission".to_string(),
            Self::Vote => "vote".to_string(),
            Self::DisputeInitiation => "dispute_initiation".to_string(),
            Self::ResourceAllocation => "resource_allocation".to_string(),
            Self::MembershipUpdate => "membership_update".to_string(),
        }
    }
}

#[async_trait]
pub trait ResourceManager: Send + Sync {
    async fn allocate_resources(&self, allocation: ResourceAllocationDetails) -> Result<(), FederationError>;
    async fn deallocate_resources(&self, allocation: ResourceAllocationDetails) -> Result<(), FederationError>;
    async fn get_available_resources(&self) -> Result<HashMap<ResourceType, u64>, FederationError>;
}

/// Top-level federation manager that coordinates federation, governance and dispute resolution
pub struct FederationManager {
    federations: Arc<RwLock<HashMap<String, Federation>>>,
    governance_manager: Option<Arc<GovernanceManager>>,
    dispute_manager: Option<Arc<DisputeManager>>,
    reputation_service: Option<Arc<dyn ReputationService>>,
    resource_manager: Option<Arc<dyn ResourceProvider>>,
    sdp_config: SDPConfig,
}

impl FederationManager {
    /// Create a new federation manager
    pub fn new(
        governance_manager: Option<Arc<GovernanceManager>>,
        dispute_manager: Option<Arc<DisputeManager>>,
        reputation_service: Option<Arc<dyn ReputationService>>,
        resource_manager: Option<Arc<dyn ResourceProvider>>,
    ) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            governance_manager,
            dispute_manager,
            reputation_service,
            resource_manager,
            sdp_config: SDPConfig::default(),
        }
    }
    
    /// Get federation by id
    pub async fn get_federation(&self, id: &str) -> Result<Federation, FederationError> {
        let federations = self.federations.read().await;
        
        match federations.get(id) {
            Some(federation) => Ok(federation.clone()),
            None => Err(FederationError::NotFound(format!("Federation not found: {}", id)))
        }
    }
    
    /// Create a new federation
    pub async fn create_federation(
        &self,
        name: String,
        description: String,
    ) -> Result<Federation, FederationError> {
        let id = Uuid::new_v4().to_string();
        let federation_id = federation_id_from_string(id);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let federation = Federation {
            id: federation_id.clone(),
            name,
            description,
            founded_date: Utc::now(),
            members: HashSet::new(),
            resource_manager: self.resource_manager.clone(),
            metadata: HashMap::new(),
            federation_type: federation::FederationType::Custom("Standard".to_string()),
            member_roles: HashMap::new(),
            terms: federation::FederationTerms::default(),
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: now,
            status: federation::FederationStatus::Active,
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
        };
        
        let mut federations = self.federations.write().await;
        federations.insert(federation_id_to_string(&federation_id), federation.clone());
        
        Ok(federation)
    }
    
    /// Update an existing federation
    pub async fn update_federation(&self, federation: Federation) -> Result<(), FederationError> {
        // Check if federation exists
        let mut federations = self.federations.write().await;
        let federation_id_str = federation_id_to_string(&federation.id);
        
        if !federations.contains_key(&federation_id_str) {
            return Err(FederationError::NotFound(format!("Federation not found: {}", federation_id_str)));
        }
        
        // Update federation
        federations.insert(federation_id_str, federation.clone());
        
        // Update in subsystems
        if let Some(governance) = &self.governance_manager {
            governance.register_federation(federation.clone())
                .await
                .map_err(|e| FederationError::GovernanceError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Get all federations
    pub async fn list_federations(&self) -> Vec<Federation> {
        let federations = self.federations.read().await;
        federations.values().cloned().collect()
    }
    
    /// Create a proposal
    pub async fn create_proposal(
        &self, 
        federation_id: &str, 
        title: String, 
        description: String, 
        proposer: String, 
        proposal_type_str: String, 
        details: HashMap<String, String>
    ) -> FederationResult<String> {
        let fed_id = federation_id_from_string(federation_id.to_string());
        let federations = self.federations.read().await;
        
        // Ensure federation exists
        let _federation = federations.get(federation_id)
            .ok_or_else(|| FederationError::FederationNotFound(federation_id.to_string()))?;
        
        if let Some(governance) = &self.governance_manager {
            // Convert the string proposal type to a ProposalType enum
            let proposal_type = ProposalType::Custom(proposal_type_str);
            
            // Create an empty tags vector
            let tags: Vec<String> = Vec::new();
            
            // No specific voting period, use default
            let voting_period: Option<u64> = None;
            
            let proposal_id = governance.create_proposal(
                title, 
                description, 
                proposer, 
                fed_id, 
                proposal_type, 
                voting_period, 
                tags
            ).await
                .map_err(|e| FederationError::GovernanceError(e.to_string()))?;
            
            Ok(proposal_id)
        } else {
            Err(FederationError::GovernanceManagerNotConfigured)
        }
    }
    
    /// Submit a vote on a proposal
    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter: String,
        decision: VoteDecision,
        justification: Option<String>,
    ) -> Result<(), FederationError> {
        if let Some(governance) = &self.governance_manager {
            governance.submit_vote(
                proposal_id,
                voter,
                decision,
                justification,
            ).await
            .map_err(|e| FederationError::GovernanceError(e.to_string()))
        } else {
            Err(FederationError::GovernanceManagerNotConfigured)
        }
    }
    
    /// File a dispute
    pub async fn file_dispute(
        &self, 
        federation_id: &str, 
        title: String, 
        description: String, 
        complainant: String,
        respondents: Vec<String>,
        dispute_type: DisputeType,
        severity: u8,
    ) -> FederationResult<String> {
        let fed_id = federation_id_from_string(federation_id.to_string());
        let federations = self.federations.read().await;
        
        // Ensure federation exists
        let _federation = federations.get(federation_id)
            .ok_or_else(|| FederationError::FederationNotFound(federation_id.to_string()))?;
        
        if let Some(dispute_manager) = &self.dispute_manager {
            let dispute_id = dispute_manager.file_dispute(
                title,
                description,
                complainant,
                respondents,
                fed_id,
                dispute_type,
                severity
            ).await
                .map_err(|e| FederationError::DisputeError(e.to_string()))?;
            Ok(dispute_id)
        } else {
            Err(FederationError::DisputeManagerNotConfigured)
        }
    }
    
    /// Get governance manager
    pub fn governance_manager(&self) -> Arc<GovernanceManager> {
        self.governance_manager.clone().unwrap()
    }
    
    /// Get dispute manager
    pub fn dispute_manager(&self) -> Arc<DisputeManager> {
        self.dispute_manager.clone().unwrap()
    }

    /// Add a member to a federation
    pub async fn add_member(
        &self,
        federation_id: &str,
        member_id: MemberId,
        roles: Vec<federation::MemberRole>,
    ) -> Result<(), FederationError> {
        // Get federation
        let federation = self.get_federation(federation_id).await?;
        
        // Check if member already exists
        if federation.members.contains(&member_id) {
            return Err(FederationError::AlreadyExists(format!("Member {:?} already exists", member_id)));
        }
        
        // Create membership action
        let action = federation::MembershipAction::Add(member_id.did.clone());
        
        // Create updated federation with new member
        let mut updated_federation = federation.clone();
        updated_federation.apply_membership_action(action)?;
        
        // Add roles if specified
        if !roles.is_empty() {
            updated_federation.member_roles.insert(member_id.did.clone(), roles);
        }
        
        // Update federation
        self.update_federation(updated_federation).await
    }

    /// Allocate resources to a member
    pub async fn allocate_resources(
        &self,
        federation_id: &str,
        member_id: &MemberId,
        resource_type: &str,
        amount: u64,
    ) -> Result<(), FederationError> {
        // Get federation
        let federation = self.get_federation(federation_id).await?;
        
        // Check if member exists
        if !federation.members.contains(member_id) {
            return Err(FederationError::NotFound(format!("Member {:?} not found", member_id)));
        }
        
        // Parse resource type
        let resource_type = match resource_type {
            "compute" => federation::ResourceType::ComputeUnit,
            "storage" => federation::ResourceType::StorageGb,
            "bandwidth" => federation::ResourceType::BandwidthMbps,
            "memory" => federation::ResourceType::MemoryGb,
            custom => federation::ResourceType::CustomResource(custom.to_string()),
        };
        
        // Create resource allocation details
        let allocation = federation::ResourceAllocationDetails {
            resource_type,
            member_id: member_id.did.clone(),
            amount,
            duration: 0, // No expiration
            details: HashMap::new(),
        };
        
        // Create updated federation with resource allocation
        let mut updated_federation = federation.clone();
        updated_federation.allocate_resource(allocation)?;
        
        // Update federation
        self.update_federation(updated_federation).await
    }

    pub async fn create_governance_proposal(
        &self, 
        federation_id: &str, 
        title: String, 
        description: String, 
        proposer: String, 
        proposal_type_str: String, 
        details: HashMap<String, String>
    ) -> FederationResult<String> {
        let fed_id = federation_id_from_string(federation_id.to_string());
        let federations = self.federations.read().await;
        
        // Ensure federation exists
        let _federation = federations.get(federation_id)
            .ok_or_else(|| FederationError::FederationNotFound(federation_id.to_string()))?;
        
        if let Some(governance) = &self.governance_manager {
            // Convert the string proposal type to a ProposalType enum
            let proposal_type = ProposalType::Custom(proposal_type_str);
            
            // Create an empty tags vector
            let tags: Vec<String> = Vec::new();
            
            // No specific voting period, use default
            let voting_period: Option<u64> = None;
            
            let proposal_id = governance.create_proposal(
                title, 
                description, 
                proposer, 
                fed_id, 
                proposal_type, 
                voting_period, 
                tags
            ).await
                .map_err(|e| FederationError::GovernanceError(e.to_string()))?;
            
            Ok(proposal_id)
        } else {
            Err(FederationError::GovernanceManagerNotConfigured)
        }
    }
}

/// Federation error types
#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Governance error: {0}")]
    GovernanceError(String),
    
    #[error("Dispute error: {0}")]
    DisputeError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Governance manager not configured")]
    GovernanceManagerNotConfigured,
    
    #[error("Dispute manager not configured")]
    DisputeManagerNotConfigured,
    
    #[error("Resource manager not configured")]
    ResourceManagerNotConfigured,
    
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
}

/// Implementation to convert federation::FederationError to FederationError
impl From<federation::FederationError> for FederationError {
    fn from(err: federation::FederationError) -> Self {
        FederationError::InternalError(err.to_string())
    }
}
