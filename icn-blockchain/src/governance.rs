use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ProposalStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub description: String,
    pub votes: HashMap<String, i64>,  // DID to vote count (reputation-weighted)
    pub status: ProposalStatus,
}

impl Proposal {
    /// Creates a new proposal with an ID and description
    pub fn new(id: u64, description: String) -> Self {
        Proposal {
            id,
            description,
            votes: HashMap::new(),
            status: ProposalStatus::Open,
        }
    }

    /// Allows a DID to vote on the proposal with a reputation-weighted vote
    pub fn vote(&mut self, did: &str, reputation: i64) {
        if let ProposalStatus::Open = self.status {
            *self.votes.entry(did.to_string()).or_insert(0) += reputation;
            println!("DID {} voted with weight {}. Total votes: {:?}", did, reputation, self.votes);
        } else {
            println!("Proposal is closed. No further votes allowed.");
        }
    }

    /// Closes voting on the proposal and finalizes the result
    pub fn close(&mut self) {
        self.status = ProposalStatus::Closed;
        println!("Proposal {} is now closed for voting.", self.id);
    }

    /// Computes the total votes (sum of weighted votes)
    pub fn total_votes(&self) -> i64 {
        self.votes.values().sum()
    }
}
