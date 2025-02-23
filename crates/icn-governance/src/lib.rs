use thiserror::Error;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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

#[derive(Debug)]
pub struct DissolutionProtocol {
    federation_id: String,
    initiated_by: String,
    reason: DissolutionReason,
    status: DissolutionStatus,
    asset_distribution: HashMap<String, AssetAllocation>,
    debt_settlements: Vec<DebtSettlement>,
    member_reassignments: Vec<MemberReassignment>,
}

#[derive(Debug)]
pub enum DissolutionReason {
    Voluntary,
    InactivityThreshold,
    GovernanceViolation,
    EconomicNonviability,
}

#[derive(Debug)]
pub enum DissolutionStatus {
    Initiated,
    InProgress,
    Completed,
    Cancelled,
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
    
    // ...existing code...
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
}

impl Proposal {
    pub fn transition_state(&mut self, new_state: ProposalState) -> Result<(), String> {
        match (&self.state, &new_state) {
            (ProposalState::Draft, ProposalState::UnderReview) => Ok(()),
            (ProposalState::UnderReview, ProposalState::Voting) => Ok(()),
            (ProposalState::Voting, ProposalState::DisputeResolution) => Ok(()),
            (ProposalState::DisputeResolution, ProposalState::Finalized) => Ok(()),
            _ => Err("Invalid state transition".to_string())
        }
        self.state = new_state;
        Ok(())
    }
    
    pub fn initiate_dispute(&mut self, initiator: String, reason: String) -> Result<(), String> {
        if self.state != ProposalState::Voting {
            return Err("Can only initiate dispute during voting phase".to_string());
        }
        
        self.dispute_resolution = Some(DisputeResolution {
            initiator,
            reason,
            mediators: vec![],
            resolution: None,
            votes: HashMap::new(),
            evidence: vec![],
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
