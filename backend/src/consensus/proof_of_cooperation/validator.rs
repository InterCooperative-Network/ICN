use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use crate::consensus::types::{ValidatorInfo, ConsensusConfig, ConsensusError};

pub struct ValidatorManager {
    validators: HashMap<String, ValidatorInfo>,
    config: ConsensusConfig,
    total_voting_power: f64,
    last_cleanup: DateTime<Utc>,
}

impl ValidatorManager {
    pub fn new(config: ConsensusConfig) -> Self {
        ValidatorManager {
            validators: HashMap::new(),
            config,
            total_voting_power: 0.0,
            last_cleanup: Utc::now(),
        }
    }

    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), ConsensusError> {
        let validator = ValidatorInfo {
            did: did.clone(),
            reputation: initial_reputation,
            voting_power: self.calculate_voting_power(initial_reputation),
            last_active_round: 0,
            consecutive_missed_rounds: 0,
            total_blocks_validated: 0,
            performance_score: 1.0,
        };

        self.validators.insert(did, validator);
        self.update_total_voting_power();
        Ok(())
    }

    pub fn get_validator(&self, did: &str) -> Option<&ValidatorInfo> {
        self.validators.get(did)
    }

    pub fn get_validators(&self) -> &HashMap<String, ValidatorInfo> {
        &self.validators
    }

    pub fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, ConsensusError> 
    {
        let mut rng = thread_rng();

        let weights: Vec<f64> = active_validators.iter()
            .map(|v| (v.reputation as f64) * v.performance_score)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err(ConsensusError::Custom("No valid validators".to_string()));
        }

        let selection_point = rng.gen_range(0.0..total_weight);
        let mut cumulative_weight = 0.0;

        for (i, weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if cumulative_weight >= selection_point {
                return Ok(active_validators[i]);
            }
        }

        Err(ConsensusError::Custom("Failed to select coordinator".to_string()))
    }

    pub fn update_validator_stats(
        &mut self,
        round_number: u64,
        votes: &HashMap<String, bool>,
        coordinator: &str
    ) {
        for (validator_id, validator) in self.validators.iter_mut() {
            if let Some(&approved) = votes.get(validator_id) {
                // Reset consecutive misses
                validator.consecutive_missed_rounds = 0;
                validator.last_active_round = round_number;
                validator.total_blocks_validated += 1;

                // Calculate reward
                let base_reward = if validator_id == coordinator {
                    self.config.base_reward * 2 // Double reward for coordinator
                } else {
                    self.config.base_reward
                };

                let performance_multiplier = if approved {
                    1.0 + (validator.performance_score * 0.5)
                } else {
                    1.0
                };

                let reward = (base_reward as f64 * performance_multiplier) as i64;
                validator.reputation += reward;

                // Update performance score
                validator.performance_score = validator.performance_score * 0.95 + 0.05;
            } else {
                // Penalize non-participation
                validator.consecutive_missed_rounds += 1;
                
                let penalty = -(self.config.base_reward as f64 *
                    self.config.penalty_factor *
                    validator.consecutive_missed_rounds as f64) as i64;
                
                validator.reputation += penalty;
                validator.performance_score = validator.performance_score * 0.95;
            }
        }

        self.update_total_voting_power();
    }

    pub fn cleanup_inactive_validators(&mut self) {
        let now = Utc::now();
        if (now - self.last_cleanup).num_hours() >= 24 {
            self.validators.retain(|_, v| {
                v.consecutive_missed_rounds < self.config.max_missed_rounds &&
                v.performance_score >= self.config.min_performance_score
            });
            self.last_cleanup = now;
            self.update_total_voting_power();
        }
    }

    fn calculate_voting_power(&self, reputation: i64) -> f64 {
        let base_power = (reputation as f64) / 1000.0;
        base_power.min(self.config.max_voting_power)
    }

    fn update_total_voting_power(&mut self) {
        self.total_voting_power = self.validators.values()
            .map(|v| v.voting_power)
            .sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_manager() -> ValidatorManager {
        ValidatorManager::new(ConsensusConfig::default())
    }

    #[test]
    fn test_register_validator() {
        let mut manager = setup_test_manager();
        assert!(manager.register_validator(
            "did:icn:test".to_string(),
            1000
        ).is_ok());
        
        let validator = manager.get_validator("did:icn:test").unwrap();
        assert_eq!(validator.reputation, 1000);
    }

    #[test]
    fn test_validator_selection() {
        let mut manager = setup_test_manager();
        
        // Add some test validators
        for i in 1..=3 {
            manager.register_validator(
                format!("did:icn:test{}", i),
                1000
            ).unwrap();
        }

        let active_validators: Vec<_> = manager.validators.values().collect();
        let coordinator = manager.select_coordinator(&active_validators);
        assert!(coordinator.is_ok());
    }
}
