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

/// Extended reputation service with federation-specific methods
#[async_trait]
pub trait ReputationService: ReputationInterface + Send + Sync {
    /// Get a federation's reputation score
    async fn get_federation_reputation(&self, federation_id: &str) -> ReputationResult<i64>;
    
    /// Update a federation's reputation
    async fn update_federation_reputation(&self, federation_id: &str, delta: i64) -> ReputationResult<()>;
    
    /// Calculate aggregate reputation for a federation based on its members
    async fn calculate_federation_reputation(&self, federation_id: &str, member_ids: &[String]) -> ReputationResult<i64>;
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

#[async_trait]
impl ReputationService for SimpleReputationManager {
    async fn get_federation_reputation(&self, federation_id: &str) -> ReputationResult<i64> {
        // For simple implementation, just treat federation_id as a regular member_id
        self.get_reputation(federation_id).await
    }
    
    async fn update_federation_reputation(&self, federation_id: &str, delta: i64) -> ReputationResult<()> {
        // For simple implementation, just treat federation_id as a regular member_id
        self.update_reputation(federation_id, delta).await
    }
    
    async fn calculate_federation_reputation(&self, federation_id: &str, member_ids: &[String]) -> ReputationResult<i64> {
        // Simple implementation: average of all member reputations
        if member_ids.is_empty() {
            return self.get_federation_reputation(federation_id).await;
        }
        
        let mut total = 0;
        for member_id in member_ids {
            total += self.get_reputation(member_id).await?;
        }
        
        Ok(total / member_ids.len() as i64)
    }
}
