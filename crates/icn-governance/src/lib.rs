use thiserror::Error;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use icn_types::{DecisionType, Federation, ExecutionEvent, EvidenceItem};
use icn_zk::verify_proof; // Import zk-SNARK verification function

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Invalid rollback - proposal is not in pending state")]
    InvalidRollback,
    #[error("Dispute already exists")]
    DisputeExists,
    #[error("Insufficient reputation to dispute")]
    InsufficientReputation,
    #[error("Invalid dispute resolution")]
    InvalidResolution,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub votes: HashMap<String, Vote>,
    pub disputes: Vec<Dispute>,
    pub execution_history: Vec<ExecutionEvent>,
    pub phase: ProposalPhase,
    pub required_signers: Vec<String>,
    pub collected_signatures: Vec<String>,
    pub state: ProposalState,
    pub dispute_resolution: Option<DisputeResolution>,
    pub timeout_timestamp: u64,
    pub zk_snark_proof: Option<String>, // Added zk-SNARK proof field
    pub voting_model: VotingModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Approved,
    Rejected,
    Disputed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalPhase {
    Submission,
    Deliberation { ends_at: DateTime<Utc> },
    Voting { ends_at: DateTime<Utc> },
    Execution,
    Reconsideration { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub approve: bool,
    pub reputation: i64,
    pub timestamp: DateTime<Utc>,
    pub voter_id: String,
    pub proposal_id: String,
    pub decision: DecisionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: String,
    pub proposal_id: String,
    pub disputer: String,
    pub reason: String,
    pub evidence: String,
    pub arbitrator_did: Option<String>,
    pub resolution: Option<Resolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub decision: DecisionType,
    pub rationale: String,
    pub signatures: Vec<String>, // Requires 75% of arbitrators
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissolutionProtocol {
    pub federation_id: String,
    pub initiated_by: String,
    pub reason: DissolutionReason,
    pub status: DissolutionStatus,
    pub asset_distribution: HashMap<String, u64>,
    pub debt_settlements: Vec<String>,
    pub member_reassignments: Vec<String>,
    pub dispute_period_ends: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissolutionReason {
    Voluntary,
    InactivityThreshold,
    GovernanceFailure,
    ResourceDepletion,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissolutionStatus {
    Initiated,
    UnderReview,
    Approved,
    Rejected,
    Completed,
}

#[derive(Debug)]
pub struct AssetAllocation {
    asset_id: String,
    recipient_id: String,
    allocation_share: f64,
}

#[derive(Debug)]
pub struct DebtSettlement {
    creditor_id: String,
    debtor_id: String,
    amount: f64,
    due_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct MemberReassignment {
    member_id: String,
    new_federation_id: Option<String>,
    transition_period: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionThresholds {
    pub resource_allocation_threshold: u64,
    pub technical_effort_threshold: u32, // in days
    pub financial_impact_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationControls {
    pub max_gap_multiplier: f64,
    pub decay_threshold_multiplier: f64,
    pub monthly_decay_rate: f64,
    pub equity_bonus_groups: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingModel {
    Equal,
    Proportional { cap_percentage: u8 },
    Hybrid {
        governance_model: Box<VotingModel>,
        resource_model: Box<VotingModel>,
    },
    Simple,
    Weighted { weight_factor: f64 },
    Hybrid { governance_model: f64, resource_model: f64 },
}

impl VotingModel {
    pub fn calculate_voting_power(&self, federation: &Federation, cooperative_id: &str) -> f64 {
        match self {
            VotingModel::Equal => 1.0,
            VotingModel::Proportional { cap_percentage } => {
                let power = federation.get_cooperative_weight(cooperative_id);
                power.min(*cap_percentage as f64 / 100.0)
            },
            VotingModel::Hybrid { governance_model, resource_model } => {
                // Use different models based on proposal type
                // ...existing code...
            }
            VotingModel::Simple => 1.0,
            VotingModel::Weighted { weight_factor } => {
                // Implement weighted voting calculation
                weight_factor * 1.0
            }
            VotingModel::Hybrid { governance_model, resource_model } => {
                // Implement hybrid voting calculation
                governance_model * 0.5 + resource_model * 0.5
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationManager {
    pub controls: ReputationControls,
    reputation_scores: HashMap<String, f64>,
    last_decay_check: DateTime<Utc>,
}

impl ReputationManager {
    pub fn apply_anti_oppression_mechanisms(&mut self) {
        let avg_reputation = self.calculate_average_reputation();
        
        for score in self.reputation_scores.values_mut() {
            if *score > avg_reputation * self.controls.decay_threshold_multiplier {
                *score *= 1.0 - self.controls.monthly_decay_rate;
            }
        }
    }
    
    pub fn calculate_average_reputation(&self) -> f64 {
        // Implement actual calculation
        0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalState {
    Draft,
    UnderReview,
    Voting,
    DisputeResolution,
    Finalized,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub initiator: String,
    pub reason: String, 
    pub mediators: Vec<String>,
    pub resolution: Option<String>,
    pub votes: HashMap<String, bool>,
    pub evidence: Vec<EvidenceItem>,
    pub id: String,
    pub proposal_id: String,
    pub timestamp: i64,
}

impl Proposal {
    pub fn transition_state(&mut self, new_state: ProposalState) -> Result<(), String> {
        let valid = match (&self.state, &new_state) {
            (ProposalState::Draft, ProposalState::UnderReview) => true,
            (ProposalState::UnderReview, ProposalState::Voting) => true,
            (ProposalState::Voting, ProposalState::DisputeResolution) => true,
            (ProposalState::DisputeResolution, ProposalState::Finalized) => true,
            _ => false,
        };

        if valid {
            self.state = new_state;
            Ok(())
        } else {
            Err("Invalid state transition".to_string())
        }
    }
    
    pub fn initiate_dispute(&mut self, initiator: String, reason: String) -> Result<(), String> {
        if self.state != ProposalState::Voting {
            return Err("Can only initiate dispute during voting phase".to_string());
        }
        
        if let Some(proof) = &self.zk_snark_proof {
            if !verify_proof(proof) {
                return Err("Invalid zk-SNARK proof".to_string());
            }
        }
        
        self.dispute_resolution = Some(DisputeResolution {
            initiator,
            reason,
            mediators: vec![],
            resolution: None,
            votes: HashMap::new(),
            evidence: vec![],
            id: String::new(),
            proposal_id: String::new(),
            timestamp: 0,
        });
        
        self.transition_state(ProposalState::DisputeResolution)
    }

    pub async fn execute_proposal(&self) -> bool {
        if let Some(proof) = &self.zk_snark_proof {
            if verify_proof(proof) {
                execute_approved_action();
                return true;
            }
        }
        false
    }

    pub fn calculate_voting_power(&self, federation: &Federation, cooperative_id: &str) -> f64 {
        match &self.voting_model {
            VotingModel::Simple => 1.0,
            VotingModel::Weighted { weight_factor } => {
                // Implement weighted voting calculation
                weight_factor * 1.0
            }
            VotingModel::Hybrid { governance_model, resource_model } => {
                // Implement hybrid voting calculation
                governance_model * 0.5 + resource_model * 0.5
            }
        }
    }

    pub fn validate_vote(&self, vote: &Vote) -> bool {
        if self.state != ProposalState::Voting {
            return false;
        }
        // Add more validation logic
        true
    }
}

fn verify_proof(proof: &str) -> bool {
    // Implement zk-SNARK proof verification logic
    true
}

fn execute_approved_action() {
    // Implement the action to be executed upon proposal approval
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reputation_decay() {
        let gov = GovernanceService::new(db_pool).await;
        
        // Test normal decay
        let result = gov.apply_decay("did:icn:test", 0.1).await;
        assert!(result.is_ok());
        
        // Test maximum decay limit
        let result = gov.apply_decay("did:icn:test", 0.9).await;
        assert!(result.is_err());
        
        // Test decay exemption
        let result = gov.apply_decay("did:icn:exempt", 0.1).await;
        assert!(result.is_ok());
        assert_eq!(gov.get_reputation("did:icn:exempt").await.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_voting_edge_cases() {
        let gov = GovernanceService::new(db_pool).await;

        // Test vote after period ends
        let proposal = gov.create_proposal("Test", "Description", "did:icn:test", 0).await.unwrap();
        let result = gov.vote(&proposal.id, "did:icn:voter", true).await;
        assert!(matches!(result, Err(GovernanceError::VotingPeriodEnded(_))));

        // Test double voting
        let proposal = gov.create_proposal("Test", "Description", "did:icn:test", 3600).await.unwrap();
        gov.vote(&proposal.id, "did:icn:voter", true).await.unwrap();
        let result = gov.vote(&proposal.id, "did:icn:voter", false).await;
        assert!(matches!(result, Err(GovernanceError::AlreadyVoted(_))));
    }
}
