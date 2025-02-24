use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[async_trait]
pub trait ReputationManager: Send + Sync {
    fn get_reputation(&self, participant: &str, context: &str) -> i64;
    fn is_eligible(&self, participant: &str, threshold: i64, context: &str) -> bool;
    async fn update_reputation(&self, participant: &str, change: i64, context: &str);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VoteStatus {
    Accepted,
    Rejected,
    Pending,
}

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Not eligible to vote")]
    NotEligibleToVote,
}

#[async_trait]
pub trait ConsensusEngine {
    async fn start(&self);
    async fn stop(&self);
    async fn submit_vote(&mut self, vote: Vote) -> Result<VoteStatus, GovernanceError>;
    async fn process_approved_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError>;
}
