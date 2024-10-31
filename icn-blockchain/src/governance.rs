use chrono::Utc;
use std::collections::VecDeque;

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
    pub proposal_type: ProposalType,
    pub description: String,
    pub resource_amount: Option<u64>,
    pub duration: u64,
    pub status: ProposalStatus,
    votes: Vec<(String, i64)>, // Tuple of voter ID and vote weight
}

impl Proposal {
    pub fn new(id: u64, proposal_type: ProposalType, description: String) -> Self {
        Proposal {
            id,
            proposal_type,
            description,
            resource_amount: None,
            duration: 60,
            status: ProposalStatus::Open,
            votes: Vec::new(),
        }
    }

    pub fn validate(&self, expected_type: ProposalType) -> bool {
        self.status == ProposalStatus::Open && self.proposal_type == expected_type
    }

    pub fn vote(&mut self, voter_id: &str, weight: i64) {
        self.votes.push((voter_id.to_string(), weight));
    }

    pub fn total_votes(&self) -> i64 {
        self.votes.iter().map(|(_, weight)| weight).sum()
    }

    pub fn close(&mut self) {
        self.status = ProposalStatus::Closed;
    }

    pub fn check_and_notify(&self, time_remaining: u64) {
        if time_remaining <= 15 && self.status == ProposalStatus::Open {
            println!(
                "Notification: Proposal '{}' is nearing its end. Time remaining: {} minutes.",
                self.description, time_remaining
            );
        }
    }
}

#[derive(Debug)]
pub struct ProposalHistory {
    pub proposals: VecDeque<Proposal>,
    pub notifications: VecDeque<String>,
}

impl ProposalHistory {
    pub fn new() -> Self {
        ProposalHistory {
            proposals: VecDeque::new(),
            notifications: VecDeque::new(),
        }
    }

    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.proposals.push_back(proposal);
        self.notifications
            .push_back("New proposal created.".to_string());
    }

    pub fn close_proposal(&mut self, proposal_id: u64) {
        if let Some(proposal) = self.proposals.iter_mut().find(|p| p.id == proposal_id) {
            proposal.close();
            self.notifications
                .push_back(format!("Proposal '{}' has closed for voting", proposal.description));
        }
    }

    pub fn send_voting_reminder(&mut self) {
        for proposal in self.proposals.iter() {
            if proposal.status == ProposalStatus::Open {
                self.notifications.push_back(format!(
                    "Reminder: Proposal '{}' is still open for voting!",
                    proposal.description
                ));
            }
        }
    }

    pub fn display_history(&self) {
        for proposal in &self.proposals {
            println!(
                "Proposal ID: {}, Description: '{}', Status: {:?}, Total Votes: {}",
                proposal.id,
                proposal.description,
                proposal.status,
                proposal.total_votes()
            );
        }
    }
}
