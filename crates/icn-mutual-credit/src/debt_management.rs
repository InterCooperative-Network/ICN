use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use crate::types::{CooperativeId, DebtId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtStatus {
    pub debt_id: DebtId,
    pub cooperative_id: CooperativeId,
    pub amount: f64,
    pub due_date: DateTime<Utc>,
    pub stage: RecoveryStage,
    pub last_updated: DateTime<Utc>,
    pub resolution_attempts: Vec<ResolutionAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryStage {
    GracePeriod { ends_at: DateTime<Utc> },
    IncreasedInterest { rate: f64 },
    DisputeResolution { voting_ends_at: DateTime<Utc> },
    FinalPenalty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionAttempt {
    pub timestamp: DateTime<Utc>,
    pub action: ResolutionAction,
    pub outcome: ResolutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionAction {
    PaymentPlan { installments: u32, amount_per_installment: f64 },
    DebtRestructuring { new_terms: String },
    FederationVote { proposal_id: String },
    CollateralLiquidation { amount: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionOutcome {
    Pending,
    Approved,
    Rejected,
    Completed,
    Failed(String),
}

pub struct DebtManager {
    grace_period: Duration,
    resolution_config: ResolutionConfig,
}

struct ResolutionConfig {
    grace_period_days: i64,
    interest_increase_percent: f64,
    dispute_resolution_days: i64,
}

impl DebtManager {
    pub fn new() -> Self {
        Self {
            grace_period: Duration::days(14), // 14 day grace period
            resolution_config: ResolutionConfig {
                grace_period_days: 14,
                interest_increase_percent: 5.0,
                dispute_resolution_days: 7,
            },
        }
    }

    pub fn initiate_recovery(&self, debt: &mut DebtStatus) {
        let now = Utc::now();
        debt.stage = RecoveryStage::GracePeriod {
            ends_at: now + self.grace_period
        };
        debt.last_updated = now;
    }

    pub fn progress_recovery_stage(&self, debt: &mut DebtStatus) {
        let now = Utc::now();
        debt.stage = match &debt.stage {
            RecoveryStage::GracePeriod { ends_at } if now > *ends_at => {
                RecoveryStage::IncreasedInterest { 
                    rate: self.resolution_config.interest_increase_percent 
                }
            },
            RecoveryStage::IncreasedInterest { .. } => {
                RecoveryStage::DisputeResolution {
                    voting_ends_at: now + Duration::days(self.resolution_config.dispute_resolution_days)
                }
            },
            RecoveryStage::DisputeResolution { voting_ends_at } if now > *voting_ends_at => {
                RecoveryStage::FinalPenalty
            },
            _ => debt.stage.clone(),
        };
        debt.last_updated = now;
    }

    pub fn apply_resolution(&mut self, debt: &mut DebtStatus, action: ResolutionAction) -> ResolutionOutcome {
        let attempt = ResolutionAttempt {
            timestamp: Utc::now(),
            action: action.clone(),
            outcome: ResolutionOutcome::Pending,
        };
        
        let outcome = match action {
            ResolutionAction::PaymentPlan { .. } => {
                // Implement payment plan logic
                ResolutionOutcome::Approved
            },
            ResolutionAction::DebtRestructuring { .. } => {
                // Implement debt restructuring logic
                ResolutionOutcome::Pending
            },
            ResolutionAction::FederationVote { .. } => {
                // Implement federation voting logic
                ResolutionOutcome::Pending
            },
            ResolutionAction::CollateralLiquidation { .. } => {
                // Implement collateral liquidation logic
                ResolutionOutcome::Pending
            },
        };

        debt.resolution_attempts.push(ResolutionAttempt {
            timestamp: attempt.timestamp,
            action,
            outcome: outcome.clone(),
        });

        outcome
    }
}
