use std::collections::HashMap;
use std::sync::RwLock;
use icn_types::{MemberId, ReputationScore, ReputationError};
use chrono::{Utc};
use async_trait::async_trait;

#[async_trait]
pub trait ReputationInterface: Send + Sync {
    async fn get_reputation(&self, did: MemberId) -> Result<ReputationScore, ReputationError>;
    async fn add_contribution(&self, did: MemberId, contribution: ReputationScore) -> Result<(), ReputationError>;
    async fn apply_decay(&self, decay_rate: f64) -> Result<(), ReputationError>;
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
    async fn get_reputation(&self, did: MemberId) -> Result<ReputationScore, ReputationError> {
        let scores = self.scores.read().map_err(|_| ReputationError::LockError)?;
        Ok(scores.get(&did).copied().unwrap_or(ReputationScore::default()))
    }

    async fn add_contribution(&self, did: MemberId, contribution: ReputationScore) -> Result<(), ReputationError> {
        let mut scores = self.scores.write().map_err(|_| ReputationError::LockError)?;
        let score = scores.entry(did).or_insert(ReputationScore::default());
        score.add_contribution(contribution);
        Ok(())
    }

    async fn apply_decay(&self, decay_rate: f64) -> Result<(), ReputationError> {
        let mut scores = self.scores.write().map_err(|_| ReputationError::LockError)?;
        for score in scores.values_mut() {
            score.apply_decay(decay_rate);
        }
        Ok(())
    }
}
