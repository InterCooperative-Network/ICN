use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum ConsensusMessage {
    ProposalExecution {
        id: String,
        changes: Vec<String>,
        timestamp: DateTime<Utc>,
    }
}
