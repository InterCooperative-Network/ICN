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