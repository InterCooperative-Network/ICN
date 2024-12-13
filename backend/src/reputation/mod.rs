pub mod reputation_system;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::identity::DID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    pub did: String,
    pub change: i64,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub context: String,
}

#[derive(Clone)]
pub struct ReputationSystem {
    pub scores: HashMap<String, i64>,
    history: HashMap<String, Vec<ReputationChange>>,
}

impl ReputationSystem {
    /// Initializes a new Reputation System.
    pub fn new() -> Self {
        ReputationSystem {
            scores: HashMap::new(),
            history: HashMap::new(),
        }
    }

    /// Increases the reputation for a specific DID by a given amount.
    pub fn increase_reputation(&mut self, did: &str, amount: i64) {
        let change = ReputationChange {
            did: did.to_string(),
            change: amount,
            timestamp: Utc::now(),
            reason: "Positive contribution".to_string(),
            context: "general".to_string(),
        };
        
        self.apply_reputation_change(did, change);
    }

    /// Decreases the reputation for a specific DID by a given amount.
    pub fn decrease_reputation(&mut self, did: &str, amount: i64) {
        let change = ReputationChange {
            did: did.to_string(),
            change: -amount,
            timestamp: Utc::now(),
            reason: "Negative action".to_string(),
            context: "general".to_string(),
        };
        
        self.apply_reputation_change(did, change);
    }

    /// Applies a reputation change with context and history tracking
    fn apply_reputation_change(&mut self, did: &str, change: ReputationChange) {
        let score = self.scores.entry(did.to_string()).or_insert(0);
        *score += change.change;
        
        // Ensure score doesn't go below zero
        if *score < 0 {
            *score = 0;
        }
        
        // Record the change in history
        self.history.entry(did.to_string())
            .or_insert_with(Vec::new)
            .push(change);
    }

    /// Retrieves the reputation score for a given DID. Defaults to 0 if no score exists.
    pub fn get_reputation(&self, did: &str) -> i64 {
        *self.scores.get(did).unwrap_or(&0)
    }

    /// Rewards a user for voting participation by increasing their reputation score.
    pub fn reward_voting(&mut self, did: &str, reward_points: i64) {
        let change = ReputationChange {
            did: did.to_string(),
            change: reward_points,
            timestamp: Utc::now(),
            reason: "Voting participation".to_string(),
            context: "governance".to_string(),
        };
        
        self.apply_reputation_change(did, change);
        
        println!(
            "Reputation for {} increased by {} points for voting participation.",
            did, reward_points
        );
    }

    /// Returns a clone of the reputation context
    pub fn get_reputation_context(&self) -> HashMap<String, i64> {
        self.scores.clone()
    }

    /// Updates the reputation context with changes from the VM execution
    pub fn update_reputations(&mut self, updated_reputations: &HashMap<String, i64>) {
        for (did, reputation) in updated_reputations {
            let change = ReputationChange {
                did: did.clone(),
                change: reputation - self.get_reputation(did),
                timestamp: Utc::now(),
                reason: "VM execution update".to_string(),
                context: "contract".to_string(),
            };
            
            self.apply_reputation_change(did, change);
        }
    }

    /// Gets the reputation history for a given DID
    pub fn get_history(&self, did: &str) -> Vec<ReputationChange> {
        self.history.get(did)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets the total reputation in the system
    pub fn total_reputation(&self) -> i64 {
        self.scores.values().sum()
    }

    /// Gets the top contributors by reputation score
    pub fn get_top_contributors(&self, limit: usize) -> Vec<(String, i64)> {
        let mut contributors: Vec<_> = self.scores.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        contributors.sort_by(|a, b| b.1.cmp(&a.1));
        contributors.truncate(limit);
        
        contributors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reputation_increase() {
        let mut system = ReputationSystem::new();
        system.increase_reputation("did:icn:test", 100);
        assert_eq!(system.get_reputation("did:icn:test"), 100);
    }
    
    #[test]
    fn test_reputation_decrease() {
        let mut system = ReputationSystem::new();
        system.increase_reputation("did:icn:test", 100);
        system.decrease_reputation("did:icn:test", 50);
        assert_eq!(system.get_reputation("did:icn:test"), 50);
    }
    
    #[test]
    fn test_reputation_floor() {
        let mut system = ReputationSystem::new();
        system.decrease_reputation("did:icn:test", 100);
        assert_eq!(system.get_reputation("did:icn:test"), 0);
    }
    
    #[test]
    fn test_reputation_history() {
        let mut system = ReputationSystem::new();
        system.increase_reputation("did:icn:test", 100);
        system.decrease_reputation("did:icn:test", 50);
        
        let history = system.get_history("did:icn:test");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].change, 100);
        assert_eq!(history[1].change, -50);
    }
    
    #[test]
    fn test_voting_reward() {
        let mut system = ReputationSystem::new();
        system.reward_voting("did:icn:test", 10);
        assert_eq!(system.get_reputation("did:icn:test"), 10);
    }
}
