use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorRules {
    max_validators_per_coop: u32,
    min_stake_requirement: u64,
    election_period_blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorState {
    pub coop_id: String,
    pub did: String,
    pub stake: u64,
    pub voting_power: u32,
    pub last_active: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteProposal {
    pub proposal_id: String,
    pub proposal_type: ProposalType,
    pub initiator_did: String,
    pub votes: HashSet<Vote>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    AddValidator(ValidatorInfo),
    RemoveValidator(String), // validator DID
    UpdateRules(ValidatorRules),
}

#[derive(Debug)]
pub struct GovernanceEngine {
    rules: ValidatorRules,
    active_validators: HashMap<String, ValidatorState>,
    active_proposals: HashMap<String, VoteProposal>,
}

impl GovernanceEngine {
    pub fn new(rules: ValidatorRules) -> Self {
        Self {
            rules,
            active_validators: HashMap::new(),
            active_proposals: HashMap::new(),
        }
    }

    pub fn can_propose_validator(&self, coop_id: &str) -> bool {
        let count = self.active_validators
            .values()
            .filter(|v| v.coop_id == coop_id)
            .count();
        count < self.rules.max_validators_per_coop as usize
    }

    pub fn validate_election_proposal(&self, proposal: &ElectionProposal) -> bool {
        // Ensure cooperative democratic process
        proposal.votes.len() >= proposal.required_votes
            && self.can_propose_validator(&proposal.coop_id)
    }

    pub fn submit_vote(&mut self, vote: Vote) -> Result<VoteStatus, GovernanceError> {
        let proposal = self.active_proposals.get_mut(&vote.proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        
        if !self.is_eligible_voter(&vote.voter_did) {
            return Err(GovernanceError::NotEligibleToVote);
        }

        proposal.votes.insert(vote);
        self.check_proposal_status(&proposal.proposal_id)
    }

    pub fn process_approved_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError> {
        let proposal = self.active_proposals.remove(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        match proposal.proposal_type {
            ProposalType::AddValidator(info) => self.add_validator(info),
            ProposalType::RemoveValidator(did) => self.remove_validator(&did),
            ProposalType::UpdateRules(rules) => {
                self.rules = rules;
                Ok(())
            }
        }
    }

    fn is_eligible_voter(&self, voter_did: &str) -> bool {
        // Placeholder logic to check if a voter is eligible
        true
    }

    fn check_proposal_status(&self, proposal_id: &str) -> Result<VoteStatus, GovernanceError> {
        // Placeholder logic to check the status of a proposal
        Ok(VoteStatus::Approved)
    }

    fn add_validator(&mut self, info: ValidatorInfo) -> Result<(), GovernanceError> {
        // Placeholder logic to add a validator
        Ok(())
    }

    fn remove_validator(&mut self, did: &str) -> Result<(), GovernanceError> {
        // Placeholder logic to remove a validator
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter_did: String,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub did: String,
    pub coop_id: String,
    pub public_key: String,
    pub stake: u64,
}

#[derive(Debug)]
pub enum VoteStatus {
    Approved,
    Rejected,
    Pending,
}

#[derive(Debug)]
pub enum GovernanceError {
    ProposalNotFound,
    NotEligibleToVote,
}
