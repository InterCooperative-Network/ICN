use serde::{Deserialize, Serialize};
use crate::credit_risk::CreditRiskManager;
use crate::types::{CooperativeId, FederationId};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralRequirement {
    pub amount: f64,
    pub collateral_type: CollateralType,
    pub percentage: f64,
    pub minimum_time_locked: i64,  // Duration in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollateralType {
    Tokens { token_id: String },
    Assets { asset_id: String },
    Labor { commitment_hours: u32 },
    FederationGuarantee { federation_id: FederationId },
}

pub struct CreditManager {
    risk_manager: CreditRiskManager,
    credit_limits: HashMap<CooperativeId, f64>,
    collateral_holdings: HashMap<CooperativeId, Vec<CollateralRequirement>>,
}

impl CreditManager {
    pub fn new(risk_manager: CreditRiskManager) -> Self {
        Self {
            risk_manager,
            credit_limits: HashMap::new(),
            collateral_holdings: HashMap::new(),
        }
    }

    pub fn calculate_credit_limit(
        &self,
        cooperative_id: &CooperativeId,
        base_amount: f64,
        credit_score: f64,
    ) -> f64 {
        self.risk_manager.get_borrowing_limit(credit_score, base_amount)
    }

    pub fn calculate_collateral_requirement(
        &self,
        cooperative_id: &CooperativeId,
        credit_amount: f64,
        credit_score: f64,
    ) -> CollateralRequirement {
        let percentage = self.risk_manager.calculate_required_collateral(credit_score);
        let amount = credit_amount * percentage;

        CollateralRequirement {
            amount,
            collateral_type: CollateralType::Tokens {
                token_id: "ICN".to_string(),
            },
            percentage,
            minimum_time_locked: 30 * 24 * 60 * 60, // 30 days in seconds
        }
    }

    pub fn verify_federation_guarantee(
        &self,
        federation_id: &FederationId,
        cooperative_id: &CooperativeId,
        amount: f64,
    ) -> bool {
        // Logic to verify federation members have voted to guarantee the credit
        // This would integrate with the federation's governance system
        true // Placeholder
    }

    pub fn lock_collateral(
        &mut self,
        cooperative_id: CooperativeId,
        requirement: CollateralRequirement,
    ) -> Result<(), String> {
        let holdings = self.collateral_holdings
            .entry(cooperative_id)
            .or_insert_with(Vec::new);
        
        holdings.push(requirement);
        Ok(())
    }

    pub fn release_collateral(
        &mut self,
        cooperative_id: &CooperativeId,
        amount: f64,
    ) -> Result<(), String> {
        if let Some(holdings) = self.collateral_holdings.get_mut(cooperative_id) {
            // Find and remove collateral that matches the amount
            if let Some(pos) = holdings.iter().position(|x| x.amount == amount) {
                holdings.remove(pos);
                return Ok(());
            }
        }
        Err("No matching collateral found".to_string())
    }
}
