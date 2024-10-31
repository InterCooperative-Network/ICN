use chrono::{Utc, DateTime};

#[derive(Debug, Clone)]
pub struct Notification {
    pub proposal_id: u64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Notification {
    pub fn new(proposal_id: u64, message: String) -> Self {
        Notification {
            proposal_id,
            message,
            timestamp: Utc::now(),
        }
    }
}
