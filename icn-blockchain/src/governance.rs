use std::collections::HashMap;
use chrono::{Utc, Duration};

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalType {
    Funding,
    PolicyChange,
    ResourceAllocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub description: String,
    pub proposal_type: ProposalType,
    pub votes: HashMap<String, i64>,
    pub status: ProposalStatus,
    pub resource_amount: Option<u64>,
    pub created_at: i64,
    pub duration_seconds: i64,
}

impl Proposal {
    pub fn new(id: u64, description: String, proposal_type: ProposalType, resource_amount: Option<u64>, duration_seconds: i64) -> Self {
        Proposal {
            id,
            description,
            proposal_type,
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            resource_amount,
            created_at: Utc::now().timestamp(),
            duration_seconds,
        }
    }

    pub fn vote(&mut self, did: &str, reputation: i64) {
        if self.status == ProposalStatus::Open {
            *self.votes.entry(did.to_string()).or_insert(0) += reputation;
            println!("DID {} voted with weight {}. Total votes: {:?}", did, reputation, self.votes);
        } else {
            println!("Proposal is closed. No further votes allowed.");
        }
    }

    pub fn close(&mut self) {
        self.status = ProposalStatus::Closed;
        println!("Proposal {} is now closed for voting.", self.id);
    }

    pub fn total_votes(&self) -> i64 {
        self.votes.values().sum()
    }

    pub fn validate(&self, expected_type: ProposalType) -> bool {
        self.status == ProposalStatus::Open && self.proposal_type == expected_type
    }

    pub fn check_status(&mut self) {
        let now = Utc::now().timestamp();
        if self.status == ProposalStatus::Open && (now - self.created_at) > self.duration_seconds {
            self.close();
        }
    }

    pub fn check_and_notify(&self, reminder_threshold: i64) {
        let now = Utc::now().timestamp();
        let time_left = self.duration_seconds - (now - self.created_at);

        if self.status == ProposalStatus::Open && time_left <= reminder_threshold {
            println!("Reminder: Proposal '{}' is closing soon! Time left: {} seconds", self.description, time_left);
        }
    }
}
