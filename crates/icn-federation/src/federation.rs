use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: String,
    pub name: String,
    pub federation_type: FederationType,
    pub members: HashMap<String, MemberStatus>,
    pub member_roles: HashMap<String, MemberRole>,
    pub terms: FederationTerms,
    pub resources: HashMap<ResourceType, ResourcePool>,
    pub proposals: Vec<FederationProposal>,
    pub created_at: u64,
    pub status: FederationStatus,
    pub disputes: HashMap<String, FederationDispute>,
    pub cross_federation_disputes: HashMap<String, Vec<FederationDispute>>,
    pub audit_log: Vec<AuditEntry>,
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
    Compute,
    Storage,
    Network,
    Memory,
    Custom(String),
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
    ChangeRole { member: String, new_role: MemberRole },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceAllocationDetails {
    pub resource_type: ResourceType,
    pub amount: u64,
    pub recipient: String,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceUpdateDetails {
    pub field: String,
    pub new_value: String,
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
    Failed,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub action_type: AuditActionType,
    pub actor: String,
    pub target: String,
    pub details: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditActionType {
    MembershipChange,
    ResourceAllocation,
    ProposalSubmission,
    Vote,
    DisputeResolution,
    TermsUpdate,
    Custom(String),
}

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Member already exists: {0}")]
    AlreadyMember(String),
    
    #[error("Member not found: {0}")]
    MemberNotFound(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    
    #[error("Invalid dispute: {0}")]
    InvalidDispute(String),
    
    #[error("Communication error: {0}")]
    CommunicationError(String),
    
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
} 