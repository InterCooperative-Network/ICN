use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Invalid federation configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Federation already exists: {0}")]
    AlreadyExists(String),
    #[error("Insufficient reputation: {0}")]
    InsufficientReputation(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Federation {
    pub id: String,
    pub federation_type: FederationType,
    pub terms: FederationTerms,
    pub members: HashMap<String, MemberStatus>,
    pub created_at: u64,
    pub status: FederationStatus,
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

impl Federation {
    pub fn new(
        id: String,
        federation_type: FederationType,
        terms: FederationTerms,
        creator_id: String,
    ) -> Self {
        let mut members = HashMap::new();
        members.insert(creator_id, MemberStatus::Active);

        Self {
            id,
            federation_type,
            terms,
            members,
            created_at: chrono::Utc::now().timestamp() as u64,
            status: FederationStatus::Forming,
        }
    }

    pub fn can_join(&self, participant_reputation: i64) -> bool {
        participant_reputation >= self.terms.minimum_reputation
    }

    pub fn add_member(&mut self, member_id: String) -> Result<(), FederationError> {
        if self.members.contains_key(&member_id) {
            return Err(FederationError::AlreadyExists(
                "Member already exists in federation".to_string(),
            ));
        }
        self.members.insert(member_id, MemberStatus::Pending);
        Ok(())
    }
}
