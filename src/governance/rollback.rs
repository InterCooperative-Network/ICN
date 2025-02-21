use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct DisputeInfo {
    pub initiator: String,
    pub reason: String,
    pub timestamp: SystemTime,
    pub evidence: Option<String>,
}

#[derive(Debug)]
pub enum RollbackError {
    ProposalNotFound,
    InvalidRollbackState,
    DisputeAlreadyExists,
    UnauthorizedRollback,
    TimeframePassed,
}

pub struct RollbackConfig {
    pub rollback_window: Duration,
    pub required_approvals: u32,
    pub cooling_period: Duration,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            rollback_window: Duration::from_secs(86400), // 24 hours
            required_approvals: 3,
            cooling_period: Duration::from_secs(3600), // 1 hour
        }
    }
}
