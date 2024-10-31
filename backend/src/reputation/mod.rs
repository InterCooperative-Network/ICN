use std::collections::HashMap;

pub struct ReputationSystem {
    pub scores: HashMap<String, i64>,
}

impl ReputationSystem {
    /// Initializes a new Reputation System.
    pub fn new() -> Self {
        ReputationSystem {
            scores: HashMap::new(),
        }
    }

    /// Increases the reputation for a specific DID by a given amount.
    pub fn increase_reputation(&mut self, did: &str, amount: i64) {
        *self.scores.entry(did.to_string()).or_insert(0) += amount;
    }

    /// Decreases the reputation for a specific DID by a given amount.
    pub fn decrease_reputation(&mut self, did: &str, amount: i64) {
        *self.scores.entry(did.to_string()).or_insert(0) -= amount;
    }

    /// Retrieves the reputation score for a given DID. Defaults to 0 if no score exists.
    pub fn get_reputation(&self, did: &str) -> i64 {
        *self.scores.get(did).unwrap_or(&0)
    }

    /// Rewards a user for voting participation by increasing their reputation score.
    pub fn reward_voting(&mut self, did: &str, reward_points: i64) {
        self.increase_reputation(did, reward_points);
        println!(
            "Reputation for {} increased by {} points for voting participation.",
            did, reward_points
        );
    }
}