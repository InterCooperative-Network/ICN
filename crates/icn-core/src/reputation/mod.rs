use std::collections::HashMap;
use std::sync::RwLock;
use icn_types::{MemberId, ReputationScore};
use chrono::{Utc};
use async_trait::async_trait;

// Define a local error type to avoid conflicts with icn_types
#[derive(Debug, Clone)]
pub enum LocalReputationError {
    NotFound(String),
    InvalidUpdate(String),
    LockError,
}

impl std::fmt::Display for LocalReputationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalReputationError::NotFound(msg) => write!(f, "Not found: {}", msg),
            LocalReputationError::InvalidUpdate(msg) => write!(f, "Invalid update: {}", msg),
            LocalReputationError::LockError => write!(f, "Lock error"),
        }
    }
}

impl std::error::Error for LocalReputationError {}

// Create extension methods through a new struct
pub struct ReputationScoreExt;

impl ReputationScoreExt {
    pub fn default_score(member_id: MemberId) -> ReputationScore {
        ReputationScore {
            member_id,
            score: 0.0,
            last_updated: Utc::now(),
        }
    }
}

#[async_trait]
pub trait ReputationInterface: Send + Sync {
    async fn get_reputation(&self, did: MemberId) -> Result<ReputationScore, LocalReputationError>;
    async fn add_contribution(&self, did: MemberId, contribution: ReputationScore) -> Result<(), LocalReputationError>;
    async fn apply_decay(&self, decay_rate: f64) -> Result<(), LocalReputationError>;
}

pub struct ReputationSystem {
    scores: RwLock<HashMap<MemberId, ReputationScore>>,
}

impl ReputationSystem {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ReputationInterface for ReputationSystem {
    async fn get_reputation(&self, did: MemberId) -> Result<ReputationScore, LocalReputationError> {
        let scores = self.scores.read().map_err(|_| LocalReputationError::LockError)?;
        Ok(scores.get(&did).cloned().unwrap_or_else(|| 
            ReputationScoreExt::default_score(did)
        ))
    }

    async fn add_contribution(&self, did: MemberId, contribution: ReputationScore) -> Result<(), LocalReputationError> {
        let mut scores = self.scores.write().map_err(|_| LocalReputationError::LockError)?;
        let score = scores.entry(did.clone()).or_insert_with(|| 
            ReputationScoreExt::default_score(did)
        );
        
        let member_id = score.member_id.clone(); // Clone before the update
        *score = ReputationScore {
            member_id,
            score: score.score + contribution.score,
            last_updated: Utc::now(),
        };
        Ok(())
    }

    async fn apply_decay(&self, decay_rate: f64) -> Result<(), LocalReputationError> {
        let mut scores = self.scores.write().map_err(|_| LocalReputationError::LockError)?;
        for score in scores.values_mut() {
            let member_id = score.member_id.clone(); // Clone before the update
            *score = ReputationScore {
                member_id,
                score: score.score * (1.0 - decay_rate),
                last_updated: Utc::now(),
            };
        }
        Ok(())
    }
}
