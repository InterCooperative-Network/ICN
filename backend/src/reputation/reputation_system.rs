use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::identity::DID;

/// Represents a change in reputation with context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationChange {
    /// Amount of reputation changed (positive or negative)
    pub amount: i64,
    
    /// Reason for the reputation change
    pub reason: String,
    
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    
    /// Context or category of the change
    pub context: ReputationContext,
    
    /// DID of the entity that triggered the change
    pub trigger_did: Option<String>,
}

/// Different contexts where reputation can be earned or lost
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReputationContext {
    /// Consensus participation and validation
    Consensus,
    
    /// Governance participation (proposals, voting)
    Governance,
    
    /// Resource sharing and allocation
    Resources,
    
    /// Cooperative membership and participation
    Membership,
    
    /// General cooperative contributions
    Contribution,
}

/// Configuration for the reputation system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// Minimum possible reputation score
    pub min_reputation: i64,
    
    /// Maximum possible reputation score
    pub max_reputation: i64,
    
    /// Base amount earned for positive actions
    pub base_reward: i64,
    
    /// Base amount lost for negative actions
    pub base_penalty: i64,
    
    /// How quickly reputation decays (percentage per day)
    pub decay_rate: f64,
    
    /// Minimum time between reputation updates
    pub update_cooldown_ms: u64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        ReputationConfig {
            min_reputation: 0,
            max_reputation: 10000,
            base_reward: 10,
            base_penalty: 10,
            decay_rate: 0.01, // 1% per day
            update_cooldown_ms: 60000, // 1 minute
        }
    }
}

/// Main reputation system implementation
pub struct ReputationSystem {
    /// Current reputation scores for all DIDs
    scores: HashMap<String, i64>,
    
    /// History of reputation changes
    history: HashMap<String, Vec<ReputationChange>>,
    
    /// Last update timestamp for each DID
    last_updated: HashMap<String, DateTime<Utc>>,
    
    /// System configuration
    config: ReputationConfig,
    
    /// Context-specific multipliers
    context_multipliers: HashMap<ReputationContext, f64>,
}

impl ReputationSystem {
    /// Creates a new reputation system with default configuration
    pub fn new() -> Self {
        let mut context_multipliers = HashMap::new();
        context_multipliers.insert(ReputationContext::Consensus, 1.5);
        context_multipliers.insert(ReputationContext::Governance, 1.2);
        context_multipliers.insert(ReputationContext::Resources, 1.0);
        context_multipliers.insert(ReputationContext::Membership, 1.0);
        context_multipliers.insert(ReputationContext::Contribution, 1.1);

        ReputationSystem {
            scores: HashMap::new(),
            history: HashMap::new(),
            last_updated: HashMap::new(),
            config: ReputationConfig::default(),
            context_multipliers,
        }
    }

    /// Gets the current reputation score for a DID
    pub fn get_reputation(&self, did: &str) -> i64 {
        *self.scores.get(did).unwrap_or(&0)
    }

    /// Gets the reputation history for a DID
    pub fn get_history(&self, did: &str) -> Vec<ReputationChange> {
        self.history.get(did)
            .cloned()
            .unwrap_or_default()
    }

    /// Updates reputation with context and reason
    pub fn update_reputation(
        &mut self,
        did: &str,
        change: i64,
        context: ReputationContext,
        reason: String,
        trigger_did: Option<String>
    ) -> Result<i64, String> {
        // Check update cooldown
        let now = Utc::now();
        if let Some(last_update) = self.last_updated.get(did) {
            let elapsed = now.signed_duration_since(*last_update).num_milliseconds();
            if elapsed < self.config.update_cooldown_ms as i64 {
                return Err("Reputation update too soon".to_string());
            }
        }

        // Apply context multiplier
        let multiplier = self.context_multipliers
            .get(&context)
            .unwrap_or(&1.0);
        let adjusted_change = (change as f64 * multiplier) as i64;

        // Update score
        let current_score = self.get_reputation(did);
        let new_score = (current_score + adjusted_change)
            .max(self.config.min_reputation)
            .min(self.config.max_reputation);

        self.scores.insert(did.to_string(), new_score);
        self.last_updated.insert(did.to_string(), now);

        // Record change in history
        let change_record = ReputationChange {
            amount: adjusted_change,
            reason,
            timestamp: now,
            context,
            trigger_did,
        };

        self.history
            .entry(did.to_string())
            .or_insert_with(Vec::new)
            .push(change_record);

        Ok(new_score)
    }

    /// Increases reputation with context
    pub fn increase_reputation(
        &mut self,
        did: &str,
        amount: i64,
        context: ReputationContext,
        reason: String
    ) -> Result<i64, String> {
        self.update_reputation(did, amount.abs(), context, reason, None)
    }

    /// Decreases reputation with context
    pub fn decrease_reputation(
        &mut self,
        did: &str,
        amount: i64,
        context: ReputationContext,
        reason: String
    ) -> Result<i64, String> {
        self.update_reputation(did, -amount.abs(), context, reason, None)
    }

    /// Gets a copy of the current reputation context
    pub fn get_reputation_context(&self) -> HashMap<String, i64> {
        self.scores.clone()
    }

    /// Updates multiple reputation scores at once
    pub fn update_reputations(&mut self, updates: &HashMap<String, i64>) {
        for (did, score) in updates {
            self.scores.insert(did.clone(), *score);
            self.last_updated.insert(did.clone(), Utc::now());
        }
    }

    /// Applies reputation decay to all scores
    pub fn apply_decay(&mut self) {
        let now = Utc::now();
        
        for (did, score) in self.scores.iter_mut() {
            if let Some(last_update) = self.last_updated.get(did) {
                let days = now.signed_duration_since(*last_update).num_days() as f64;
                if days > 0.0 {
                    let decay_factor = (1.0 - self.config.decay_rate).powi(days as i32);
                    *score = (*score as f64 * decay_factor) as i64;
                    self.last_updated.insert(did.clone(), now);
                }
            }
        }
    }

    /// Checks if a DID meets a reputation threshold
    pub fn meets_threshold(&self, did: &str, threshold: i64) -> bool {
        self.get_reputation(did) >= threshold
    }

    /// Gets top contributors by reputation
    pub fn get_top_contributors(&self, limit: usize) -> Vec<(String, i64)> {
        let mut contributors: Vec<_> = self.scores.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        contributors.sort_by(|a, b| b.1.cmp(&a.1));
        contributors.truncate(limit);
        
        contributors
    }

    /// Calculates the total reputation in the system
    pub fn total_reputation(&self) -> i64 {
        self.scores.values().sum()
    }

    /// Updates the configuration
    pub fn update_config(&mut self, config: ReputationConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_system() -> ReputationSystem {
        ReputationSystem::new()
    }

    #[test]
    fn test_initial_reputation() {
        let system = setup_test_system();
        assert_eq!(system.get_reputation("test_did"), 0);
    }

    #[test]
    fn test_reputation_increase() {
        let mut system = setup_test_system();
        let result = system.increase_reputation(
            "test_did",
            100,
            ReputationContext::Consensus,
            "test increase".to_string()
        );
        
        assert!(result.is_ok());
        assert_eq!(system.get_reputation("test_did"), 150); // With 1.5x multiplier
    }

    #[test]
    fn test_reputation_decrease() {
        let mut system = setup_test_system();
        
        // First increase reputation
        system.increase_reputation(
            "test_did",
            100,
            ReputationContext::Consensus,
            "test increase".to_string()
        ).unwrap();
        
        // Then decrease it
        let result = system.decrease_reputation(
            "test_did",
            50,
            ReputationContext::Consensus,
            "test decrease".to_string()
        );
        
        assert!(result.is_ok());
        assert_eq!(system.get_reputation("test_did"), 75); // 150 - (50 * 1.5)
    }

    #[test]
    fn test_reputation_limits() {
        let mut system = setup_test_system();
        
        // Test maximum
        system.increase_reputation(
            "test_did",
            system.config.max_reputation * 2,
            ReputationContext::Consensus,
            "test max".to_string()
        ).unwrap();
        
        assert_eq!(system.get_reputation("test_did"), system.config.max_reputation);
        
        // Test minimum
        system.decrease_reputation(
            "test_did",
            system.config.max_reputation * 2,
            ReputationContext::Consensus,
            "test min".to_string()
        ).unwrap();
        
        assert_eq!(system.get_reputation("test_did"), system.config.min_reputation);
    }

    #[test]
    fn test_reputation_decay() {
        let mut system = setup_test_system();
        
        // Set initial reputation
        system.increase_reputation(
            "test_did",
            1000,
            ReputationContext::Consensus,
            "initial".to_string()
        ).unwrap();
        
        // Force last_updated to be one day ago
        let one_day_ago = Utc::now() - chrono::Duration::days(1);
        system.last_updated.insert("test_did".to_string(), one_day_ago);
        
        // Apply decay
        system.apply_decay();
        
        // Check that reputation has decayed by decay_rate
        let expected = (1500.0 * (1.0 - system.config.decay_rate)) as i64;
        assert_eq!(system.get_reputation("test_did"), expected);
    }

    #[test]
    fn test_top_contributors() {
        let mut system = setup_test_system();
        
        // Add some test data
        system.increase_reputation(
            "did1",
            100,
            ReputationContext::Consensus,
            "test".to_string()
        ).unwrap();
        
        system.increase_reputation(
            "did2",
            200,
            ReputationContext::Consensus,
            "test".to_string()
        ).unwrap();
        
        let top = system.get_top_contributors(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "did2");
        assert_eq!(top[1].0, "did1");
    }

    #[test]
    fn test_update_cooldown() {
        let mut system = setup_test_system();
        
        // First update should succeed
        let result1 = system.increase_reputation(
            "test_did",
            100,
            ReputationContext::Consensus,
            "test".to_string()
        );
        assert!(result1.is_ok());
        
        // Immediate second update should fail
        let result2 = system.increase_reputation(
            "test_did",
            100,
            ReputationContext::Consensus,
            "test".to_string()
        );
        assert!(result2.is_err());
    }
}
