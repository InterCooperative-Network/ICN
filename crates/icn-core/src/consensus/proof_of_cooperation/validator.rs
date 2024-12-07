// src/consensus/proof_of_cooperation/validator.rs

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};

use crate::consensus::types::{ValidatorInfo, ConsensusConfig, ConsensusError};
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};

/// Tracks validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    pub total_blocks_validated: u64,
    pub successful_proposals: u64,
    pub missed_rounds: u64,
    pub avg_response_time_ms: u64,
    pub last_active: DateTime<Utc>,
    pub performance_history: Vec<(DateTime<Utc>, f64)>,
}

impl Default for ValidatorMetrics {
    fn default() -> Self {
        ValidatorMetrics {
            total_blocks_validated: 0,
            successful_proposals: 0,
            missed_rounds: 0,
            avg_response_time_ms: 0,
            last_active: Utc::now(),
            performance_history: Vec::new(),
        }
    }
}

pub struct ValidatorManager {
    validators: HashMap<String, ValidatorInfo>,
    metrics: HashMap<String, ValidatorMetrics>,
    config: ConsensusConfig,
    total_voting_power: f64,
    last_cleanup: DateTime<Utc>,
    active_set: HashSet<String>,
    suspicious_validators: HashSet<String>,
}

impl ValidatorManager {
    pub fn new(config: ConsensusConfig) -> Self {
        ValidatorManager {
            validators: HashMap::new(),
            metrics: HashMap::new(),
            config,
            total_voting_power: 0.0,
            last_cleanup: Utc::now(),
            active_set: HashSet::new(),
            suspicious_validators: HashSet::new(),
        }
    }

    /// Registers a new validator with initial reputation
    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), ConsensusError> {
        // Validate DID format and uniqueness
        if self.validators.contains_key(&did) {
            return Err(ConsensusError::Custom("Validator already registered".to_string()));
        }

        // Create validator info
        let validator = ValidatorInfo {
            did: did.clone(),
            reputation: initial_reputation,
            voting_power: self.calculate_voting_power(initial_reputation),
            last_active_round: 0,
            consecutive_missed_rounds: 0,
            total_blocks_validated: 0,
            performance_score: 1.0,
        };

        // Initialize metrics
        self.metrics.insert(did.clone(), ValidatorMetrics::default());
        
        // Add to active set if meets minimum requirements
        if initial_reputation >= self.config.min_validator_reputation {
            self.active_set.insert(did.clone());
        }

        self.validators.insert(did, validator);
        self.update_total_voting_power();
        Ok(())
    }

    /// Updates validator statistics after a round
    pub fn update_validator_stats(
        &mut self,
        round_number: u64,
        votes: &HashMap<String, bool>,
        coordinator: &str,
    ) {
        let now = Utc::now();

        for (validator_id, validator) in self.validators.iter_mut() {
            let metrics = self.metrics.get_mut(validator_id).unwrap();
            
            if let Some(&approved) = votes.get(validator_id) {
                // Update participation metrics
                metrics.last_active = now;
                validator.consecutive_missed_rounds = 0;
                validator.last_active_round = round_number;
                validator.total_blocks_validated += 1;
                metrics.total_blocks_validated += 1;

                // Calculate performance score
                let performance = if approved { 1.0 } else { -0.5 };
                metrics.performance_history.push((now, performance));
                
                // Update reputation
                let reputation_change = self.calculate_reputation_change(
                    validator_id == coordinator,
                    approved,
                    validator.performance_score
                );
                
                validator.reputation += reputation_change;
                
                // Update performance score with exponential moving average
                validator.performance_score = validator.performance_score * 0.95 + 0.05 * performance;
            } else {
                // Handle missed round
                validator.consecutive_missed_rounds += 1;
                metrics.missed_rounds += 1;
                
                // Apply penalty
                let penalty = -(self.config.base_reward as f64 *
                    self.config.penalty_factor *
                    validator.consecutive_missed_rounds as f64) as i64;
                
                validator.reputation += penalty;
                validator.performance_score *= 0.95;

                // Check for suspicious behavior
                if validator.consecutive_missed_rounds > self.config.max_missed_rounds / 2 {
                    self.suspicious_validators.insert(validator_id.clone());
                }
            }

            // Maintain active set
            if validator.reputation >= self.config.min_validator_reputation &&
               validator.performance_score >= self.config.min_performance_score {
                self.active_set.insert(validator_id.clone());
            } else {
                self.active_set.remove(validator_id);
            }
        }

        self.update_total_voting_power();
    }

    /// Selects a coordinator for the next round
    pub fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, ConsensusError> 
    {
        if active_validators.is_empty() {
            return Err(ConsensusError::InsufficientValidators);
        }

        let mut rng = thread_rng();

        // Calculate selection weights based on reputation and performance
        let weights: Vec<f64> = active_validators.iter()
            .map(|v| (v.reputation as f64) * v.performance_score)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err(ConsensusError::Custom("No valid validators".to_string()));
        }

        // Probabilistic selection
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

    /// Checks and updates validator health
    pub fn cleanup_inactive_validators(&mut self) {
        let now = Utc::now();
        let cleanup_interval = Duration::hours(24);

        if (now - self.last_cleanup) >= cleanup_interval {
            // Remove validators that consistently underperform
            self.validators.retain(|did, v| {
                let metrics = self.metrics.get(did).unwrap();
                let active_recently = (now - metrics.last_active) < cleanup_interval;
                let meets_requirements = 
                    v.consecutive_missed_rounds < self.config.max_missed_rounds &&
                    v.performance_score >= self.config.min_performance_score;
                
                active_recently && meets_requirements
            });

            // Clean up old performance history
            for metrics in self.metrics.values_mut() {
                metrics.performance_history.retain(|(time, _)| 
                    (now - *time) < Duration::days(30)
                );
            }

            self.last_cleanup = now;
            self.update_total_voting_power();
        }
    }

    // Helper methods
    fn calculate_voting_power(&self, reputation: i64) -> f64 {
        let base_power = (reputation as f64) / 1000.0;
        base_power.min(self.config.max_voting_power)
    }

    fn calculate_reputation_change(&self, is_coordinator: bool, approved: bool, performance_score: f64) -> i64 {
        let base_reward = if is_coordinator {
            self.config.base_reward * 2 // Double reward for coordinator
        } else {
            self.config.base_reward
        };

        let performance_multiplier = if approved {
            1.0 + (performance_score * 0.5)
        } else {
            -0.5
        };

        (base_reward as f64 * performance_multiplier) as i64
    }

    fn update_total_voting_power(&mut self) {
        self.total_voting_power = self.validators.values()
            .map(|v| v.voting_power)
            .sum();
    }

    // Getters
    pub fn get_validator(&self, did: &str) -> Option<&ValidatorInfo> {
        self.validators.get(did)
    }

    pub fn get_validators(&self) -> &HashMap<String, ValidatorInfo> {
        &self.validators
    }

    pub fn get_active_validators(&self) -> Vec<&ValidatorInfo> {
        self.active_set.iter()
            .filter_map(|did| self.validators.get(did))
            .collect()
    }

    pub fn get_metrics(&self, did: &str) -> Option<&ValidatorMetrics> {
        self.metrics.get(did)
    }
}

impl EnergyAware for ValidatorManager {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        monitor.record_instruction();
        
        // Record validator state size
        let validator_size = (self.validators.len() * std::mem::size_of::<ValidatorInfo>()) as u64;
        monitor.record_memory_operation(validator_size);
        
        // Record metrics storage
        let metrics_size = (self.metrics.len() * std::mem::size_of::<ValidatorMetrics>()) as u64;
        monitor.record_storage_operation(metrics_size);
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
    fn test_validator_metrics() {
        let mut manager = setup_test_manager();
        let did = "did:icn:test".to_string();
        
        manager.register_validator(did.clone(), 1000).unwrap();
        
        // Create test votes
        let mut votes = HashMap::new();
        votes.insert(did.clone(), true);
        
        // Update stats
        manager.update_validator_stats(1, &votes, &did);
        
        let metrics = manager.get_metrics(&did).unwrap();
        assert_eq!(metrics.total_blocks_validated, 1);
        assert_eq!(metrics.missed_rounds, 0);
    }

    #[test]
    fn test_cleanup_inactive() {
        let mut manager = setup_test_manager();
        
        // Register test validators
        manager.register_validator("did:icn:active".to_string(), 1000).unwrap();
        manager.register_validator("did:icn:inactive".to_string(), 1000).unwrap();
        
        // Simulate inactivity
        if let Some(validator) = manager.validators.get_mut("did:icn:inactive") {
            validator.consecutive_missed_rounds = 10;
            validator.performance_score = 0.1;
        }
        
        // Force cleanup
        manager.last_cleanup = Utc::now() - Duration::hours(25);
        manager.cleanup_inactive_validators();
        
        assert!(manager.get_validator("did:icn:active").is_some());
        assert!(manager.get_validator("did:icn:inactive").is_none());
    }

    #[test]
    fn test_reputation_changes() {
        let mut manager = setup_test_manager();
        let did = "did:icn:test".to_string();
        
        manager.register_validator(did.clone(), 1000).unwrap();
        
        // Test successful validation
        let mut votes = HashMap::new();
        votes.insert(did.clone(), true);
        manager.update_validator_stats(1, &votes, &did);
        
        let validator = manager.get_validator(&did).unwrap();
        assert!(validator.reputation > 1000);
        
        // Test failed validation
        votes.insert(did.clone(), false);
        manager.update_validator_stats(2, &votes, &did);
        
        let validator = manager.get_validator(&did).unwrap();
        assert!(validator.reputation < 1000);
    }
}