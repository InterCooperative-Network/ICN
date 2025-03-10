use serde::{Serialize, Deserialize};
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReputationScore {
    pub score: i64,
    pub last_update: u64,
    pub update_count: u32,
}

impl ReputationScore {
    pub fn new(score: i64) -> Self {
        Self {
            score,
            last_update: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            update_count: 1,
        }
    }
}

#[derive(Error, Debug)]
pub enum ReputationError {
    #[error("Reputation not found: {0}")]
    NotFound(String),
    
    #[error("Invalid reputation value: {0}")]
    InvalidValue(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type ReputationResult<T> = Result<T, ReputationError>;

/// Interface for reputation management systems
#[async_trait]
pub trait ReputationInterface: Send + Sync {
    /// Update a member's reputation
    async fn update_reputation(&self, member_id: &str, delta: i64) -> ReputationResult<()>;
    
    /// Get a member's current reputation
    async fn get_reputation(&self, member_id: &str) -> ReputationResult<i64>;
    
    /// Validate if a member has at least the minimum required reputation
    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> ReputationResult<bool>;
    
    /// Calculate voting power based on reputation
    async fn get_voting_power(&self, member_id: &str) -> ReputationResult<f64>;
}

// Simple in-memory reputation manager for testing
pub struct SimpleReputationManager {
    scores: RwLock<HashMap<String, i64>>,
}

impl SimpleReputationManager {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ReputationInterface for SimpleReputationManager {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> ReputationResult<()> {
        let mut scores = self.scores.write().await;
        let current = scores.entry(member_id.to_string()).or_insert(0);
        *current += delta;
        Ok(())
    }
    
    async fn get_reputation(&self, member_id: &str) -> ReputationResult<i64> {
        let scores = self.scores.read().await;
        Ok(*scores.get(member_id).unwrap_or(&0))
    }
    
    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> ReputationResult<bool> {
        let rep = self.get_reputation(member_id).await?;
        Ok(rep >= min_required)
    }
    
    async fn get_voting_power(&self, member_id: &str) -> ReputationResult<f64> {
        let rep = self.get_reputation(member_id).await?;
        // Simple linear voting power calculation
        Ok(rep as f64 / 100.0)
    }
}
