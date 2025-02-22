use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::{CooperativeId, FederationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditRiskScore {
    pub cooperative_id: CooperativeId,
    pub overall_score: f64,
    pub factors: RiskFactors,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    pub governance_participation: f64,  // 0-1 score based on voting history
    pub resource_contributions: f64,    // 0-1 score based on resource sharing
    pub credit_history: f64,           // 0-1 score based on repayment history
    pub network_endorsements: f64,     // 0-1 score based on federation endorsements
    pub age_factor: f64,               // 0-1 score based on cooperative age
}

pub struct CreditRiskManager {
    risk_scores: HashMap<CooperativeId, CreditRiskScore>,
    factor_weights: RiskWeights,
}

#[derive(Debug, Clone)]
pub struct RiskWeights {
    governance: f64,
    resources: f64,
    credit_history: f64,
    endorsements: f64,
    age: f64,
}

impl CreditRiskManager {
    pub fn new() -> Self {
        Self {
            risk_scores: HashMap::new(),
            factor_weights: RiskWeights {
                governance: 0.25,
                resources: 0.20,
                credit_history: 0.30,
                endorsements: 0.15,
                age: 0.10,
            },
        }
    }

    pub fn calculate_credit_score(&self, cooperative_id: &CooperativeId, factors: RiskFactors) -> f64 {
        let weights = &self.factor_weights;
        
        (factors.governance_participation * weights.governance) +
        (factors.resource_contributions * weights.resources) +
        (factors.credit_history * weights.credit_history) +
        (factors.network_endorsements * weights.endorsements) +
        (factors.age_factor * weights.age)
    }

    pub fn get_borrowing_limit(&self, credit_score: f64, base_limit: f64) -> f64 {
        let multiplier = match credit_score {
            score if score >= 0.8 => 2.0,   // High trust - 2x base limit
            score if score >= 0.6 => 1.5,   // Good standing - 1.5x base limit
            score if score >= 0.4 => 1.0,   // Average - base limit
            score if score >= 0.2 => 0.5,   // Below average - 0.5x base limit
            _ => 0.25,                      // Poor standing - 0.25x base limit
        };
        
        base_limit * multiplier
    }

    pub fn calculate_required_collateral(&self, credit_score: f64) -> f64 {
        match credit_score {
            score if score >= 0.8 => 0.20,  // 20% collateral for high trust
            score if score >= 0.6 => 0.30,  // 30% collateral for good standing
            score if score >= 0.4 => 0.40,  // 40% collateral for average
            score if score >= 0.2 => 0.50,  // 50% collateral for below average
            _ => 0.60,                      // 60% collateral for poor standing
        }
    }

    pub fn update_credit_score(&mut self, cooperative_id: CooperativeId, factors: RiskFactors) {
        let score = self.calculate_credit_score(&cooperative_id, factors.clone());
        let risk_score = CreditRiskScore {
            cooperative_id: cooperative_id.clone(),
            overall_score: score,
            factors,
            last_updated: chrono::Utc::now().timestamp(),
        };
        self.risk_scores.insert(cooperative_id, risk_score);
    }
}
