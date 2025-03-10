use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    /// Unique federation ID
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Type of federation
    pub federation_type: FederationType,
    
    /// Members of the federation
    pub members: HashMap<String, MemberInfo>,
    
    /// Member roles
    pub member_roles: HashMap<String, Vec<MemberRole>>,
    
    /// Federation terms
    pub terms: FederationTerms,
    
    /// Shared resources
    pub resources: HashMap<ResourceType, ResourcePool>,
    
    /// Governance proposals
    pub proposals: Vec<ProposalReference>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Current status
    pub status: FederationStatus,
    
    /// Internal disputes
    pub disputes: HashMap<String, DisputeReference>,
    
    /// Cross-federation disputes
    pub cross_federation_disputes: HashMap<String, CrossFederationDisputeReference>,
    
    /// Audit log
    pub audit_log: Vec<AuditEntry>,
}

/// Reference to a governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalReference {
    /// Proposal ID
    pub id: String,
    
    /// Proposal title
    pub title: String,
    
    /// Proposal status
    pub status: ProposalStatus,
    
    /// Creation timestamp
    pub created_at: u64,
}

/// Reference to a dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeReference {
    /// Dispute ID
    pub id: String,
    
    /// Dispute title
    pub title: String,
    
    /// Dispute status
    pub status: String,
    
    /// Complainant member ID
    pub complainant: String,
    
    /// Respondent member IDs
    pub respondents: Vec<String>,
    
    /// Creation timestamp
    pub created_at: u64,
}

/// Reference to a cross-federation dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFederationDisputeReference {
    /// Dispute ID
    pub id: String,
    
    /// Dispute title
    pub title: String,
    
    /// Dispute status
    pub status: String,
    
    /// Other federation ID
    pub other_federation_id: String,
    
    /// Creation timestamp
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTerms {
    pub governance_rules: GovernanceRules,
    pub resource_rules: ResourceRules,
    pub membership_rules: MembershipRules,
    pub dispute_resolution_rules: DisputeResolutionRules,
    pub cross_federation_rules: CrossFederationRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRules {
    pub min_votes_required: u32,
    pub approval_threshold_percent: u8,
    pub min_voting_period_hours: u32,
    pub max_voting_period_hours: u32,
    pub allowed_proposal_types: Vec<ProposalType>,
    pub veto_rights: HashMap<String, Vec<ProposalType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRules {
    pub min_contribution: u64,
    pub max_allocation_per_member: u64,
    pub allocation_strategy: AllocationStrategy,
    pub resource_types: Vec<ResourceType>,
    pub sharing_policies: Vec<SharingPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipRules {
    pub min_reputation_score: f64,
    pub max_members: u32,
    pub membership_duration: Option<Duration>,
    pub required_roles: Vec<MemberRole>,
    pub onboarding_process: OnboardingProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolutionRules {
    pub resolution_time_limit_hours: u32,
    pub min_arbitrators: u32,
    pub arbitrator_selection: ArbitratorSelection,
    pub appeal_process: AppealProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFederationRules {
    pub allowed_federation_types: Vec<FederationType>,
    pub resource_sharing_limits: HashMap<ResourceType, u64>,
    pub min_reputation_requirement: f64,
    pub governance_participation: GovernanceParticipation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationType {
    ResourceSharing,
    Governance,
    Research,
    Development,
    Hybrid,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberRole {
    Admin,
    Member,
    Observer,
    Arbitrator,
    ResourceProvider,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationStatus {
    Active,
    Inactive,
    Suspended,
    Dissolving,
    Dissolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    EqualShare,
    ProportionalToContribution,
    NeedsBasedPriority,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    ComputeUnit,
    StorageGb,
    BandwidthMbps,
    MemoryGb,
    CustomResource(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingPolicy {
    Unrestricted,
    MembersOnly,
    ApprovalRequired,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingProcess {
    Immediate,
    VotingRequired,
    InvitationOnly,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitratorSelection {
    Random,
    Reputation,
    Voting,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppealProcess {
    None,
    SingleLevel,
    MultiLevel,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceParticipation {
    None,
    ReadOnly,
    ProposalOnly,
    FullParticipation,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub resource_type: ResourceType,
    pub total_capacity: u64,
    pub available_capacity: u64,
    pub allocations: HashMap<String, u64>,
    pub sharing_policy: SharingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub votes: HashMap<String, Vote>,
    pub created_at: u64,
    pub voting_ends_at: u64,
    pub execution_status: Option<ExecutionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    MembershipChange(MembershipAction),
    ResourceAllocation(ResourceAllocationDetails),
    GovernanceUpdate(GovernanceUpdateDetails),
    FederationTermsUpdate(FederationTermsUpdateDetails),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MembershipAction {
    Add(String),
    Remove(String),
    ChangeRole(String, Vec<MemberRole>),
    Suspend(String, u64),
    Reinstate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceAllocationDetails {
    pub resource_type: ResourceType,
    pub member_id: String,
    pub amount: u64,
    pub duration: u64,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceUpdateDetails {
    pub parameters: HashMap<String, String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationTermsUpdateDetails {
    pub section: String,
    pub changes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub decision: VoteDecision,
    pub timestamp: u64,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDispute {
    pub id: String,
    pub initiator: String,
    pub respondent: String,
    pub dispute_type: DisputeType,
    pub evidence: Vec<Evidence>,
    pub status: DisputeStatus,
    pub resolution: Option<DisputeResolution>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeType {
    ResourceAllocation,
    Governance,
    Membership,
    TermsViolation,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub evidence_type: String,
    pub content: String,
    pub submitter: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeStatus {
    Filed,
    UnderReview,
    Resolved,
    Appealed,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub decision: DisputeDecision,
    pub rationale: String,
    pub arbitrators: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeDecision {
    Upheld,
    Rejected,
    Compromise(String),
}

/// Audit log entry for federation activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of the entry
    pub timestamp: u64,
    
    /// Type of event
    pub event_type: String,
    
    /// Description of the event
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub id: String,
    pub resource_type: ResourceType,
    pub amount: u64,
    pub allocated_at: u64,
    pub expires_at: Option<u64>,
    pub details: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Member not found: {0}")]
    MemberNotFound(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    
    #[error("Dispute not found: {0}")]
    DisputeNotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Governance error: {0}")]
    GovernanceError(String),
    
    #[error("Dispute resolution error: {0}")]
    DisputeError(String),
    
    #[error("Networking error: {0}")]
    NetworkingError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    /// Member status
    pub status: MemberStatus,
    
    /// Join timestamp
    pub joined_at: u64,
    
    /// Last active timestamp
    pub last_active: u64,
    
    /// Resource allocations
    pub resource_allocations: Vec<ResourceAllocation>,
    
    /// When suspension ends (if suspended)
    pub suspension_end: Option<u64>,
    
    /// Member metadata
    pub metadata: HashMap<String, String>,
}

impl Default for MemberInfo {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            status: MemberStatus::Active,
            joined_at: now,
            last_active: now,
            resource_allocations: Vec::new(),
            suspension_end: None,
            metadata: HashMap::new(),
        }
    }
}

impl Federation {
    pub fn apply_membership_action(&mut self, action: MembershipAction) -> Result<(), FederationError> {
        match action {
            MembershipAction::Add(member_id) => {
                if self.members.contains_key(&member_id) {
                    return Err(FederationError::InvalidOperation(
                        format!("Member {} already exists", member_id)
                    ));
                }
                
                self.members.insert(member_id.clone(), Default::default());
                self.member_roles.insert(member_id, vec![MemberRole::Member]);
                
                Ok(())
            },
            MembershipAction::Remove(member_id) => {
                if !self.members.contains_key(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id));
                }
                
                self.members.remove(&member_id);
                self.member_roles.remove(&member_id);
                
                Ok(())
            },
            MembershipAction::ChangeRole(member_id, roles) => {
                if !self.members.contains_key(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id));
                }
                
                self.member_roles.insert(member_id, roles);
                
                Ok(())
            },
            MembershipAction::Suspend(member_id, duration) => {
                if !self.members.contains_key(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id));
                }
                
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                let member_info = self.members.get_mut(&member_id).unwrap();
                member_info.status = MemberStatus::Suspended;
                member_info.suspension_end = Some(now + duration);
                
                Ok(())
            },
            MembershipAction::Reinstate(member_id) => {
                if !self.members.contains_key(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id));
                }
                
                let member_info = self.members.get_mut(&member_id).unwrap();
                member_info.status = MemberStatus::Active;
                member_info.suspension_end = None;
                
                Ok(())
            },
        }
    }
    
    pub fn allocate_resource(&mut self, details: ResourceAllocationDetails) -> Result<(), FederationError> {
        if !self.members.contains_key(&details.member_id) {
            return Err(FederationError::MemberNotFound(details.member_id));
        }
        
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let allocation = ResourceAllocation {
            id: Uuid::new_v4().to_string(),
            resource_type: details.resource_type.clone(),
            amount: details.amount,
            allocated_at: now,
            expires_at: if details.duration > 0 { Some(now + details.duration) } else { None },
            details: details.details,
        };
        
        let member_info = self.members.get_mut(&details.member_id).unwrap();
        member_info.resource_allocations.push(allocation);
        
        Ok(())
    }
    
    pub fn update_governance(&mut self, details: GovernanceUpdateDetails) -> Result<(), FederationError> {
        for (key, value) in details.parameters {
            self.terms.governance_params.insert(key, value);
        }
        
        self.add_audit_log_entry(
            "governance_update",
            format!("Governance parameters updated: {}", details.reason)
        );
        
        Ok(())
    }
    
    pub fn update_terms(&mut self, terms: FederationTerms) -> Result<(), FederationError> {
        self.terms = terms;
        
        self.add_audit_log_entry(
            "terms_update",
            "Federation terms updated".to_string()
        );
        
        Ok(())
    }
    
    fn add_audit_log_entry(&mut self, event_type: &str, description: String) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let entry = AuditEntry {
            timestamp: now,
            event_type: event_type.to_string(),
            description,
        };
        
        self.audit_log.push(entry);
        
        if self.audit_log.len() > 1000 {
            self.audit_log.remove(0);
        }
    }
} 