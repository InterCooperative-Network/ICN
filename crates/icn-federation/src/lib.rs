use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;
use serde_json;
use uuid::Uuid;

pub mod federation;
pub mod governance;
pub mod dispute;

pub use federation::{
    Federation, FederationType, FederationTerms, FederationError, 
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

// Define our own FederationId type that can be converted from/to string easily
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FederationId(pub String);

impl From<String> for FederationId {
    fn from(id: String) -> Self {
        FederationId(id)
    }
}

impl From<&str> for FederationId {
    fn from(id: &str) -> Self {
        FederationId(id.to_string())
    }
}

impl From<FederationId> for String {
    fn from(id: FederationId) -> Self {
        id.0
    }
}

impl std::fmt::Display for FederationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
    governance_manager: Arc<governance::GovernanceManager>,
    dispute_manager: Arc<dispute::DisputeManager>,
    resource_manager: Arc<dyn ResourceManager>,
    sdp_config: SDPConfig,
}

impl FederationManager {
    pub fn new(
        governance_manager: Arc<governance::GovernanceManager>,
        dispute_manager: Arc<dispute::DisputeManager>,
        resource_manager: Arc<dyn ResourceManager>,
    ) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            governance_manager,
            dispute_manager,
            resource_manager,
            sdp_config: SDPConfig::default(),
        }
    }
    
    /// Create a new federation
    pub async fn create_federation(
        &self,
        id: String,
        name: String,
        federation_type: FederationType,
        terms: FederationTerms,
        founder: String,
    ) -> Result<String, FederationError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Create federation with founder as first member
        let mut federation = Federation {
            id: id.clone(),
            name,
            federation_type,
            members: HashMap::new(),
            member_roles: HashMap::new(),
            terms,
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: now,
            status: FederationStatus::Active,
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
        };
        
        // Add founder
        federation.members.insert(founder.clone(), MemberInfo::default());
        
        // Set founder role
        federation.member_roles.insert(
            founder,
            vec![MemberRole::Founder, MemberRole::Admin],
        );
        
        // Register with subsystems
        self.governance_manager.register_federation(federation.clone()).await
            .map_err(|e| FederationError::GovernanceError(e.to_string()))?;
            
        self.dispute_manager.register_federation(federation.clone()).await
            .map_err(|e| FederationError::DisputeError(e.to_string()))?;
        
        // Store federation
        let mut federations = self.federations.write().await;
        federations.insert(id.clone(), federation);
        
        Ok(id)
    }
    
    /// Get a federation by ID
    pub async fn get_federation(&self, id: &str) -> Result<Federation, FederationError> {
        let federations = self.federations.read().await;
        federations.get(id)
            .cloned()
            .ok_or_else(|| FederationError::FederationNotFound(id.to_string()))
    }
    
    /// Update a federation
    pub async fn update_federation(&self, federation: Federation) -> Result<(), FederationError> {
        // Check if federation exists
        let mut federations = self.federations.write().await;
        if !federations.contains_key(&federation.id) {
            return Err(FederationError::FederationNotFound(federation.id));
        }
        
        // Update federation
        federations.insert(federation.id.clone(), federation.clone());
        
        // Update in subsystems
        self.governance_manager.register_federation(federation.clone()).await
            .map_err(|e| FederationError::GovernanceError(e.to_string()))?;
            
        self.dispute_manager.register_federation(federation).await
            .map_err(|e| FederationError::DisputeError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get all federations
    pub async fn list_federations(&self) -> Vec<Federation> {
        let federations = self.federations.read().await;
        federations.values().cloned().collect()
    }
    
    /// Create a proposal for a federation
    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposer: String,
        federation_id: String,
        proposal_type: ProposalType,
        voting_period: Option<u64>,
        tags: Option<Vec<String>>,
    ) -> Result<String, FederationError> {
        let federation_id = FederationId(federation_id);
        let tags = tags.unwrap_or_default();
        
        self.governance_manager.create_proposal(
            title,
            description,
            proposer,
            federation_id,
            proposal_type,
            voting_period,
            tags,
        ).await
        .map_err(|e| FederationError::GovernanceError(e.to_string()))
    }
    
    /// Submit a vote on a proposal
    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter: String,
        decision: VoteDecision,
        justification: Option<String>,
    ) -> Result<(), FederationError> {
        self.governance_manager.submit_vote(
            proposal_id,
            voter,
            decision,
            justification,
        ).await
        .map_err(|e| FederationError::GovernanceError(e.to_string()))
    }
    
    /// File a dispute
    pub async fn file_dispute(
        &self,
        title: String,
        description: String,
        complainant: String,
        respondents: Vec<String>,
        federation_id: String,
        dispute_type: DisputeType,
        severity: u8,
    ) -> Result<String, FederationError> {
        let federation_id = FederationId(federation_id);
        
        self.dispute_manager.file_dispute(
            title,
            description,
            complainant,
            respondents,
            federation_id,
            dispute_type,
            severity,
        ).await
        .map_err(|e| FederationError::DisputeError(e.to_string()))
    }
    
    /// Get governance manager
    pub fn governance_manager(&self) -> Arc<governance::GovernanceManager> {
        self.governance_manager.clone()
    }
    
    /// Get dispute manager
    pub fn dispute_manager(&self) -> Arc<dispute::DisputeManager> {
        self.dispute_manager.clone()
    }

    pub async fn add_member(
        &self,
        federation_id: String,
        member_did: String,
        commitments: Vec<String>,
    ) -> Result<(), FederationError> {
        let federation_id = FederationId(federation_id);
        
        // Verify federation exists
        let mut federation = match self.get_federation(&federation_id.0).await {
            Some(fed) => fed,
            None => return Err(FederationError::FederationNotFound(federation_id.0)),
        };
        
        // Check if the member is already in the federation
        if federation.members.contains_key(&member_did) {
            return Err(FederationError::AlreadyMember(member_did.to_string()));
        }
        
        // Verify eligibility (reputation, commitments, etc.)
        // TODO: Implement reputation check
        if false { // placeholder for reputation check
            return Err(FederationError::ValidationError("Member does not meet eligibility requirements".to_string()));
        }
        
        // Verify commitments
        if commitments.is_empty() {
            return Err(FederationError::ValidationError("Invalid commitments".to_string()));
        }
        
        // Add the member
        federation.members.insert(member_did.to_string(), MemberInfo::default());
        federation.member_roles.insert(member_did.to_string(), vec![MemberRole::Member]);
        
        // Update federation
        self.update_federation(federation).await
    }

    pub async fn allocate_resources(
        &self,
        federation_id: String,
        allocation: ResourceAllocationDetails,
    ) -> Result<(), FederationError> {
        let federation_id = FederationId(federation_id);
        
        // Verify federation exists
        let mut federation = match self.get_federation(&federation_id.0).await {
            Some(fed) => fed,
            None => return Err(FederationError::FederationNotFound(federation_id.0)),
        };
        
        // Verify member exists
        if !federation.members.contains_key(&allocation.member_id) {
            return Err(FederationError::MemberNotFound(allocation.member_id.clone()));
        }
        
        // Verify resource type is supported
        let resource_pool = federation.resources.get(&allocation.resource_type)
            .ok_or_else(|| FederationError::InsufficientResources("Resource type not available".to_string()))?;
        
        // Verify sufficient resources are available
        if resource_pool.available_capacity < allocation.amount {
            return Err(FederationError::InsufficientResources("Not enough resources available".to_string()));
        }
        
        // Allocate resources
        federation.allocate_resource(allocation.clone())?;
        
        // Update federation
        self.update_federation(federation).await
    }
}
