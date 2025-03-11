use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};
use thiserror::Error;
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use icn_types::{FederationId, CooperativeId, MemberId};
use crate::resource_manager::ResourceProvider;

/// A federation representing a network of cooperative organizations
#[derive(Clone, Serialize, Deserialize)]
pub struct Federation {
    /// Unique Federation ID
    pub id: FederationId,
    /// Name of the federation
    pub name: String,
    /// Description of the federation
    pub description: String,
    /// Federation founding date
    pub founded_date: DateTime<Utc>,
    /// Set of member IDs 
    pub members: HashSet<MemberId>,
    /// Resource manager for this federation
    #[serde(skip)]
    pub resource_manager: Option<Arc<dyn ResourceProvider>>,
    /// Key-value metadata storage
    pub metadata: HashMap<String, String>,
    
    /// Type of federation
    pub federation_type: FederationType,
    
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

impl std::fmt::Debug for Federation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Federation")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("founded_date", &self.founded_date)
            .field("members", &self.members)
            .field("resource_manager", &"<ResourceProvider>")
            .field("metadata", &self.metadata)
            .field("federation_type", &self.federation_type)
            .field("member_roles", &self.member_roles)
            .field("terms", &self.terms)
            .field("resources", &self.resources)
            .field("proposals", &self.proposals)
            .field("created_at", &self.created_at)
            .field("status", &self.status)
            .field("disputes", &self.disputes)
            .field("cross_federation_disputes", &self.cross_federation_disputes)
            .field("audit_log", &self.audit_log)
            .finish()
    }
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

impl Default for FederationTerms {
    fn default() -> Self {
        Self {
            governance_rules: GovernanceRules::default(),
            resource_rules: ResourceRules::default(),
            membership_rules: MembershipRules::default(),
            dispute_resolution_rules: DisputeResolutionRules::default(),
            cross_federation_rules: CrossFederationRules::default(),
        }
    }
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

impl Default for GovernanceRules {
    fn default() -> Self {
        Self {
            min_votes_required: 3,
            approval_threshold_percent: 66,
            min_voting_period_hours: 24,
            max_voting_period_hours: 168, // 1 week
            allowed_proposal_types: vec![
                ProposalType::MembershipChange(MembershipAction::Add("".to_string())),
                ProposalType::ResourceAllocation(ResourceAllocationDetails::default()),
                ProposalType::GovernanceUpdate(GovernanceUpdateDetails::default()),
                ProposalType::FederationTermsUpdate(FederationTermsUpdateDetails::default()),
            ],
            veto_rights: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRules {
    pub min_contribution: u64,
    pub max_allocation_per_member: u64,
    pub allocation_strategy: AllocationStrategy,
    pub resource_types: Vec<ResourceType>,
    pub sharing_policies: Vec<SharingPolicy>,
}

impl Default for ResourceRules {
    fn default() -> Self {
        Self {
            min_contribution: 0,
            max_allocation_per_member: 1000,
            allocation_strategy: AllocationStrategy::EqualShare,
            resource_types: vec![ResourceType::ComputeUnit, ResourceType::StorageGb, ResourceType::BandwidthMbps],
            sharing_policies: vec![SharingPolicy::MembersOnly],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipRules {
    pub min_reputation_score: f64,
    pub max_members: u32,
    pub membership_duration: Option<Duration>,
    pub required_roles: Vec<MemberRole>,
    pub onboarding_process: OnboardingProcess,
}

impl Default for MembershipRules {
    fn default() -> Self {
        Self {
            min_reputation_score: 0.0,
            max_members: 100,
            membership_duration: None,
            required_roles: vec![MemberRole::Member],
            onboarding_process: OnboardingProcess::VotingRequired,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolutionRules {
    pub resolution_time_limit_hours: u32,
    pub min_arbitrators: u32,
    pub arbitrator_selection: ArbitratorSelection,
    pub appeal_process: AppealProcess,
}

impl Default for DisputeResolutionRules {
    fn default() -> Self {
        Self {
            resolution_time_limit_hours: 72,
            min_arbitrators: 3,
            arbitrator_selection: ArbitratorSelection::Reputation,
            appeal_process: AppealProcess::SingleLevel,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFederationRules {
    pub allowed_federation_types: Vec<FederationType>,
    pub resource_sharing_limits: HashMap<ResourceType, u64>,
    pub min_reputation_requirement: f64,
    pub governance_participation: GovernanceParticipation,
}

impl Default for CrossFederationRules {
    fn default() -> Self {
        Self {
            allowed_federation_types: vec![FederationType::ResourceSharing, FederationType::Governance],
            resource_sharing_limits: HashMap::new(),
            min_reputation_requirement: 50.0,
            governance_participation: GovernanceParticipation::ReadOnly,
        }
    }
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
    Founder,
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

impl Default for FederationStatus {
    fn default() -> Self {
        Self::Active
    }
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

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::ComputeUnit => write!(f, "ComputeUnit"),
            ResourceType::StorageGb => write!(f, "StorageGb"),
            ResourceType::BandwidthMbps => write!(f, "BandwidthMbps"),
            ResourceType::MemoryGb => write!(f, "MemoryGb"),
            ResourceType::CustomResource(name) => write!(f, "CustomResource({})", name),
        }
    }
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

impl Default for ResourceAllocationDetails {
    fn default() -> Self {
        Self {
            resource_type: ResourceType::ComputeUnit,
            member_id: String::new(),
            amount: 0,
            duration: 86400, // 1 day in seconds
            details: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceUpdateDetails {
    pub parameters: HashMap<String, String>,
    pub reason: String,
}

impl Default for GovernanceUpdateDetails {
    fn default() -> Self {
        Self {
            parameters: HashMap::new(),
            reason: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationTermsUpdateDetails {
    pub section: String,
    pub changes: HashMap<String, String>,
}

impl Default for FederationTermsUpdateDetails {
    fn default() -> Self {
        Self {
            section: String::new(),
            changes: HashMap::new(),
        }
    }
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
    
    #[error("Member is already in the federation: {0}")]
    AlreadyMember(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    #[error("Resource manager error: {0}")]
    ResourceError(String),
    
    #[error("Resource manager not configured")]
    ResourceManagerNotConfigured,
    
    #[error("Governance manager not configured")]
    GovernanceManagerNotConfigured,
    
    #[error("Dispute manager not configured")]
    DisputeManagerNotConfigured,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
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
            MembershipAction::Add(member_id_str) => {
                let member_id = MemberId { 
                    did: member_id_str.clone(), 
                    cooperative_id: CooperativeId("default".to_string()) 
                };
                
                if self.members.contains(&member_id) {
                    return Err(FederationError::InvalidOperation(
                        format!("Member {:?} already exists", member_id)
                    ));
                }
                
                self.members.insert(member_id.clone());
                self.member_roles.insert(member_id_str, vec![MemberRole::Member]);
                
                Ok(())
            }
            MembershipAction::Remove(member_id_str) => {
                let member_id = MemberId { 
                    did: member_id_str.clone(), 
                    cooperative_id: CooperativeId("default".to_string()) 
                };
                if !self.members.contains(&member_id) {
                    return Err(FederationError::InvalidOperation(
                        format!("Member {:?} does not exist", member_id)
                    ));
                }
                
                self.members.remove(&member_id);
                self.member_roles.remove(&member_id_str);
                
                Ok(())
            },
            MembershipAction::ChangeRole(member_id_str, roles) => {
                let member_id = MemberId { 
                    did: member_id_str.clone(), 
                    cooperative_id: CooperativeId("default".to_string()) 
                };
                if !self.members.contains(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id.did));
                }
                
                self.member_roles.insert(member_id_str, roles);
                
                Ok(())
            },
            MembershipAction::Suspend(member_id_str, duration) => {
                let member_id = MemberId { 
                    did: member_id_str.clone(), 
                    cooperative_id: CooperativeId("default".to_string()) 
                };
                if !self.members.contains(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id.did));
                }
                
                // We can't use get_mut on HashSet, so we need to update the member status differently
                // For now, we'll just add a note in the audit log
                self.add_audit_log_entry(
                    "MemberSuspended", 
                    format!("Member {} suspended for {} seconds", member_id_str, duration)
                );
                
                Ok(())
            },
            MembershipAction::Reinstate(member_id_str) => {
                let member_id = MemberId { 
                    did: member_id_str.clone(), 
                    cooperative_id: CooperativeId("default".to_string()) 
                };
                if !self.members.contains(&member_id) {
                    return Err(FederationError::MemberNotFound(member_id.did));
                }
                
                // We can't use get_mut on HashSet, so we need to update the member status differently
                // For now, we'll just add a note in the audit log
                self.add_audit_log_entry(
                    "MemberReinstated", 
                    format!("Member {} reinstated", member_id_str)
                );
                
                Ok(())
            }
        }
    }
    
    pub fn allocate_resource(&mut self, details: ResourceAllocationDetails) -> Result<(), FederationError> {
        // Check if member exists
        let member_id = MemberId { 
            did: details.member_id.clone(), 
            cooperative_id: CooperativeId("default".to_string()) 
        };
        if !self.members.contains(&member_id) {
            return Err(FederationError::InvalidOperation(
                format!("Member {:?} does not exist in federation", member_id)
            ));
        }
        
        // Get or create resource pool
        let pool = self.resources.entry(details.resource_type.clone())
            .or_insert_with(|| ResourcePool {
                resource_type: details.resource_type.clone(),
                total_capacity: details.amount,
                available_capacity: details.amount,
                allocations: HashMap::new(),
                sharing_policy: SharingPolicy::MembersOnly,
            });
        
        // Check if there are enough resources
        if pool.available_capacity < details.amount {
            return Err(FederationError::InsufficientResources(
                format!("Not enough {} resources: requested {}, available {}", 
                    details.resource_type.to_string(), details.amount, pool.available_capacity)
            ));
        }
        
        // Allocate resources
        let current = pool.allocations.entry(details.member_id.clone()).or_insert(0);
        *current += details.amount;
        pool.available_capacity -= details.amount;
        
        // Add audit log
        self.add_audit_log_entry(
            "ResourceAllocation",
            format!("Allocated {} {} to member {}", 
                details.amount, details.resource_type.to_string(), details.member_id)
        );
        
        Ok(())
    }
    
    pub fn update_governance(&mut self, details: GovernanceUpdateDetails) -> Result<(), FederationError> {
        // Instead of directly inserting into a map, we'll update the relevant fields
        for (key, value) in details.parameters {
            match key.as_str() {
                "min_votes_required" => {
                    if let Ok(val) = value.parse::<u32>() {
                        self.terms.governance_rules.min_votes_required = val;
                    }
                },
                "approval_threshold_percent" => {
                    if let Ok(val) = value.parse::<u8>() {
                        self.terms.governance_rules.approval_threshold_percent = val;
                    }
                },
                "min_voting_period_hours" => {
                    if let Ok(val) = value.parse::<u32>() {
                        self.terms.governance_rules.min_voting_period_hours = val;
                    }
                },
                "max_voting_period_hours" => {
                    if let Ok(val) = value.parse::<u32>() {
                        self.terms.governance_rules.max_voting_period_hours = val;
                    }
                },
                _ => {
                    // Ignore unknown parameters
                }
            }
        }
        
        self.add_audit_log_entry(
            "governance_update",
            format!("Governance updated: {}", details.reason)
        );
        
        Ok(())
    }
    
    pub fn update_terms(&mut self, terms_update: FederationTermsUpdateDetails) -> Result<(), FederationError> {
        // Apply specific changes based on the section
        match terms_update.section.as_str() {
            "governance" => {
                for (key, value) in terms_update.changes {
                    self.update_governance_rules(key, value)?;
                }
            },
            "resources" => {
                // Update resource rules based on changes
                for (key, value) in terms_update.changes {
                    match key.as_str() {
                        "min_contribution" => {
                            if let Ok(val) = value.parse::<u64>() {
                                self.terms.resource_rules.min_contribution = val;
                            }
                        },
                        "max_allocation" => {
                            if let Ok(val) = value.parse::<u64>() {
                                self.terms.resource_rules.max_allocation_per_member = val;
                            }
                        },
                        // Add more fields as needed
                        _ => {}
                    }
                }
            },
            "membership" => {
                // Update membership rules based on changes
                for (key, value) in terms_update.changes {
                    match key.as_str() {
                        "min_reputation" => {
                            if let Ok(val) = value.parse::<f64>() {
                                self.terms.membership_rules.min_reputation_score = val;
                            }
                        },
                        "max_members" => {
                            if let Ok(val) = value.parse::<u32>() {
                                self.terms.membership_rules.max_members = val;
                            }
                        },
                        // Add more fields as needed
                        _ => {}
                    }
                }
            },
            _ => {
                return Err(FederationError::InvalidOperation(
                    format!("Unknown terms section: {}", terms_update.section)
                ));
            }
        }
        
        self.add_audit_log_entry("terms_update", format!("Updated terms section: {}", terms_update.section));
        Ok(())
    }
    
    pub fn update_governance_rules(&mut self, key: String, value: String) -> Result<(), FederationError> {
        let mut params = HashMap::new();
        params.insert(key, value);
        self.update_governance(GovernanceUpdateDetails {
            parameters: params,
            reason: "Governance rules update".to_string(),
        })
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