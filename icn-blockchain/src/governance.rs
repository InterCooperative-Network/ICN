use std::collections::{HashMap, VecDeque};  // Added VecDeque here
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

#[derive(Debug)]
pub struct ProposalHistory {
    proposals: VecDeque<Proposal>,
}

impl ProposalHistory {
    pub fn new() -> Self {
        ProposalHistory {
            proposals: VecDeque::new(),
        }
    }

    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.proposals.push_back(proposal);
    }

    pub fn display_history(&self) {
        for proposal in &self.proposals {
            println!(
                "Proposal ID: {}, Description: '{}', Result: {}, Total Votes: {}",
                proposal.id,
                proposal.description,
                match proposal.status {
                    ProposalStatus::Closed => "Closed",
                    ProposalStatus::Open => "Open",
                },
                proposal.total_votes()
            );
        }
    }

    /// Calculates the total votes cast on each proposal
    pub fn total_votes_for_proposals(&self) {
        for proposal in &self.proposals {
            let total_votes = proposal.total_votes();
            println!(
                "Proposal ID: {} - Total Votes Cast: {}",
                proposal.id, total_votes
            );
        }
    }

    /// Calculates participation rate based on a provided total member count
    pub fn participation_rate(&self, total_members: usize) {
        for proposal in &self.proposals {
            let unique_voters = proposal.votes.len();
            let participation_rate = (unique_voters as f64 / total_members as f64) * 100.0;
            println!(
                "Proposal ID: {} - Participation Rate: {:.2}%",
                proposal.id, participation_rate
            );
        }
    }

    /// Calculates positive vs. negative vote trend for each proposal
    pub fn voting_trends(&self) {
        for proposal in &self.proposals {
            let positive_votes: i64 = proposal.votes.values().filter(|&&v| v > 0).sum();
            let negative_votes: i64 = proposal.votes.values().filter(|&&v| v < 0).sum();
            println!(
                "Proposal ID: {} - Positive Votes: {}, Negative Votes: {}",
                proposal.id, positive_votes, negative_votes
            );
        }
    }
}
