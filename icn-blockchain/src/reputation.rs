use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ReputationSystem {
    scores: HashMap<String, i64>,  // Maps DID to its reputation score
}

impl ReputationSystem {
    /// Creates a new, empty ReputationSystem
    pub fn new() -> Self {
        ReputationSystem {
            scores: HashMap::new(),
        }
    }

    /// Increases reputation for a given DID
    pub fn increase_reputation(&mut self, did: &str, amount: i64) {
        let score = self.scores.entry(did.to_string()).or_insert(0);
        *score += amount;
        println!("Increased reputation for {} by {}. New score: {}", did, amount, *score);
    }

    /// Decreases reputation for a given DID
    pub fn decrease_reputation(&mut self, did: &str, amount: i64) {
        let score = self.scores.entry(did.to_string()).or_insert(0);
        *score -= amount;
        println!("Decreased reputation for {} by {}. New score: {}", did, amount, *score);
    }

    /// Retrieves the reputation score for a given DID
    pub fn get_reputation(&self, did: &str) -> i64 {
        *self.scores.get(did).unwrap_or(&0)
    }
}
