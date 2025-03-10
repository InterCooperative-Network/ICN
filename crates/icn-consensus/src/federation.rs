use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;
use std::time::Duration;
use icn_federation::{FederationType, FederationTerms};

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Invalid federation configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Federation already exists: {0}")]
    AlreadyExists(String),
    #[error("Insufficient reputation: {0}")]
    InsufficientReputation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Federation {
    pub id: String,
    pub federation_type: FederationType,
    pub terms: FederationTerms,
    pub members: Vec<String>,
    pub created_at: u64,
    pub status: FederationStatus,
    pub proposals: HashMap<i64, Proposal>,
    pub votes: HashMap<i64, Vec<Vote>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MemberStatus {
    Active,
    Pending,
    Suspended,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum FederationStatus {
    Forming,
    Active,
    Suspended,
    Dissolved,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_by: String,
    pub ends_at: u64,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Vote {
    pub proposal_id: i64,
    pub voter: String,
    pub approve: bool,
}

impl Federation {
    pub fn new(
        id: String,
        members: Vec<String>,
        federation_type: FederationType,
        terms: FederationTerms,
    ) -> Self {
        Self {
            id,
            members,
            federation_type,
            terms,
            created_at: chrono::Utc::now().timestamp() as u64,
            status: FederationStatus::Forming,
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

    pub fn can_join(&self, participant_reputation: i64) -> bool {
        participant_reputation >= self.terms.minimum_reputation
    }

    pub fn add_member(&mut self, member_id: String) -> Result<(), FederationError> {
        if self.members.contains(&member_id) {
            return Err(FederationError::AlreadyExists(
                "Member already exists in federation".to_string(),
            ));
        }
        self.members.push(member_id);
        Ok(())
    }

    pub fn submit_proposal(&mut self, title: String, description: String, created_by: String, ends_at: u64) -> Result<i64, FederationError> {
        let proposal_id = self.proposals.len() as i64 + 1;
        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            created_by,
            ends_at,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        self.proposals.insert(proposal_id, proposal);
        Ok(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: i64, voter: String, approve: bool) -> Result<(), FederationError> {
        if !self.proposals.contains_key(&proposal_id) {
            return Err(FederationError::InvalidConfiguration("Proposal not found".to_string()));
        }
        let vote = Vote {
            proposal_id,
            voter,
            approve,
        };
        self.votes.entry(proposal_id).or_insert_with(Vec::new).push(vote);
        Ok(())
    }
}
