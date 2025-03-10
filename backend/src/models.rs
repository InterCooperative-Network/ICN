use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_by: String,
    pub ends_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub verifiable_credential: Option<String>,
    pub did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: i64,
    pub voter: String,
    pub approve: bool,
    pub verifiable_credential: Option<String>,
    pub zk_snark_proof: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub id: i64,
    pub contributor: String,
    pub description: String,
    pub impact_score: i64,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub admin_did: String,
    pub created_at: NaiveDateTime,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMember {
    pub federation_id: String,
    pub member_did: String,
    pub role: String,
    pub joined_at: NaiveDateTime,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooperation {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub admin_did: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub owner_did: String,
    pub resource_type: String,
    pub capacity: i64,
    pub usage: i64,
    pub status: String,
}
