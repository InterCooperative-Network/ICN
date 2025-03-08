use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use icn_types::{FederationId, Did};
use icn_reputation::{ReputationManager, ReputationScore};
use crate::messaging::{FederationMessenger, MessageType, MessagePriority, MessageVisibility};

/// Error types for federation governance
#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    
    #[error("Vote not found: {0}")]
    VoteNotFound(String),
    
    #[error("Unauthorized action: {0}")]
    Unauthorized(String),
    
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    
    #[error("Voting period ended")]
    VotingPeriodEnded,
    
    #[error("Insufficient reputation: required {required}, actual {actual}")]
    InsufficientReputation { required: i64, actual: i64 },
    
    #[error("Proposal already exists: {0}")]
    ProposalExists(String),
    
    #[error("Already voted on proposal")]
    AlreadyVoted,
    
    #[error("Federation messaging error: {0}")]
    MessagingError(String),
    
    #[error("Reputation system error: {0}")]
    ReputationError(String),
}

/// Types of federation governance proposals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    /// Add a new member to the federation
    AddMember,
    
    /// Remove a member from the federation
    RemoveMember,
    
    /// Change federation parameters
    ChangeParameters,
    
    /// Allocate resources
    ResourceAllocation,
    
    /// Form partnership with another federation
    FormPartnership,
    
    /// Dissolve partnership with another federation
    DissolvePartnership,
    
    /// Modify federation rules
    ModifyRules,
    
    /// Generic proposal type
    Generic,
    
    /// Custom proposal type
    Custom(String),
}

/// Voting methods for proposals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VotingMethod {
    /// Simple majority voting (>50%)
    SimpleMajority,
    
    /// Super majority voting (e.g., 2/3 or 3/4)
    SuperMajority(f64),
    
    /// Unanimous voting (100%)
    Unanimous,
    
    /// Quadratic voting (votes weighted by square root of reputation)
    QuadraticVoting,
    
    /// Reputation-weighted voting
    ReputationWeighted,
    
    /// Delegation-based voting
    DelegatedVoting,
    
    /// Custom voting method
    Custom(String),
}

/// Current status of a proposal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal has been created but not yet open for voting
    Draft,
    
    /// Proposal is open for voting
    Active,
    
    /// Proposal has been approved
    Approved,
    
    /// Proposal has been rejected
    Rejected,
    
    /// Proposal has been withdrawn
    Withdrawn,
    
    /// Proposal execution is in progress
    Executing,
    
    /// Proposal has been executed
    Executed,
    
    /// Proposal has failed execution
    Failed,
}

/// Voting options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteOption {
    /// Vote in favor of the proposal
    Approve,
    
    /// Vote against the proposal
    Reject,
    
    /// Abstain from voting (neutral)
    Abstain,
    
    /// Request more information before voting
    RequestInfo,
}

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: String,
    
    /// Title of the proposal
    pub title: String,
    
    /// Detailed description of the proposal
    pub description: String,
    
    /// Type of the proposal
    pub proposal_type: ProposalType,
    
    /// DID of the proposer
    pub proposer: Did,
    
    /// When the proposal was created
    pub created_at: DateTime<Utc>,
    
    /// When voting starts
    pub voting_starts_at: DateTime<Utc>,
    
    /// When voting ends
    pub voting_ends_at: DateTime<Utc>,
    
    /// Current status of the proposal
    pub status: ProposalStatus,
    
    /// Voting method to be used
    pub voting_method: VotingMethod,
    
    /// Minimum reputation required to vote
    pub min_reputation: i64,
    
    /// Proposal data (type-specific content)
    pub data: HashMap<String, String>,
    
    /// Proposal attachments (e.g., documents, evidence)
    pub attachments: Vec<ProposalAttachment>,
    
    /// Record of votes on this proposal
    pub votes: HashMap<Did, Vote>,
    
    /// Metadata and tags
    pub metadata: HashMap<String, String>,
    
    /// Federation this proposal belongs to
    pub federation_id: FederationId,
}

/// Attachment to a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalAttachment {
    /// Name of the attachment
    pub name: String,
    
    /// MIME type of the attachment
    pub mime_type: String,
    
    /// Content hash for integrity verification
    pub hash: String,
    
    /// URL or IPFS CID where the content can be retrieved
    pub url: String,
    
    /// Size in bytes
    pub size: u64,
}

/// A vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// ID of the vote
    pub id: String,
    
    /// ID of the proposal being voted on
    pub proposal_id: String,
    
    /// DID of the voter
    pub voter: Did,
    
    /// The vote choice
    pub vote: VoteOption,
    
    /// Optional explanation for the vote
    pub explanation: Option<String>,
    
    /// Reputation weight of the vote
    pub weight: f64,
    
    /// When the vote was cast
    pub timestamp: DateTime<Utc>,
    
    /// If this vote was delegated
    pub delegated: bool,
    
    /// If delegated, the original voter
    pub delegated_from: Option<Did>,
    
    /// Digital signature of the vote
    pub signature: String,
}

/// Results of a vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResults {
    /// ID of the proposal
    pub proposal_id: String,
    
    /// Total number of votes
    pub total_votes: usize,
    
    /// Number of approve votes
    pub approve_count: usize,
    
    /// Number of reject votes
    pub reject_count: usize,
    
    /// Number of abstain votes
    pub abstain_count: usize,
    
    /// Number of request info votes
    pub request_info_count: usize,
    
    /// Sum of approve vote weights
    pub approve_weight: f64,
    
    /// Sum of reject vote weights
    pub reject_weight: f64,
    
    /// Sum of abstain vote weights
    pub abstain_weight: f64,
    
    /// Sum of request info vote weights
    pub request_info_weight: f64,
    
    /// Total possible voting weight
    pub total_possible_weight: f64,
    
    /// Participation rate (0.0 - 1.0)
    pub participation_rate: f64,
    
    /// Whether the proposal passed
    pub passed: bool,
}

/// Federation governance system
pub struct FederationGovernance {
    /// Federation ID
    federation_id: FederationId,
    
    /// Active proposals
    proposals: RwLock<HashMap<String, Proposal>>,
    
    /// Archive of past proposals
    proposal_archive: RwLock<HashMap<String, Proposal>>,
    
    /// Federation messenger for proposal notifications
    messenger: Arc<FederationMessenger>,
    
    /// Reputation manager for vote weighting
    reputation_manager: Arc<dyn ReputationManager>,
    
    /// Vote delegates
    delegates: RwLock<HashMap<Did, Did>>,
    
    /// Minimum reputation to create proposals
    min_proposal_reputation: RwLock<i64>,
    
    /// Proposals pending execution
    pending_execution: RwLock<Vec<String>>,
    
    /// Vote proxy configurations
    vote_proxies: RwLock<HashMap<Did, Vec<Did>>>,
}

impl FederationGovernance {
    /// Create a new federation governance system
    pub fn new(
        federation_id: FederationId,
        messenger: Arc<FederationMessenger>,
        reputation_manager: Arc<dyn ReputationManager>,
    ) -> Self {
        Self {
            federation_id,
            proposals: RwLock::new(HashMap::new()),
            proposal_archive: RwLock::new(HashMap::new()),
            messenger,
            reputation_manager,
            delegates: RwLock::new(HashMap::new()),
            min_proposal_reputation: RwLock::new(100), // Default minimum reputation
            pending_execution: RwLock::new(Vec::new()),
            vote_proxies: RwLock::new(HashMap::new()),
        }
    }

    /// Set minimum reputation required to create proposals
    pub async fn set_min_proposal_reputation(&self, reputation: i64) {
        let mut min_rep = self.min_proposal_reputation.write().await;
        *min_rep = reputation;
    }

    /// Create a new proposal
    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        proposer: Did,
        voting_starts_at: DateTime<Utc>,
        voting_ends_at: DateTime<Utc>,
        voting_method: VotingMethod,
        min_reputation: i64,
        data: HashMap<String, String>,
        attachments: Vec<ProposalAttachment>,
        metadata: HashMap<String, String>,
    ) -> Result<String, GovernanceError> {
        // Check if proposer has sufficient reputation
        let proposer_reputation = self.reputation_manager.get_reputation(&proposer)
            .await
            .map_err(|e| GovernanceError::ReputationError(e.to_string()))?;
            
        let min_rep = *self.min_proposal_reputation.read().await;
        
        if proposer_reputation.score < min_rep {
            return Err(GovernanceError::InsufficientReputation { 
                required: min_rep,
                actual: proposer_reputation.score,
            });
        }
        
        // Validate proposal parameters
        if voting_starts_at >= voting_ends_at {
            return Err(GovernanceError::InvalidProposal("Voting end time must be after start time".to_string()));
        }
        
        // Create a new proposal ID
        let proposal_id = Uuid::new_v4().to_string();
        
        // Create the proposal
        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            proposal_type,
            proposer: proposer.clone(),
            created_at: Utc::now(),
            voting_starts_at,
            voting_ends_at,
            status: ProposalStatus::Draft,
            voting_method,
            min_reputation,
            data,
            attachments,
            votes: HashMap::new(),
            metadata,
            federation_id: self.federation_id.clone(),
        };
        
        // Store the proposal
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal_id.clone(), proposal.clone());
        
        // Notify federation members about the new proposal
        self.notify_new_proposal(&proposal).await?;
        
        Ok(proposal_id)
    }

    /// Notify federation members about a new proposal
    async fn notify_new_proposal(&self, proposal: &Proposal) -> Result<(), GovernanceError> {
        let title = format!("New Proposal: {}", proposal.title);
        let message = format!(
            "A new proposal has been created by {}.\n\nTitle: {}\n\nDescription: {}\n\nVoting starts: {}\nVoting ends: {}\n\nProposal ID: {}",
            proposal.proposer,
            proposal.title,
            proposal.description,
            proposal.voting_starts_at.to_rfc3339(),
            proposal.voting_ends_at.to_rfc3339(),
            proposal.id
        );
        
        // Send message via the messenger
        // In a production system, this would be more sophisticated with proper serialization
        self.messenger.send_new_message(
            &self.federation_id, // Send to all federation members
            MessageType::Proposal,
            &title,
            message.as_bytes(),
            MessageVisibility::Federation,
            MessagePriority::Normal,
            Vec::new(),
            None,
        )
        .await
        .map_err(|e| GovernanceError::MessagingError(e.to_string()))?;
        
        Ok(())
    }

    /// Get a proposal by ID
    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Proposal, GovernanceError> {
        // Check active proposals
        let proposals = self.proposals.read().await;
        if let Some(proposal) = proposals.get(proposal_id) {
            return Ok(proposal.clone());
        }
        
        // Check archived proposals
        let archived = self.proposal_archive.read().await;
        archived.get(proposal_id)
            .cloned()
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))
    }

    /// List all active proposals
    pub async fn list_active_proposals(&self) -> Vec<Proposal> {
        let proposals = self.proposals.read().await;
        proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .cloned()
            .collect()
    }

    /// List all proposals
    pub async fn list_all_proposals(&self, include_archived: bool) -> Vec<Proposal> {
        let mut result = Vec::new();
        
        // Get active proposals
        {
            let proposals = self.proposals.read().await;
            result.extend(proposals.values().cloned());
        }
        
        // Get archived proposals if requested
        if include_archived {
            let archived = self.proposal_archive.read().await;
            result.extend(archived.values().cloned());
        }
        
        result
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(
        &self,
        proposal_id: &str,
        voter: Did,
        vote: VoteOption,
        explanation: Option<String>,
        signature: String,
    ) -> Result<String, GovernanceError> {
        // Get the proposal
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
        
        // Check if voting is open
        let now = Utc::now();
        if now < proposal.voting_starts_at {
            return Err(GovernanceError::InvalidVote("Voting has not started yet".to_string()));
        }
        if now > proposal.voting_ends_at {
            return Err(GovernanceError::VotingPeriodEnded);
        }
        
        // Check if proposal is active
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::InvalidVote(format!("Proposal is not active: {:?}", proposal.status)));
        }
        
        // Check if voter has already voted
        if proposal.votes.contains_key(&voter) {
            return Err(GovernanceError::AlreadyVoted);
        }
        
        // Check voter reputation
        let voter_reputation = self.reputation_manager.get_reputation(&voter)
            .await
            .map_err(|e| GovernanceError::ReputationError(e.to_string()))?;
            
        if voter_reputation.score < proposal.min_reputation {
            return Err(GovernanceError::InsufficientReputation {
                required: proposal.min_reputation,
                actual: voter_reputation.score,
            });
        }
        
        // Calculate vote weight based on voting method
        let weight = self.calculate_vote_weight(&proposal.voting_method, &voter_reputation).await;
        
        // Generate vote ID
        let vote_id = Uuid::new_v4().to_string();
        
        // Create the vote
        let vote_record = Vote {
            id: vote_id.clone(),
            proposal_id: proposal_id.to_string(),
            voter: voter.clone(),
            vote,
            explanation,
            weight,
            timestamp: now,
            delegated: false,
            delegated_from: None,
            signature,
        };
        
        // Record the vote
        proposal.votes.insert(voter, vote_record.clone());
        
        // Check if we should update proposal status
        self.update_proposal_status(proposal).await;
        
        Ok(vote_id)
    }

    /// Calculate vote weight based on voting method and reputation
    async fn calculate_vote_weight(&self, voting_method: &VotingMethod, reputation: &ReputationScore) -> f64 {
        match voting_method {
            VotingMethod::SimpleMajority | VotingMethod::SuperMajority(_) | VotingMethod::Unanimous => {
                // Each vote counts as 1
                1.0
            },
            VotingMethod::QuadraticVoting => {
                // Square root of reputation
                (reputation.score as f64).sqrt()
            },
            VotingMethod::ReputationWeighted => {
                // Weight is proportional to reputation
                reputation.score as f64
            },
            VotingMethod::DelegatedVoting => {
                // In delegated voting, we need to account for delegated votes
                // This is a simplification - real delegation would be more complex
                reputation.score as f64
            },
            VotingMethod::Custom(_) => {
                // Default for custom methods
                1.0
            },
        }
    }

    /// Update proposal status based on votes
    async fn update_proposal_status(&self, proposal: &mut Proposal) {
        // Calculate current vote totals
        let results = self.calculate_vote_results(proposal);
        
        // Check if we have a decision
        match proposal.voting_method {
            VotingMethod::SimpleMajority => {
                // Simple majority requires > 50% approval of total votes cast
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.approve_weight > results.reject_weight {
                    proposal.status = ProposalStatus::Approved;
                }
            },
            VotingMethod::SuperMajority(threshold) => {
                // Super majority requires approval to meet or exceed threshold
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.approve_weight / (results.approve_weight + results.reject_weight) >= threshold {
                    proposal.status = ProposalStatus::Approved;
                }
            },
            VotingMethod::Unanimous => {
                // Unanimous requires all votes to be approvals
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.reject_weight == 0.0 && results.approve_weight > 0.0 {
                    proposal.status = ProposalStatus::Approved;
                }
            },
            VotingMethod::QuadraticVoting | VotingMethod::ReputationWeighted => {
                // Weighted voting methods require more approval weight than rejection weight
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.approve_weight > results.reject_weight {
                    proposal.status = ProposalStatus::Approved;
                }
            },
            VotingMethod::DelegatedVoting => {
                // Delegated voting also needs more approval weight than rejection weight
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.approve_weight > results.reject_weight {
                    proposal.status = ProposalStatus::Approved;
                }
            },
            VotingMethod::Custom(_) => {
                // Custom methods would have their own implementation
                // For now, we use a simple majority as default
                if results.participation_rate >= 0.5 && // Minimum participation requirement
                   results.approve_weight > results.reject_weight {
                    proposal.status = ProposalStatus::Approved;
                }
            },
        }
    }

    /// Calculate vote results for a proposal
    pub fn calculate_vote_results(&self, proposal: &Proposal) -> VoteResults {
        let mut results = VoteResults {
            proposal_id: proposal.id.clone(),
            total_votes: proposal.votes.len(),
            approve_count: 0,
            reject_count: 0,
            abstain_count: 0,
            request_info_count: 0,
            approve_weight: 0.0,
            reject_weight: 0.0,
            abstain_weight: 0.0,
            request_info_weight: 0.0,
            total_possible_weight: 0.0, // This will be calculated later
            participation_rate: 0.0,    // This will be calculated later
            passed: false,
        };
        
        // Count votes and weights
        for vote in proposal.votes.values() {
            match vote.vote {
                VoteOption::Approve => {
                    results.approve_count += 1;
                    results.approve_weight += vote.weight;
                },
                VoteOption::Reject => {
                    results.reject_count += 1;
                    results.reject_weight += vote.weight;
                },
                VoteOption::Abstain => {
                    results.abstain_count += 1;
                    results.abstain_weight += vote.weight;
                },
                VoteOption::RequestInfo => {
                    results.request_info_count += 1;
                    results.request_info_weight += vote.weight;
                },
            }
        }
        
        // For simplicity in this implementation, we estimate total possible weight
        // In a real system, we'd calculate based on eligible voters
        results.total_possible_weight = results.approve_weight + 
                                       results.reject_weight + 
                                       results.abstain_weight + 
                                       results.request_info_weight;
                                       
        if results.total_possible_weight > 0.0 {
            results.participation_rate = (results.approve_weight + 
                                        results.reject_weight) / 
                                        results.total_possible_weight;
        }
        
        // Determine if proposal passed
        results.passed = proposal.status == ProposalStatus::Approved;
        
        results
    }

    /// Delegate voting authority to another member
    pub async fn set_voting_delegate(&self, delegator: Did, delegate: Did) -> Result<(), GovernanceError> {
        let mut delegates = self.delegates.write().await;
        delegates.insert(delegator, delegate);
        Ok(())
    }

    /// Remove voting delegation
    pub async fn remove_voting_delegate(&self, delegator: &Did) -> Result<(), GovernanceError> {
        let mut delegates = self.delegates.write().await;
        delegates.remove(delegator);
        Ok(())
    }

    /// Execute an approved proposal
    pub async fn execute_proposal(&self, proposal_id: &str) -> Result<(), GovernanceError> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
            
        // Check if proposal is approved
        if proposal.status != ProposalStatus::Approved {
            return Err(GovernanceError::InvalidProposal(
                format!("Proposal not approved: {:?}", proposal.status)
            ));
        }
        
        // Update status to executing
        proposal.status = ProposalStatus::Executing;
        
        // Add to pending execution queue
        let mut pending = self.pending_execution.write().await;
        pending.push(proposal_id.to_string());
        
        // In a real implementation, we'd have proposal handlers for different types
        // For now, just mark it as executed
        proposal.status = ProposalStatus::Executed;
        
        // Send notification about executed proposal
        self.notify_proposal_execution(proposal).await?;
        
        Ok(())
    }

    /// Notify federation members about proposal execution
    async fn notify_proposal_execution(&self, proposal: &Proposal) -> Result<(), GovernanceError> {
        let title = format!("Proposal Executed: {}", proposal.title);
        let message = format!(
            "Proposal has been executed.\n\nTitle: {}\n\nProposal ID: {}",
            proposal.title,
            proposal.id
        );
        
        // Send message via the messenger
        self.messenger.send_new_message(
            &self.federation_id, // Send to all federation members
            MessageType::SystemNotification,
            &title,
            message.as_bytes(),
            MessageVisibility::Federation,
            MessagePriority::High,
            Vec::new(),
            None,
        )
        .await
        .map_err(|e| GovernanceError::MessagingError(e.to_string()))?;
        
        Ok(())
    }

    /// Set up a new vote proxy
    pub async fn set_vote_proxy(&self, voter: Did, proxy_voters: Vec<Did>) -> Result<(), GovernanceError> {
        let mut proxies = self.vote_proxies.write().await;
        proxies.insert(voter, proxy_voters);
        Ok(())
    }

    /// Get vote results for a proposal
    pub async fn get_vote_results(&self, proposal_id: &str) -> Result<VoteResults, GovernanceError> {
        let proposal = self.get_proposal(proposal_id).await?;
        Ok(self.calculate_vote_results(&proposal))
    }

    /// Archive a proposal
    pub async fn archive_proposal(&self, proposal_id: &str) -> Result<(), GovernanceError> {
        let mut proposals = self.proposals.write().await;
        
        // Find and remove the proposal from active proposals
        let proposal = proposals.remove(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
            
        // Add to archive
        let mut archive = self.proposal_archive.write().await;
        archive.insert(proposal_id.to_string(), proposal);
        
        Ok(())
    }

    /// Check and update status of all proposals
    pub async fn update_all_proposal_statuses(&self) -> Result<usize, GovernanceError> {
        let now = Utc::now();
        let mut updated_count = 0;
        let mut to_archive = Vec::new();
        
        // First, get all proposals that need updating
        let mut proposals = self.proposals.write().await;
        
        for (id, proposal) in proposals.iter_mut() {
            // Check if voting period has ended
            if proposal.status == ProposalStatus::Active && now > proposal.voting_ends_at {
                // Update status based on votes
                self.update_proposal_status(proposal).await;
                
                // If still active after update, it didn't reach the threshold - mark as rejected
                if proposal.status == ProposalStatus::Active {
                    proposal.status = ProposalStatus::Rejected;
                }
                
                updated_count += 1;
            }
            
            // Check if proposal is final and should be archived
            match proposal.status {
                ProposalStatus::Executed | ProposalStatus::Rejected | ProposalStatus::Failed | ProposalStatus::Withdrawn => {
                    to_archive.push(id.clone());
                }
                _ => {}
            }
        }
        
        // Archive completed proposals
        for id in to_archive {
            if let Some(proposal) = proposals.remove(&id) {
                let mut archive = self.proposal_archive.write().await;
                archive.insert(id, proposal);
                updated_count += 1;
            }
        }
        
        Ok(updated_count)
    }

    /// Start the background proposal status updater
    pub async fn start_background_updater(governance: Arc<FederationGovernance>) {
        tokio::spawn(async move {
            loop {
                // Update proposal statuses
                if let Err(e) = governance.update_all_proposal_statuses().await {
                    eprintln!("Error updating proposal statuses: {:?}", e);
                }
                
                // Sleep for a reasonable interval
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }
}