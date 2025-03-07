use serde::{Serialize, Deserialize};
use icn_zkp::zk_snark;
use std::sync::Arc;
use async_trait::async_trait;
use icn_types::ReputationError;
use icn_storage::StorageInterface;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationScore {
    pub score: i64,
}

impl ReputationScore {
    pub fn generate_proof(&self) -> Vec<u8> {
        // Generate zk-SNARK proof for the reputation score
        zk_snark::generate_proof(self.score)
    }

    pub fn verify_proof(proof: &[u8], expected_score: i64) -> bool {
        // Verify zk-SNARK proof for the reputation score
        zk_snark::verify_proof(proof, expected_score)
    }
}

#[async_trait::async_trait]
pub trait ReputationInterface: Send + Sync {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> Result<(), ReputationError>;
    async fn get_reputation(&self, member_id: &str) -> Result<i64, ReputationError>;
    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> Result<bool, ReputationError>;
}

pub struct ReputationManager {
    store: Arc<dyn StorageInterface>,
}

impl ReputationManager {
    pub fn new(store: Arc<dyn StorageInterface>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl ReputationInterface for ReputationManager {
    async fn update_reputation(&self, member_id: &str, delta: i64) -> Result<(), ReputationError> {
        let current = self.get_reputation(member_id).await.unwrap_or(0);
        let new_score = current + delta;
        
        self.store.store(&format!("reputation:{}", member_id), &new_score.to_le_bytes())
            .await
            .map_err(|e| ReputationError { message: e.to_string() })
    }

    async fn get_reputation(&self, member_id: &str) -> Result<i64, ReputationError> {
        match self.store.get(&format!("reputation:{}", member_id)).await {
            Ok(bytes) => {
                let score = i64::from_le_bytes(bytes.try_into().map_err(|_| ReputationError {
                    message: "Invalid reputation data format".to_string()
                })?);
                Ok(score)
            }
            Err(_) => Ok(0) // New members start with 0 reputation
        }
    }

    async fn validate_reputation(&self, member_id: &str, min_required: i64) -> Result<bool, ReputationError> {
        let current = self.get_reputation(member_id).await?;
        Ok(current >= min_required)
    }
}
