use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReputationContext {
    Consensus,
    Governance,
    Resources,
    Membership,
    Contribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    pub raw_score: i64,
    pub weighted_score: f64,
    pub context_scores: HashMap<ReputationContext, i64>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSystem {
    scores: HashMap<String, ReputationScore>,
    context_multipliers: HashMap<ReputationContext, f64>,
}

impl ReputationSystem {
    pub fn new() -> Self {
        let mut context_multipliers = HashMap::new();
        context_multipliers.insert(ReputationContext::Consensus, 1.5);
        context_multipliers.insert(ReputationContext::Governance, 1.2);
        context_multipliers.insert(ReputationContext::Resources, 1.0);
        context_multipliers.insert(ReputationContext::Membership, 1.0);
        context_multipliers.insert(ReputationContext::Contribution, 1.1);

        Self {
            scores: HashMap::new(),
            context_multipliers,
        }
    }
}
