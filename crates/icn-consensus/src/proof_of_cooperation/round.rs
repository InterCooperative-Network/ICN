use icn_types::{Block, DID};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Initializing,
    ProposalPhase,
    VotingPhase,
    Committed,
    Failed,
}

#[derive(Debug)]
pub struct ConsensusRound {
    pub number: u64,
    pub status: RoundStatus,
    pub proposer: Option<DID>,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<DID, bool>,
    pub started_at: DateTime<Utc>,
}

impl ConsensusRound {
    pub fn new(number: u64, proposer: Option<DID>) -> Self {
        Self {
            number,
            status: RoundStatus::Initializing,
            proposer,
            proposed_block: None,
            votes: HashMap::new(),
            started_at: Utc::now(),
        }
    }

    pub fn add_vote(&mut self, validator: DID, approve: bool) -> bool {
        if self.status == RoundStatus::VotingPhase {
            self.votes.insert(validator, approve);
            true
        } else {
            false
        }
    }
}
