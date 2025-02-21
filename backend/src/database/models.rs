use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_by: String,
    pub ends_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: i64,
    pub voter: String,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contribution {
    pub id: i64,
    pub did: String,
    pub score: i64,
    pub timestamp: f64,
}
