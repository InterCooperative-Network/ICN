use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;
use serde_json;
use uuid::Uuid;

mod federation;
pub use federation::*;

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

pub struct FederationManager {
    federations: Arc<RwLock<HashMap<String, Federation>>>,
    resource_manager: Arc<dyn ResourceManager>,
    sdp_config: SDPConfig,
}

impl FederationManager {
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
            sdp_config: SDPConfig::default(),
        }
    }

    pub async fn create_federation(
        &self,
        name: String,
        federation_type: FederationType,
        initial_terms: FederationTerms,
        founding_member: String,
    ) -> Result<String, FederationError> {
        let federation_id = format!("fed_{}", Uuid::new_v4());
        let mut members = HashMap::new();
        members.insert(founding_member.clone(), MemberStatus::Active);

        let mut member_roles = HashMap::new();
        member_roles.insert(founding_member, MemberRole::Admin);

        let federation = Federation {
            id: federation_id.clone(),
            name,
            federation_type,
            members,
            member_roles,
            terms: initial_terms,
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: FederationStatus::Active,
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
        };

        let mut federations = self.federations.write().await;
        federations.insert(federation_id.clone(), federation);

        Ok(federation_id)
    }

    pub async fn join_federation(
        &self,
        federation_id: &str,
        member_did: &str,
        commitment: Vec<String>,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            if federation.members.contains_key(member_did) {
                return Err(FederationError::AlreadyMember(member_did.to_string()));
            }

            // Verify member eligibility
            if !self.verify_member_eligibility(member_did).await {
                return Err(FederationError::ValidationError("Member does not meet eligibility requirements".to_string()));
            }

            // Verify commitments
            if !self.verify_commitments(&federation.terms, &commitment).await {
                return Err(FederationError::ValidationError("Invalid commitments".to_string()));
            }

            federation.members.insert(member_did.to_string(), MemberStatus::Active);
            federation.member_roles.insert(member_did.to_string(), MemberRole::Member);

            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    async fn verify_member_eligibility(&self, _member_did: &str) -> bool {
        // Implement actual eligibility verification
        true
    }

    async fn verify_commitments(&self, _terms: &FederationTerms, _commitment: &[String]) -> bool {
        // Implement actual commitment verification
        true
    }

    pub async fn submit_proposal(
        &self,
        federation_id: &str,
        proposal: FederationProposal,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            // Verify proposer is a member
            if !federation.members.contains_key(&proposal.proposer) {
                return Err(FederationError::MemberNotFound(proposal.proposer));
            }

            // Verify proposal type is allowed
            if !federation.terms.governance_rules.allowed_proposal_types.contains(&proposal.proposal_type) {
                return Err(FederationError::InvalidProposal("Proposal type not allowed".to_string()));
            }

            federation.proposals.push(proposal);
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    pub async fn vote(
        &self,
        federation_id: &str,
        vote: Vote,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            // Verify voter is a member
            if !federation.members.contains_key(&vote.voter) {
                return Err(FederationError::MemberNotFound(vote.voter));
            }

            // Find the active proposal
            for proposal in &mut federation.proposals {
                if proposal.status == ProposalStatus::Active {
                    proposal.votes.insert(vote.voter.clone(), vote);
                    return Ok(());
                }
            }

            Err(FederationError::InvalidVote("No active proposal found".to_string()))
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    pub async fn get_federation(&self, federation_id: &str) -> Result<Option<Federation>, FederationError> {
        let federations = self.federations.read().await;
        Ok(federations.get(federation_id).cloned())
    }

    pub async fn list_federations(&self) -> Result<Vec<Federation>, FederationError> {
        let federations = self.federations.read().await;
        Ok(federations.values().cloned().collect())
    }

    pub async fn update_federation_status(
        &self,
        federation_id: &str,
        new_status: FederationStatus,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            federation.status = new_status;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    pub async fn allocate_resources(
        &self,
        federation_id: &str,
        allocation: ResourceAllocationDetails,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            // Verify recipient is a member
            if !federation.members.contains_key(&allocation.recipient) {
                return Err(FederationError::MemberNotFound(allocation.recipient));
            }

            // Verify allocation is within limits
            let pool = federation.resources.get(&allocation.resource_type)
                .ok_or_else(|| FederationError::InsufficientResources("Resource type not available".to_string()))?;

            if pool.available_capacity < allocation.amount {
                return Err(FederationError::InsufficientResources("Not enough resources available".to_string()));
            }

            // Allocate resources
            self.resource_manager.allocate_resources(allocation).await?;

            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }
}
