use serde::{Serialize, Deserialize};
use icn_zkp::zk_snark;
use std::sync::Arc;
use async_trait::async_trait;
use icn_types::ReputationError;
use icn_storage::StorageInterface;
use tokio::sync::RwLock;

mod anti_monopoly;
use anti_monopoly::{AntiMonopolyManager, AntiMonopolyConfig};

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn generate_proof(&self) -> Vec<u8> {
        // Generate zk-SNARK proof for the reputation score
        zk_snark::generate_proof(self.score)
    }

    pub fn verify_proof(proof: &[u8], expected_score: i64) -> bool {
        // Verify zk-SNARK proof for the reputation score
        zk_snark::verify_proof(proof, expected_score)
    }
}

#[async_trait]
pub trait ReputationInterface: Send + Sync {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> Result<(), ReputationError>;
    async fn get_reputation(&self, member_id: &str) -> Result<i64, ReputationError>;
    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> Result<bool, ReputationError>;
    async fn get_voting_power(&self, member_id: &str) -> Result<f64, ReputationError>;
}

pub struct ReputationManager {
    store: Arc<dyn StorageInterface>,
    anti_monopoly: Arc<AntiMonopolyManager>,
    scores_cache: Arc<RwLock<std::collections::HashMap<String, ReputationScore>>>,
}

impl ReputationManager {
    pub fn new(store: Arc<dyn StorageInterface>) -> Self {
        Self {
            store,
            anti_monopoly: Arc::new(AntiMonopolyManager::new(AntiMonopolyConfig::default())),
            scores_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn with_config(store: Arc<dyn StorageInterface>, config: AntiMonopolyConfig) -> Self {
        Self {
            store,
            anti_monopoly: Arc::new(AntiMonopolyManager::new(config)),
            scores_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn load_score(&self, member_id: &str) -> Result<ReputationScore, ReputationError> {
        // Check cache first
        if let Some(score) = self.scores_cache.read().await.get(member_id) {
            return Ok(score.clone());
        }

        // Load from storage
        match self.store.get(&format!("reputation:{}", member_id)).await {
            Ok(bytes) => {
                let score: ReputationScore = bincode::deserialize(&bytes)
                    .map_err(|e| ReputationError { message: format!("Deserialization error: {}", e) })?;
                
                // Update cache
                self.scores_cache.write().await.insert(member_id.to_string(), score.clone());
                Ok(score)
            }
            Err(_) => Ok(ReputationScore::new(0))
        }
    }

    async fn save_score(&self, member_id: &str, score: ReputationScore) -> Result<(), ReputationError> {
        // Update cache
        self.scores_cache.write().await.insert(member_id.to_string(), score.clone());

        // Save to storage
        let bytes = bincode::serialize(&score)
            .map_err(|e| ReputationError { message: format!("Serialization error: {}", e) })?;
        
        self.store.store(&format!("reputation:{}", member_id), &bytes)
            .await
            .map_err(|e| ReputationError { message: e.to_string() })
    }

    pub async fn get_distribution_stats(&self) -> Result<anti_monopoly::ReputationDistribution, ReputationError> {
        Ok(self.anti_monopoly.get_distribution_stats().await)
    }

    pub async fn maintain(&self) -> Result<(), ReputationError> {
        // Run anti-monopoly maintenance
        self.anti_monopoly.maintain().await;

        // Update all scores in storage
        let mut cache = self.scores_cache.write().await;
        for (member_id, score) in cache.iter_mut() {
            let decayed_score = self.anti_monopoly.apply_decay(member_id, score.score).await;
            if decayed_score != score.score {
                score.score = decayed_score;
                score.last_update = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                let bytes = bincode::serialize(&score)
                    .map_err(|e| ReputationError { message: format!("Serialization error: {}", e) })?;
                
                self.store.store(&format!("reputation:{}", member_id), &bytes)
                    .await
                    .map_err(|e| ReputationError { message: e.to_string() })?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ReputationInterface for ReputationManager {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> Result<(), ReputationError> {
        let mut score = self.load_score(member_id).await?;
        
        // Apply anti-monopoly rules before updating
        let new_score = if delta > 0 {
            // For positive updates, check if member has too much influence
            if self.anti_monopoly.has_excessive_influence(member_id).await {
                // Apply reduced increase
                score.score + (delta as f64 * 0.5) as i64
            } else {
                score.score + delta
            }
        } else {
            score.score + delta
        };

        // Apply decay if necessary
        score.score = self.anti_monopoly.apply_decay(member_id, new_score).await;
        score.update_count += 1;
        score.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.save_score(member_id, score).await
    }

    async fn get_reputation(&self, member_id: &str) -> Result<i64, ReputationError> {
        Ok(self.load_score(member_id).await?.score)
    }

    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> Result<bool, ReputationError> {
        let score = self.load_score(member_id).await?.score;
        Ok(score >= min_required)
    }

    async fn get_voting_power(&self, member_id: &str) -> Result<f64, ReputationError> {
        let score = self.load_score(member_id).await?.score;
        Ok(self.anti_monopoly.calculate_voting_power(score))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockStorage {
        data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl StorageInterface for MockStorage {
        async fn store(&self, key: &str, value: &[u8]) -> Result<(), String> {
            self.data.write().await.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn get(&self, key: &str) -> Result<Vec<u8>, String> {
            self.data.read().await
                .get(key)
                .cloned()
                .ok_or_else(|| "Not found".to_string())
        }

        async fn delete(&self, key: &str) -> Result<(), String> {
            self.data.write().await.remove(key);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_reputation_updates() {
        let store = Arc::new(MockStorage::new());
        let manager = ReputationManager::new(store);

        // Test initial reputation
        assert_eq!(manager.get_reputation("test1").await.unwrap(), 0);

        // Test reputation increase
        manager.update_reputation("test1", 100).await.unwrap();
        assert_eq!(manager.get_reputation("test1").await.unwrap(), 100);

        // Test reputation decrease
        manager.update_reputation("test1", -50).await.unwrap();
        assert_eq!(manager.get_reputation("test1").await.unwrap(), 50);
    }

    #[tokio::test]
    async fn test_voting_power() {
        let store = Arc::new(MockStorage::new());
        let manager = ReputationManager::new(store);

        // Test initial voting power
        assert_eq!(manager.get_voting_power("test1").await.unwrap(), 0.0);

        // Test voting power with reputation
        manager.update_reputation("test1", 1000).await.unwrap();
        let power = manager.get_voting_power("test1").await.unwrap();
        assert!(power > 0.0 && power <= 100.0);
    }

    #[tokio::test]
    async fn test_anti_monopoly() {
        let store = Arc::new(MockStorage::new());
        let config = AntiMonopolyConfig {
            max_reputation: 1000,
            decay_rate: 0.1,
            decay_threshold: 0.8,
            ..Default::default()
        };
        let manager = ReputationManager::with_config(store, config);

        // Test reputation cap
        manager.update_reputation("test1", 900).await.unwrap();
        manager.update_reputation("test1", 200).await.unwrap();
        assert!(manager.get_reputation("test1").await.unwrap() < 1000);

        // Test decay
        manager.maintain().await.unwrap();
        assert!(manager.get_reputation("test1").await.unwrap() < 900);
    }
}
