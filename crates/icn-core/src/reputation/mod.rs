use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;
use icn_types::{MemberId, ReputationScore};
use chrono::{DateTime, Utc};

#[async_trait]
pub trait ReputationInterface: Send + Sync {
    async fn get_reputation(&self, did: String, category: String) -> Result<i64, Box<dyn std::error::Error>>;
    async fn dynamic_adjustment(&self, did: String, contribution: i64) -> Result<(), Box<dyn std::error::Error>>;
    async fn apply_decay(&self, did: String, decay_rate: f64) -> Result<(), Box<dyn std::error::Error>>;
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
}

pub struct ReputationManager {
    scores: RwLock<HashMap<MemberId, ReputationScore>>,
    categories: RwLock<HashMap<String, f64>>, // Category weights
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ReputationInterface for ReputationManager {
    async fn get_reputation(&self, did: String, _category: String) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(*self.scores.read().await.get(&did).unwrap_or(&0))
    }

    async fn dynamic_adjustment(&self, did: String, contribution: i64) -> Result<(), Box<dyn std::error::Error>> {
        let mut scores = self.scores.write().await;
        let score = scores.entry(did).or_insert(0);
        *score += contribution;
        Ok(())
    }

    async fn apply_decay(&self, did: String, decay_rate: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut scores = self.scores.write().await;
        if let Some(score) = scores.get_mut(&did) {
            *score = (*score as f64 * (1.0 - decay_rate)) as i64;
        }
        Ok(())
    }

    async fn start(&self) -> Result<(), String> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        Ok(())
    }
}

impl ReputationManager {
    pub async fn update_reputation(&self, member_id: MemberId, delta: f64, category: &str) -> Result<(), String> {
        let mut scores = self.scores.write().await;
        let categories = self.categories.read().await;
        
        let weight = categories.get(category).unwrap_or(&1.0);
        let weighted_delta = delta * weight;
        
        let score = scores.entry(member_id.clone()).or_insert(ReputationScore {
            member_id,
            score: 0.0,
            last_updated: Utc::now(),
        });
        
        score.score += weighted_delta;
        score.last_updated = Utc::now();
        
        Ok(())
    }

    pub async fn get_reputation(&self, member_id: &MemberId) -> Option<f64> {
        self.scores.read().await
            .get(member_id)
            .map(|score| score.score)
    }
}
