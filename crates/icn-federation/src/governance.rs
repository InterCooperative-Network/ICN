use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;
use log::{debug, info, warn, error};

use icn_types::FederationId;
use icn_reputation::ReputationInterface;
use icn_crypto::KeyPair;

use crate::federation::{
    Federation, FederationType, FederationTerms, FederationError,
    ProposalType, ProposalStatus, Vote, VoteDecision, MemberRole,
};
use icn_types::{MemberId, CooperativeId};

/// Error types for federation governance
#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    
    #[error("Member not found: {0}")]
    MemberNotFound(String),
    
    #[error("Member not authorized: {0}")]
    Unauthorized(String),
    
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    
    #[error("Voting period ended")]
    VotingPeriodEnded,
    
    #[error("Quorum not reached")]
    QuorumNotReached,
    
    #[error("Federation error: {0}")]
    FederationError(#[from] FederationError),
    
    #[error("Reputation service error: {0}")]
    ReputationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type GovernanceResult<T> = Result<T, GovernanceError>;

/// Voting power allocation strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VotingStrategy {
    /// One member, one vote (democratic)
    EqualVoting,
    
    /// Voting power proportional to reputation
    ReputationWeighted,
    
    /// Voting power with quadratic scaling (sqrt of reputation)
    QuadraticVoting,
    
    /// Voting power based on resource contribution
    ResourceWeighted,
}

impl Default for VotingStrategy {
    fn default() -> Self {
        Self::ReputationWeighted
    }
}

/// Configuration for federation governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Voting strategy to use
    pub voting_strategy: VotingStrategy,
    
    /// Minimum percentage of votes required for quorum (0-100)
    pub quorum_percentage: u8,
    
    /// Minimum percentage of yes votes required to pass a proposal (0-100)
    pub approval_threshold: u8,
    
    /// Default voting period in seconds
    pub default_voting_period: u64,
    
    /// Minimum reputation required to create proposals
    pub min_proposal_reputation: i64,
    
    /// Minimum reputation required to vote
    pub min_voting_reputation: i64,
    
    /// Minimum reputation required for member admission
    pub min_membership_reputation: i64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_strategy: VotingStrategy::default(),
            quorum_percentage: 50,
            approval_threshold: 66,  // 2/3 majority
            default_voting_period: 7 * 24 * 60 * 60, // 7 days
            min_proposal_reputation: 100,
            min_voting_reputation: 10,
            min_membership_reputation: 0,
        }
    }
}

/// Federation proposal with voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// Unique proposal ID
    pub id: String,
    
    /// Proposal title
    pub title: String,
    
    /// Detailed description
    pub description: String,
    
    /// Member who created the proposal
    pub proposer: String,
    
    /// Federation this proposal belongs to
    pub federation_id: FederationId,
    
    /// Type of proposal
    pub proposal_type: ProposalType,
    
    /// Current status
    pub status: ProposalStatus,
    
    /// Votes cast for this proposal
    pub votes: HashMap<String, Vote>,
    
    /// When voting starts
    pub created_at: u64,
    
    /// When voting ends
    pub ends_at: u64,
    
    /// When the proposal was executed (if applicable)
    pub executed_at: Option<u64>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
    
    /// Optional supporting evidence (URLs, hashes, etc.)
    pub evidence: Vec<String>,
    
    /// Execution result details
    pub execution_result: Option<String>,
}

impl GovernanceProposal {
    pub fn new(
        title: String,
        description: String,
        proposer: String,
        federation_id: FederationId,
        proposal_type: ProposalType,
        voting_period: u64,
        tags: Vec<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            proposer,
            federation_id,
            proposal_type,
            status: ProposalStatus::Draft,
            votes: HashMap::new(),
            created_at: now,
            ends_at: now + voting_period,
            executed_at: None,
            tags,
            evidence: Vec::new(),
            execution_result: None,
        }
    }
    
    /// Check if voting is still open for this proposal
    pub fn is_voting_open(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        self.status == ProposalStatus::Active && now < self.ends_at
    }
    
    /// Add evidence to the proposal
    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
    }
    
    /// Calculate vote totals
    pub fn count_votes(&self) -> (u64, u64, u64) {
        let mut yes_votes = 0;
        let mut no_votes = 0;
        let mut abstain_votes = 0;
        
        for vote in self.votes.values() {
            match vote.decision {
                VoteDecision::Approve => yes_votes += 1,
                VoteDecision::Reject => no_votes += 1,
                VoteDecision::Abstain => abstain_votes += 1,
            }
        }
        
        (yes_votes, no_votes, abstain_votes)
    }
    
    /// Calculate weighted vote totals
    pub fn count_weighted_votes(&self) -> (f64, f64, f64) {
        let mut yes_weight = 0.0;
        let mut no_weight = 0.0;
        let mut abstain_weight = 0.0;
        
        for vote in self.votes.values() {
            // Assuming vote weight is stored in the justification field for simplicity
            // In a real implementation, you'd have a proper weight field
            let weight = 1.0;
            
            match vote.decision {
                VoteDecision::Approve => yes_weight += weight,
                VoteDecision::Reject => no_weight += weight,
                VoteDecision::Abstain => abstain_weight += weight,
            }
        }
        
        (yes_weight, no_weight, abstain_weight)
    }
}

/// Federation governance manager
pub struct GovernanceManager {
    config: GovernanceConfig,
    proposals: Arc<RwLock<HashMap<String, GovernanceProposal>>>,
    federations: Arc<RwLock<HashMap<FederationId, Federation>>>,
    reputation_service: Arc<dyn ReputationInterface>,
}

impl GovernanceManager {
    pub fn new(
        config: GovernanceConfig,
        reputation_service: Arc<dyn ReputationInterface>,
    ) -> Self {
        Self {
            config,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            federations: Arc::new(RwLock::new(HashMap::new())),
            reputation_service,
        }
    }
    
    /// Register a federation with the governance manager
    pub async fn register_federation(&self, federation: Federation) -> GovernanceResult<()> {
        let mut federations = self.federations.write().await;
        federations.insert(federation.id.clone(), federation);
        Ok(())
    }
    
    /// Create a new proposal
    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposer: String,
        federation_id: FederationId,
        proposal_type: ProposalType,
        voting_period: Option<u64>,
        tags: Vec<String>,
    ) -> GovernanceResult<String> {
        // Check if federation exists
        let federations = self.federations.read().await;
        let federation = federations.get(&federation_id)
            .ok_or_else(|| GovernanceError::FederationError(
                FederationError::FederationNotFound(federation_id.0.clone())
            ))?;
            
        // Check if proposer is a member
        let proposer_id = MemberId { 
            did: proposer.clone(), 
            cooperative_id: CooperativeId("default".to_string()) 
        };
        if !federation.members.contains(&proposer_id) {
            return Err(GovernanceError::MemberNotFound(proposer));
        }
        
        // Check if proposer has sufficient reputation
        let reputation = self.reputation_service.get_reputation(&proposer).await
            .map_err(|e| GovernanceError::ReputationError(e.to_string()))?;
            
        if reputation < self.config.min_proposal_reputation {
            return Err(GovernanceError::Unauthorized(
                format!("Insufficient reputation to create proposal: {} < {}", 
                    reputation, self.config.min_proposal_reputation)
            ));
        }
        
        // Create proposal
        let voting_period = voting_period.unwrap_or(self.config.default_voting_period);
        let proposal = GovernanceProposal::new(
            title,
            description,
            proposer,
            federation_id,
            proposal_type,
            voting_period,
            tags,
        );
        
        let proposal_id = proposal.id.clone();
        
        // Store proposal
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal_id.clone(), proposal);
        
        Ok(proposal_id)
    }
    
    /// Submit a vote for a proposal
    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter: String,
        decision: VoteDecision,
        justification: Option<String>,
    ) -> GovernanceResult<()> {
        // Get proposal
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
            
        // Check if voting is still open
        if !proposal.is_voting_open() {
            return Err(GovernanceError::VotingPeriodEnded);
        }
        
        // Check if federation exists
        let federations = self.federations.read().await;
        let federation = federations.get(&proposal.federation_id)
            .ok_or_else(|| GovernanceError::FederationError(
                FederationError::FederationNotFound(proposal.federation_id.0.clone())
            ))?;
            
        // Check if voter is a member
        let voter_id = MemberId { 
            did: voter.clone(), 
            cooperative_id: CooperativeId("default".to_string()) 
        };
        if !federation.members.contains(&voter_id) {
            return Err(GovernanceError::MemberNotFound(voter));
        }
        
        // Check if voter has sufficient reputation
        let reputation = self.reputation_service.get_reputation(&voter).await
            .map_err(|e| GovernanceError::ReputationError(e.to_string()))?;
            
        if reputation < self.config.min_voting_reputation {
            return Err(GovernanceError::Unauthorized(
                format!("Insufficient reputation to vote: {} < {}", 
                    reputation, self.config.min_voting_reputation)
            ));
        }
        
        // Create vote
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let vote = Vote {
            voter: voter.clone(),
            decision,
            timestamp: now,
            justification,
        };
        
        // Store vote
        proposal.votes.insert(voter, vote);
        
        // Update proposal status if needed
        self.check_proposal_outcome(proposal).await?;
        
        Ok(())
    }
    
    /// Check if a proposal has reached a decision
    async fn check_proposal_outcome(&self, proposal: &mut GovernanceProposal) -> GovernanceResult<()> {
        if proposal.status != ProposalStatus::Active {
            return Ok(());
        }
        
        // Get federation
        let federations = self.federations.read().await;
        let federation = federations.get(&proposal.federation_id)
            .ok_or_else(|| GovernanceError::FederationError(
                FederationError::FederationNotFound(proposal.federation_id.0.clone())
            ))?;
            
        // Count votes
        let (yes_votes, no_votes, _) = proposal.count_votes();
        let total_votes = yes_votes + no_votes;
        let total_members = federation.members.len() as u64;
        
        // Check quorum
        let quorum_threshold = (total_members * self.config.quorum_percentage as u64) / 100;
        if total_votes < quorum_threshold {
            // Not enough votes yet
            return Ok(());
        }
        
        // Check approval threshold
        let approval_threshold = (total_votes * self.config.approval_threshold as u64) / 100;
        if yes_votes >= approval_threshold {
            // Proposal approved
            proposal.status = ProposalStatus::Approved;
        } else if no_votes > total_votes - approval_threshold {
            // Proposal rejected (impossible to reach threshold)
            proposal.status = ProposalStatus::Rejected;
        }
        
        Ok(())
    }
    
    /// Execute an approved proposal
    pub async fn execute_proposal(&self, proposal_id: &str) -> GovernanceResult<()> {
        // Get proposal
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
            
        // Check if proposal is approved
        if proposal.status != ProposalStatus::Approved {
            return Err(GovernanceError::InvalidVote(
                format!("Cannot execute proposal with status: {:?}", proposal.status)
            ));
        }
        
        // Get federation
        let mut federations = self.federations.write().await;
        let federation = federations.get_mut(&proposal.federation_id)
            .ok_or_else(|| GovernanceError::FederationError(
                FederationError::FederationNotFound(proposal.federation_id.0.clone())
            ))?;
            
        // Execute proposal based on type
        match &proposal.proposal_type {
            ProposalType::MembershipChange(action) => {
                // Execute membership change
                federation.apply_membership_action(action.clone())?;
                proposal.execution_result = Some("Membership updated successfully".to_string());
            }
            ProposalType::ResourceAllocation(details) => {
                // Execute resource allocation
                federation.allocate_resource(details.clone())?;
                proposal.execution_result = Some("Resource allocated successfully".to_string());
            }
            ProposalType::GovernanceUpdate(details) => {
                // Execute governance update
                federation.update_governance(details.clone())?;
                proposal.execution_result = Some("Governance updated successfully".to_string());
            }
            ProposalType::FederationTermsUpdate(details) => {
                // Execute terms update
                federation.update_terms(details.clone())?;
                proposal.execution_result = Some("Terms updated successfully".to_string());
            }
            ProposalType::Custom(action) => {
                // Just log custom actions
                proposal.execution_result = Some(format!("Custom action executed: {}", action));
            }
        }
        
        // Update proposal status
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(now);
        
        Ok(())
    }
    
    /// Get a proposal by ID
    pub async fn get_proposal(&self, proposal_id: &str) -> GovernanceResult<GovernanceProposal> {
        let proposals = self.proposals.read().await;
        proposals.get(proposal_id)
            .cloned()
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))
    }
    
    /// List all proposals for a federation
    pub async fn list_proposals(
        &self,
        federation_id: &FederationId,
        status_filter: Option<ProposalStatus>,
    ) -> GovernanceResult<Vec<GovernanceProposal>> {
        let proposals = self.proposals.read().await;
        
        let filtered = proposals.values()
            .filter(|p| &p.federation_id == federation_id)
            .filter(|p| status_filter.as_ref().map_or(true, |s| p.status == *s))
            .cloned()
            .collect();
            
        Ok(filtered)
    }
    
    /// Check for proposals with ended voting periods
    pub async fn check_expired_proposals(&self) -> GovernanceResult<Vec<String>> {
        let mut updated_proposals = Vec::new();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut proposals = self.proposals.write().await;
        
        for proposal in proposals.values_mut() {
            if proposal.status == ProposalStatus::Active && now >= proposal.ends_at {
                // Voting period ended
                
                // Count votes
                let (yes_votes, no_votes, _) = proposal.count_votes();
                let total_votes = yes_votes + no_votes;
                
                // Get total members
                let federations = self.federations.read().await;
                if let Some(federation) = federations.get(&proposal.federation_id) {
                    let total_members = federation.members.len() as u64;
                    
                    // Check quorum
                    let quorum_threshold = (total_members * self.config.quorum_percentage as u64) / 100;
                    if total_votes < quorum_threshold {
                        // Failed due to lack of quorum
                        proposal.status = ProposalStatus::Rejected;
                        proposal.execution_result = Some("Rejected due to insufficient quorum".to_string());
                    } else {
                        // Check approval threshold
                        let approval_threshold = (total_votes * self.config.approval_threshold as u64) / 100;
                        if yes_votes >= approval_threshold {
                            // Proposal approved
                            proposal.status = ProposalStatus::Approved;
                        } else {
                            // Proposal rejected
                            proposal.status = ProposalStatus::Rejected;
                        }
                    }
                    
                    updated_proposals.push(proposal.id.clone());
                }
            }
        }
        
        Ok(updated_proposals)
    }
    
    /// Calculate voting power for a member based on reputation
    pub async fn calculate_voting_power(&self, member_id: &str) -> GovernanceResult<f64> {
        let reputation = self.reputation_service.get_reputation(member_id).await
            .map_err(|e| GovernanceError::ReputationError(e.to_string()))?;
            
        let voting_power = match self.config.voting_strategy {
            VotingStrategy::EqualVoting => {
                // Everyone gets equal vote
                1.0
            }
            VotingStrategy::ReputationWeighted => {
                // Linear scaling with reputation
                reputation as f64 / 1000.0
            }
            VotingStrategy::QuadraticVoting => {
                // Square root scaling for diminishing returns
                (reputation as f64).sqrt()
            }
            VotingStrategy::ResourceWeighted => {
                // For now, approximation based on reputation
                // In a real implementation, would be based on resource contributions
                reputation as f64 / 500.0
            }
        };
        
        Ok(voting_power.max(0.1).min(100.0)) // Clamp between 0.1 and 100
    }

    async fn get_federation(&self, federation_id: &FederationId) -> GovernanceResult<Federation> {
        let federations = self.federations.read().await;
        federations.get(federation_id)
            .cloned()
            .ok_or_else(|| GovernanceError::FederationError(
                FederationError::FederationNotFound(federation_id.0.clone())
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::{MembershipAction, ResourceAllocationDetails, ResourceType};
    
    struct MockReputationService {
        reputations: HashMap<String, i64>,
    }
    
    #[async_trait::async_trait]
    impl ReputationInterface for MockReputationService {
        async fn update_reputation(&self, _member_id: &str, _delta: i64) -> Result<(), icn_types::ReputationError> {
            Ok(())
        }
        
        async fn get_reputation(&self, member_id: &str) -> Result<i64, icn_types::ReputationError> {
            Ok(*self.reputations.get(member_id).unwrap_or(&0))
        }
        
        async fn validate_reputation(&self, member_id: &str, min_required: i64) -> Result<bool, icn_types::ReputationError> {
            let rep = self.get_reputation(member_id).await?;
            Ok(rep >= min_required)
        }
        
        async fn get_voting_power(&self, member_id: &str) -> Result<f64, icn_types::ReputationError> {
            let rep = self.get_reputation(member_id).await?;
            Ok(rep as f64 / 100.0)
        }
    }
    
    #[tokio::test]
    async fn test_proposal_creation() {
        // Setup
        let mut reputations = HashMap::new();
        reputations.insert("member1".to_string(), 200);
        
        let reputation_service = Arc::new(MockReputationService { reputations });
        let governance = GovernanceManager::new(GovernanceConfig::default(), reputation_service);
        
        // Create a test federation
        let mut federation = Federation {
            id: "fed1".to_string(),
            name: "Test Federation".to_string(),
            federation_type: FederationType::ResourceSharing,
            members: HashMap::new(),
            member_roles: HashMap::new(),
            terms: FederationTerms::default(),
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: 0,
            status: Default::default(),
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
        };
        
        federation.members.insert("member1".to_string(), Default::default());
        
        governance.register_federation(federation).await.unwrap();
        
        // Test proposal creation
        let proposal_id = governance.create_proposal(
            "Test Proposal".to_string(),
            "This is a test".to_string(),
            "member1".to_string(),
            "fed1".to_string(),
            ProposalType::Custom("test".to_string()),
            None,
            vec!["test".to_string()],
        ).await.unwrap();
        
        // Verify proposal was created
        let proposal = governance.get_proposal(&proposal_id).await.unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.proposer, "member1");
    }
    
    #[tokio::test]
    async fn test_voting() {
        // Setup
        let mut reputations = HashMap::new();
        reputations.insert("member1".to_string(), 200);
        reputations.insert("member2".to_string(), 150);
        reputations.insert("member3".to_string(), 100);
        
        let reputation_service = Arc::new(MockReputationService { reputations });
        let governance = GovernanceManager::new(GovernanceConfig::default(), reputation_service);
        
        // Create a test federation
        let mut federation = Federation {
            id: "fed1".to_string(),
            name: "Test Federation".to_string(),
            federation_type: FederationType::ResourceSharing,
            members: HashMap::new(),
            member_roles: HashMap::new(),
            terms: FederationTerms::default(),
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: 0,
            status: Default::default(),
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
        };
        
        federation.members.insert("member1".to_string(), Default::default());
        federation.members.insert("member2".to_string(), Default::default());
        federation.members.insert("member3".to_string(), Default::default());
        
        governance.register_federation(federation).await.unwrap();
        
        // Create a proposal
        let proposal_id = governance.create_proposal(
            "Test Proposal".to_string(),
            "This is a test".to_string(),
            "member1".to_string(),
            "fed1".to_string(),
            ProposalType::MembershipChange(MembershipAction::Add("new_member".to_string())),
            Some(3600), // 1 hour voting period
            vec!["test".to_string()],
        ).await.unwrap();
        
        // Cast votes
        governance.submit_vote(&proposal_id, "member1".to_string(), VoteDecision::Approve, None).await.unwrap();
        governance.submit_vote(&proposal_id, "member2".to_string(), VoteDecision::Approve, None).await.unwrap();
        governance.submit_vote(&proposal_id, "member3".to_string(), VoteDecision::Reject, None).await.unwrap();
        
        // Check proposal status
        let proposal = governance.get_proposal(&proposal_id).await.unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved); // 2/3 approval
        
        // Execute proposal
        governance.execute_proposal(&proposal_id).await.unwrap();
        
        // Verify proposal was executed
        let proposal = governance.get_proposal(&proposal_id).await.unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert!(proposal.execution_result.is_some());
    }
}