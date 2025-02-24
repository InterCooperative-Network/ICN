use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub creator_did: String,
    pub creation_time: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteData {
    pub proposal_id: String,
    pub voter_did: String,
    pub approve: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Finalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalOutcome {
    pub id: String,
    pub status: ProposalStatus,
    pub total_weight: u32,
    pub approval_weight: u32,
    pub finalization_time: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    #[error("Insufficient cooperation score: {0}")]
    InsufficientCooperation(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub struct GovernanceSystem {
    proposals: RwLock<HashMap<String, ProposalData>>,
    votes: RwLock<HashMap<String, Vec<VoteData>>>,
    proof_of_cooperation: RwLock<ProofOfCooperation>,
}

impl GovernanceSystem {
    pub fn new(proof_of_cooperation: ProofOfCooperation) -> Self {
        Self {
            proposals: RwLock::new(HashMap::new()),
            votes: RwLock::new(HashMap::new()),
            proof_of_cooperation: RwLock::new(proof_of_cooperation),
        }
    }

    pub async fn create_proposal(&self, proposal: ProposalData) -> Result<(), GovernanceError> {
        // Verify creator's cooperation score
        let creator_score = self.verify_proof_of_cooperation(&proposal.creator_did).await?;
        if creator_score < 10 { // Minimum score to create proposals
            return Err(GovernanceError::InsufficientCooperation(
                format!("Score {} is below minimum threshold", creator_score)
            ));
        }

        // Store proposal
        let mut proposals = self.proposals.write().await;
        if proposals.contains_key(&proposal.id) {
            return Err(GovernanceError::InvalidProposal("Proposal ID already exists".into()));
        }
        
        proposals.insert(proposal.id.clone(), proposal);
        self.votes.write().await.insert(proposal.id, Vec::new());
        
        Ok(())
    }

    pub async fn cast_vote(&self, vote: VoteData) -> Result<(), GovernanceError> {
        // Verify proposal exists and is active
        let proposals = self.proposals.read().await;
        let proposal = proposals.get(&vote.proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Verify voter's cooperation score
        let voter_score = self.verify_proof_of_cooperation(&vote.voter_did).await?;
        if voter_score == 0 {
            return Err(GovernanceError::InsufficientCooperation(
                "No cooperation score found".into()
            ));
        }

        // Record vote with weight
        let mut votes = self.votes.write().await;
        let proposal_votes = votes.get_mut(&vote.proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Check for duplicate votes
        if proposal_votes.iter().any(|v| v.voter_did == vote.voter_did) {
            return Err(GovernanceError::InvalidVote("Duplicate vote".into()));
        }

        proposal_votes.push(vote);
        Ok(())
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> Result<ProposalOutcome, GovernanceError> {
        let mut proposals = self.proposals.write().await;
        let votes = self.votes.read().await;
        
        let proposal = proposals.get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        let proposal_votes = votes.get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        let mut total_weight = 0u32;
        let mut approval_weight = 0u32;

        // Calculate weighted votes
        for vote in proposal_votes {
            if let Ok(weight) = self.verify_proof_of_cooperation(&vote.voter_did).await {
                total_weight += weight;
                if vote.approve {
                    approval_weight += weight;
                }
            }
        }

        // Determine outcome (require >50% weighted approval)
        let status = if total_weight > 0 && approval_weight * 2 > total_weight {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        let outcome = ProposalOutcome {
            id: proposal_id.to_string(),
            status,
            total_weight,
            approval_weight,
            finalization_time: Utc::now(),
        };

        Ok(outcome)
    }

    pub async fn verify_proof_of_cooperation(&self, voter: &str) -> Result<u32, GovernanceError> {
        let poc = self.proof_of_cooperation.read().await;
        
        // Get cooperation score from reputation system
        if let Some(score) = poc.get_cooperation_score(voter).await {
            Ok(score)
        } else {
            Err(GovernanceError::InsufficientCooperation(
                format!("No cooperation score found for {}", voter)
            ))
        }
    }
}

#[async_trait]
pub trait LedgerClient {
    async fn store_proposal(&self, proposal: &ProposalData) -> Result<(), GovernanceError>;
    async fn store_vote(&self, vote: &VoteData) -> Result<(), GovernanceError>;
    async fn store_outcome(&self, outcome: &ProposalOutcome) -> Result<(), GovernanceError>;
}
