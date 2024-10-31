// notifications.rs

use chrono::{Utc, DateTime};

#[derive(Debug, Clone)]
pub struct Notification {
    pub proposal_id: u64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Notification {
    /// Creates a new notification with the current timestamp.
    pub fn new(proposal_id: u64, message: String) -> Self {
        Notification {
            proposal_id,
            message,
            timestamp: Utc::now(),
        }
    }

    /// Displays a formatted message for the notification.
    pub fn display(&self) {
        println!(
            "[{}] Proposal ID {}: {}",
            self.timestamp, self.proposal_id, self.message
        );
    }
}
