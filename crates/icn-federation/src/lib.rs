use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use icn_types::{Block, Transaction};
use icn_governance::{DissolutionProtocol, DissolutionReason, DissolutionStatus};
use icn_zkp::RollupBatch;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: String,
    pub name: String,
    pub federation_type: FederationType,
    pub members: HashMap<String, MemberStatus>, // DID -> status
    pub member_roles: HashMap<String, MemberRole>, // DID -> role
    pub terms: FederationTerms,
    pub resources: HashMap<String, ResourcePool>,
    pub proposals: Vec<FederationProposal>, // Add proposals field
    pub created_at: u64,
    pub status: FederationStatus,
    pub disputes: HashMap<String, FederationDispute>, // Add disputes field
    pub cross_federation_disputes: HashMap<String, Vec<FederationDispute>>,
    pub audit_log: Vec<AuditEntry>,
}

impl Federation {
    pub fn add_member(&mut self, did: String, role: MemberRole) -> Result<(), FederationError> {
        if self.members.contains_key(&did) {
            return Err(FederationError::AlreadyMember(did));
        }

        // Verify member meets minimum reputation requirements
        if !self.verify_member_eligibility(&did) {
            return Err(FederationError::InsufficientReputation(
                "Member does not meet minimum reputation requirements".to_string(),
            ));
        }

        self.members.insert(did, MemberStatus::Active);
        Ok(())
    }

    pub fn remove_member(&mut self, did: &str) -> Result<(), FederationError> {
        if !self.members.contains_key(did) {
            return Err(FederationError::MemberNotFound(did.to_string()));
        }

        self.members.remove(did);
        Ok(())
    }

    pub fn get_member_status(&self, did: &str) -> Option<&MemberStatus> {
        self.members.get(did)
    }

    pub fn update_member_status(&mut self, did: &str, status: MemberStatus) -> Result<(), FederationError> {
        if let Some(member_status) = self.members.get_mut(did) {
            *member_status = status;
            Ok(())
        } else {
            Err(FederationError::MemberNotFound(did.to_string()))
        }
    }

    pub fn get_active_members(&self) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, status)| matches!(status, MemberStatus::Active))
            .map(|(did, _)| did.clone())
            .collect()
    }

    pub fn verify_member_eligibility(&self, did: &str) -> bool {
        // This would integrate with the reputation system in practice
        true // Simplified for example
    }

    pub fn submit_proposal(&mut self, proposal: FederationProposal) -> Result<(), FederationError> {
        // Validate proposal
        self.validate_proposal(&proposal)?;

        // Set proposal voting period
        let mut proposal = proposal;
        proposal.voting_ends_at = chrono::Utc::now().timestamp() as u64 + 
            (self.terms.governance_rules.max_voting_period_hours * 3600);

        self.proposals.push(proposal);
        Ok(())
    }

    pub fn vote(&mut self, vote: Vote) -> Result<(), FederationError> {
        // Validate vote
        self.validate_vote(&vote)?;

        // Get proposal
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == vote.proposal_id)
            .ok_or(FederationError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Record vote
        proposal.votes.insert(vote.voter, vote.approve);

        // Check if voting period ended and finalize if needed
        let now = chrono::Utc::now().timestamp() as u64;
        if now > proposal.voting_ends_at {
            proposal.status = self.finalize_proposal(&proposal.id)?;
        }

        Ok(())
    }

    pub fn validate_proposal(&self, proposal: &FederationProposal) -> Result<(), FederationError> {
        // Check if proposal type is allowed
        if !self.terms.governance_rules.allowed_proposal_types.contains(&proposal.proposal_type.to_string()) {
            return Err(FederationError::InvalidProposalType(proposal.proposal_type.to_string()));
        }

        // Validate proposer has sufficient reputation
        if !self.verify_member_eligibility(&proposal.proposer) {
            return Err(FederationError::InsufficientReputation(
                "Proposer does not meet minimum reputation requirements".to_string()
            ));
        }

        Ok(())
    }

    pub fn validate_vote(&self, vote: &Vote) -> Result<(), FederationError> {
        // Check if voter is a member
        if !self.members.contains_key(&vote.voter) {
            return Err(FederationError::UnauthorizedAction { action: "vote".to_string(), did: vote.voter.clone() });
        }

        // Check if proposal exists
        let proposal = self.proposals.iter()
            .find(|p| p.id == vote.proposal_id)
            .ok_or(FederationError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Check if voting period is still open
        let now = chrono::Utc::now().timestamp() as u64;
        if now > proposal.voting_ends_at {
            return Err(FederationError::VotingPeriodEnded(vote.proposal_id.clone()));
        }

        // Check for veto rights
        if let Some(member_role) = self.member_roles.get(&vote.voter) {
            if let Some(veto_actions) = self.terms.governance_rules.veto_rights.get(&member_role.to_string()) {
                if veto_actions.contains(&proposal.proposal_type.to_string()) && !vote.approve {
                    // Record veto
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<ProposalStatus, FederationError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(FederationError::ProposalNotFound(proposal_id.to_string()))?;

        // Create vote batch for on-chain processing
        let batch = RollupBatch {
            proposal_id: proposal_id.to_string(),
            votes: proposal.votes.iter()
                .map(|(voter, approve)| Vote { voter: voter.clone(), approve: *approve })
                .collect(),
            rollup_root: [0u8; 32], // Computed by ZK prover
            proof: Vec::new(), // Generated by ZK prover
        };

        // Submit batch to chain
        self.contract.submit_vote_batch(batch)?;

        // Execute proposal on-chain
        let approved = self.contract.execute_proposal(proposal_id)?;
        
        proposal.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        Ok(proposal.status.clone())
    }

    pub fn calculate_asset_distribution(&self) -> HashMap<String, AssetAllocation> {
        let mut distributions = HashMap::new();
        // Implement fair asset distribution calculation
        distributions
    }

    pub fn settle_outstanding_debts(&self) -> Vec<DebtSettlement> {
        let mut settlements = Vec::new();
        // Implement debt settlement calculation
        settlements
    }

    pub fn reassign_members(&self) -> Vec<MemberReassignment> {
        let mut reassignments = Vec::new();
        // Implement member reassignment logic
        reassignments
    }

    pub fn calculate_vote_weight(&self, cooperative_id: &str, proposal: &FederationProposal) -> f64 {
        let voting_model = match proposal.proposal_type {
            ProposalType::GovernanceChange(_) | ProposalType::PolicyUpdate(_) => 
                &self.terms.governance_rules.governance_voting_model,
            ProposalType::ResourceAllocation(_) =>
                &self.terms.governance_rules.resource_voting_model,
            _ => &self.terms.governance_rules.default_voting_model,
        };

        voting_model.calculate_voting_power(self, cooperative_id)
    }

    pub fn get_cooperative_weight(&self, cooperative_id: &str) -> f64 {
        let total_members: u32 = self.members.values().map(|m| m.member_count).sum();
        let coop_members = self.members.get(cooperative_id)
            .map(|m| m.member_count)
            .unwrap_or(0);
        
        coop_members as f64 / total_members as f64
    }

    pub fn initiate_dissolution(&mut self, initiator: String, reason: String) -> Result<DissolutionProtocol, FederationError> {
        let protocol = DissolutionProtocol {
            federation_id: self.id.clone(),
            initiated_by: initiator,
            reason: DissolutionReason::Voluntary,
            status: DissolutionStatus::Initiated,
            asset_distribution: HashMap::new(),
            debt_settlements: Vec::new(),
            member_reassignments: Vec::new(),
            dispute_period_ends: SystemTime::now() + Duration::from_secs(7 * 24 * 60 * 60), // 7 days
        };

        self.status = FederationStatus::DisputePeriod;
        Ok(protocol)
    }

    pub fn submit_dissolution_dispute(&mut self, dispute: FederationDispute) -> Result<(), FederationError> {
        if self.status != FederationStatus::DisputePeriod {
            return Err(FederationError::InvalidStatusTransition { from: "DisputePeriod".to_string(), to: self.status.to_string() });
        }

        if !self.members.contains_key(&dispute.initiator) {
            return Err(FederationError::UnauthorizedAction { action: "submit_dissolution_dispute".to_string(), did: dispute.initiator.clone() });
        }

        self.disputes.insert(dispute.id.clone(), dispute);
        self.status = FederationStatus::DisputeResolution;
        Ok(())
    }

    pub fn vote_on_dispute(&mut self, dispute_id: &str, voter: String, support: bool) -> Result<(), FederationError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or(FederationError::DisputeNotFound(dispute_id.to_string()))?;

        if !self.members.contains_key(&voter) {
            return Err(FederationError::UnauthorizedAction { action: "vote_on_dispute".to_string(), did: voter.clone() });
        }

        dispute.supporting_votes.insert(voter, support);

        // Check if we have enough votes to resolve the dispute
        let total_votes = dispute.supporting_votes.len();
        let supporting_votes = dispute.supporting_votes.values().filter(|&&v| v).count();
        let required_votes = (self.members.len() * 2) / 3; // 2/3 majority

        if total_votes >= required_votes {
            if supporting_votes > total_votes / 2 {
                dispute.status = DisputeStatus::Resolved;
                self.status = FederationStatus::Active;
            } else {
                dispute.status = DisputeStatus::Rejected;
                self.status = FederationStatus::Dissolved;
            }
        }

        Ok(())
    }

    pub fn resolve_dispute(&mut self, dispute_id: &str, resolution: DisputeResolution) -> Result<(), FederationError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or(FederationError::DisputeNotFound(dispute_id.to_string()))?;

        dispute.resolution = Some(resolution);
        dispute.status = DisputeStatus::Resolved;

        // If all disputes are resolved, proceed with dissolution
        if self.disputes.values().all(|d| d.status == DisputeStatus::Resolved || d.status == DisputeStatus::Rejected) {
            let any_upheld = self.disputes.values().any(|d| d.status == DisputeStatus::Resolved);
            self.status = if any_upheld {
                FederationStatus::Active
            } else {
                FederationStatus::Dissolved
            };
        }

        Ok(())
    }

    pub async fn initiate_cross_federation_dispute(
        &mut self,
        target_federation: &str,
        dispute: FederationDispute
    ) -> Result<(), FederationError> {
        // Verify both federations exist and have sufficient reputation
        self.verify_cross_federation_eligibility(target_federation).await?;
        
        let disputes = self.cross_federation_disputes
            .entry(target_federation.to_string())
            .or_insert_with(Vec::new);
        
        disputes.push(dispute);
        
        // Log dispute for audit
        self.audit_log.push(AuditEntry {
            action: "cross_federation_dispute".into(),
            target_federation: Some(target_federation.to_string()),
            timestamp: chrono::Utc::now(),
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub resource_type: String,
    pub total_amount: u64,
    pub available_amount: u64,
    pub contributors: HashMap<String, u64>, // DID -> amount contributed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationProposal {
    pub id: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub description: String,
    pub votes: HashMap<String, bool>, // DID -> vote
    pub status: ProposalStatus,
    pub created_at: u64,
    pub voting_ends_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    AddMember(String),
    RemoveMember(String),
    UpdateTerms(FederationTerms),
    AllocateResources(ResourceAllocation),
    UpdatePolicy(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub resource_type: String,
    pub amount: u64,
    pub recipient: String,
    pub duration: Option<u64>,
}

pub struct FederationManager {
    federations: Arc<RwLock<HashMap<String, Federation>>>,
    resource_manager: Arc<dyn ResourceManager>,
}

impl FederationManager {
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
        }
    }

    pub async fn create_federation(
        &self,
        name: String,
        federation_type: FederationType,
        initial_terms: FederationTerms,
        founding_member: String,
    ) -> Result<String, FederationError> {
        let federation_id = format!("fed_{}", uuid::Uuid::new_v4());
        let federation = Federation {
            id: federation_id.clone(),
            name,
            federation_type,
            members: vec![founding_member].into_iter().map(|m| (m, MemberStatus::Active)).collect(),
            member_roles: HashMap::new(),
            terms: initial_terms,
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
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

            // Verify commitments against federation terms
            if !self.verify_commitments(&federation.terms, &commitment).await {
                return Err(FederationError::InvalidCommitment(member_did.to_string()));
            }

            federation.members.insert(member_did.to_string(), MemberStatus::Active);
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    async fn verify_commitments(&self, terms: &FederationTerms, commitment: &[String]) -> bool {
        // Add commitment verification logic here
        true // Placeholder
    }

    pub async fn submit_proposal(
        &self,
        federation_id: &str,
        proposal: FederationProposal,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;

        if let Some(federation) = federations.get_mut(federation_id) {
            federation.submit_proposal(proposal)?;
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
            federation.vote(vote)?;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }
}

#[async_trait]
pub trait ResourceManager: Send + Sync {
    async fn allocate_resources(&self, allocation: ResourceAllocation) -> Result<(), String>;
    async fn release_resources(&self, resource_type: &str, amount: u64) -> Result<(), String>;
}

pub trait FederationDissolution {
    fn initiate_dissolution(&mut self, initiator: &str, reason: DissolutionReason) -> Result<DissolutionProtocol, Error>;
    fn process_dissolution(&mut self, protocol: &DissolutionProtocol) -> Result<DissolutionStatus, Error>;
    fn cancel_dissolution(&mut self, protocol_id: &str) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationType {
    Cooperative,
    Community,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTerms {
    pub minimum_reputation: i64,
    pub resource_sharing_policies: String,
    pub governance_rules: GovernanceRules,
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRules {
    pub min_votes_required: u32,
    pub approval_threshold_percent: u32,
    pub min_voting_period_hours: u32,
    pub max_voting_period_hours: u32,
    pub allowed_proposal_types: Vec<String>,
    pub veto_rights: HashMap<String, Vec<String>>, // role -> action types that can be vetoed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationStatus {
    Active,
    Suspended,
    Dissolved,
    DisputePeriod,
    DisputeResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberRole {
    Admin,
    Member,
    Observer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDispute {
    pub id: String,
    pub federation_id: String,
    pub initiator: String,
    pub reason: String,
    pub evidence: Option<String>,
    pub supporting_votes: HashMap<String, bool>,
    pub created_at: u64,
    pub status: DisputeStatus,
    pub resolution: Option<DisputeResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub decision: String,
    pub rationale: String,
    pub resolved_at: u64,
    pub resolver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeStatus {
    Pending,
    Resolved,
    Rejected,
}

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Already a member: {0}")]
    AlreadyMember(String),
    
    #[error("Invalid commitment: {0}")]
    InvalidCommitment(String),
    
    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },
    
    #[error("Unauthorized action: {action} by {did}")]
    UnauthorizedAction { action: String, did: String },
    
    #[error("Member not found: {0}")]
    MemberNotFound(String),
    
    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Invalid proposal type: {0}")] 
    InvalidProposalType(String),
    
    #[error("Voting period ended for proposal {0}")]
    VotingPeriodEnded(String),
    
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    
    #[error("Insufficient reputation: {0}")]
    InsufficientReputation(String),
    
    #[error("Dispute not found: {0}")]
    DisputeNotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Consensus error: {0}")]
    ConsensusError(#[from] ConsensusError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub action: String,
    pub target_federation: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
