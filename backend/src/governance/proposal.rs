use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalType {
    Funding,
    PolicyChange,
    ResourceAllocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub description: String,
    pub resource_amount: Option<u64>,
    pub duration: u64,
    pub status: ProposalStatus,
    pub required_reputation: i64, // Minimum reputation required to submit a proposal
    votes: Vec<(String, i64)>, // Tuple of voter ID and vote weight
}

impl Proposal {
    /// Initializes a new proposal with the given parameters.
    pub fn new(
        id: u64,
        proposal_type: ProposalType,
        description: String,
        required_reputation: i64,
        duration: u64,
    ) -> Self {
        Proposal {
            id,
            proposal_type,
            description,
            resource_amount: None,
            duration,
            status: ProposalStatus::Open,
            required_reputation,
            votes: Vec::new(),
        }
    }

    /// Validates the proposal type and ensures it's open for voting.
    pub fn validate(&self, expected_type: ProposalType) -> bool {
        self.status == ProposalStatus::Open && self.proposal_type == expected_type
    }

    /// Registers a vote with the given voter ID and weight.
    pub fn vote(&mut self, voter_id: &str, weight: i64) {
        self.votes.push((voter_id.to_string(), weight));
    }

    /// Calculates the total votes based on weight.
    pub fn total_votes(&self) -> i64 {
        self.votes.iter().map(|(_, weight)| weight).sum()
    }

    /// Closes the proposal, preventing further voting.
    pub fn close(&mut self) {
        self.status = ProposalStatus::Closed;
    }

    /// Checks if the proposal is nearing its closing time and sends a notification.
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
    pub proposals: HashMap<u64, Proposal>, // Track proposals by ID for easier access
    pub notifications: Vec<String>,
}

impl ProposalHistory {
    /// Initializes a new proposal history tracker.
    pub fn new() -> Self {
        ProposalHistory {
            proposals: HashMap::new(),
            notifications: Vec::new(),
        }
    }

    /// Adds a proposal to the history, generating a notification.
    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.proposals.insert(proposal.id, proposal);
        self.notifications.push("New proposal created.".to_string());
    }

    /// Closes a specific proposal by ID and notifies of closure.
    pub fn close_proposal(&mut self, proposal_id: u64) {
        if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
            proposal.close();
            self.notifications.push(format!(
                "Proposal '{}' has closed for voting",
                proposal.description
            ));
        }
    }

    /// Sends reminders for open proposals.
    pub fn send_voting_reminder(&mut self) {
        for proposal in self.proposals.values() {
            if proposal.status == ProposalStatus::Open {
                self.notifications.push(format!(
                    "Reminder: Proposal '{}' is still open for voting!",
                    proposal.description
                ));
            }
        }
    }

    /// Displays the proposal history with current vote counts.
    pub fn display_history(&self) {
        for proposal in self.proposals.values() {
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
