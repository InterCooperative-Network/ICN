use async_trait::async_trait;
use crate::reputation::ReputationManager;

#[async_trait]
pub trait ConsensusEngine {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn get_reputation(&self, did: &str, category: &str) -> Result<i64, String>;
    async fn is_eligible(&self, did: &str, min_reputation: i64, category: &str) -> Result<bool, String>;
    async fn dynamic_adjustment(&self, did: &str, contribution: i64) -> Result<(), String>;
    async fn apply_decay(&self, did: &str, decay_rate: f64) -> Result<(), String>;
    async fn reputation_based_access(&self, did: &str, min_reputation: i64) -> Result<bool, String>;
}

pub struct ProofOfCooperation {
    reputation_manager: ReputationManager,
}

impl ProofOfCooperation {
    pub fn new(reputation_manager: ReputationManager) -> Self {
        Self { reputation_manager }
    }

    pub async fn verify_validator_set(&self) -> Result<bool, String> {
        let total_validators = self.reputation_manager.count_eligible_validators(50, "consensus").await?;
        let max_faulty = total_validators / 3;
        
        // Verify we maintain BFT properties
        if total_validators < (3 * max_faulty + 1) {
            return Ok(false);
        }

        // Verify validator diversity
        let unique_organizations = self.reputation_manager.count_unique_organizations().await?;
        if unique_organizations < (2 * max_faulty + 1) {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn reputation_based_access(&self, did: &str, min_reputation: i64) -> Result<bool, String> {
        let reputation = self.reputation_manager.get_reputation(did, "consensus").await?;
        Ok(reputation >= min_reputation)
    }
}

#[async_trait]
impl ConsensusEngine for ProofOfCooperation {
    async fn start(&self) -> Result<(), String> {
        // Placeholder logic for starting the consensus engine
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        // Placeholder logic for stopping the consensus engine
        Ok(())
    }

    async fn get_reputation(&self, did: &str, category: &str) -> Result<i64, String> {
        self.reputation_manager.get_reputation(did, category).await
    }

    async fn is_eligible(&self, did: &str, min_reputation: i64, category: &str) -> Result<bool, String> {
        self.reputation_manager.is_eligible(did, min_reputation, category).await
    }

    async fn dynamic_adjustment(&self, did: &str, contribution: i64) -> Result<(), String> {
        self.reputation_manager.dynamic_adjustment(did, contribution).await
    }

    async fn apply_decay(&self, did: &str, decay_rate: f64) -> Result<(), String> {
        self.reputation_manager.apply_decay(did, decay_rate).await
    }

    async fn reputation_based_access(&self, did: &str, min_reputation: i64) -> Result<bool, String> {
        self.reputation_manager.reputation_based_access(did, min_reputation).await
    }
}
