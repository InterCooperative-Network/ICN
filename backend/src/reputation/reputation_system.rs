use std::collections::HashMap;

pub struct ReputationSystem {
    reputations: HashMap<String, i64>, // DID -> Reputation Score
}

impl ReputationSystem {
    pub fn new() -> Self {
        ReputationSystem {
            reputations: HashMap::new(),
        }
    }

    pub fn get_reputation_context(&self) -> HashMap<String, i64> {
        self.reputations.clone()
    }

    pub fn update_reputations(&mut self, updated_reputations: HashMap<String, i64>) {
        for (did, score) in updated_reputations {
            self.reputations.insert(did, score);
        }
    }

    pub fn get_reputation(&self, did: &str) -> i64 {
        self.reputations.get(did).copied().unwrap_or(0)
    }

    // Additional methods for reputation management...
}
