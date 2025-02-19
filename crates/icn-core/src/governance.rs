use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalState {
    Open,
    Closed,
    Finalized,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: String,
    pub description: String,
    pub state: ProposalState,
    pub votes: Vec<Vote>,
    pub final_result: Option<bool>, // true if accepted, false if rejected
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,
    pub approve: bool,
}

pub struct GovernanceManager {
    pub proposals: Vec<Proposal>,
}

impl GovernanceManager {
    pub fn new() -> Self {
        GovernanceManager { proposals: Vec::new() }
    }

    pub fn create_proposal(&mut self, id: String, description: String) -> Proposal {
        let proposal = Proposal {
            id: id.clone(),
            description,
            state: ProposalState::Open,
            votes: Vec::new(),
            final_result: None,
        };
        self.proposals.push(proposal.clone());
        proposal
    }

    pub fn cast_vote(&mut self, proposal_id: &str, vote: Vote) -> Result<(), String> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id);
        if let Some(prop) = proposal {
            if prop.state != ProposalState::Open {
                return Err("Voting is closed for this proposal".into());
            }
            if prop.votes.iter().any(|v| v.voter == vote.voter) {
                return Err("Voter has already voted".into());
            }
            // Basic role-based check: only DID-holding members allowed
            if !vote.voter.starts_with("did:") {
                return Err("Voter is not a valid DID holder".into());
            }
            prop.votes.push(vote);
            Ok(())
        } else {
            Err("Proposal not found".into())
        }
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<bool, String> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id);
        if let Some(prop) = proposal {
            if prop.state != ProposalState::Open {
                return Err("Proposal voting already closed".into());
            }
            prop.state = ProposalState::Closed;
            let total_votes = prop.votes.len();
            if total_votes == 0 {
                prop.final_result = Some(false);
                prop.state = ProposalState::Finalized;
                return Ok(false);
            }
            let votes_for = prop.votes.iter().filter(|v| v.approve).count();
            // Threshold: proposal accepted if more than 50% approvals.
            let accepted = votes_for > total_votes / 2;
            prop.final_result = Some(accepted);
            prop.state = ProposalState::Finalized;
            Ok(accepted)
        } else {
            Err("Proposal not found".into())
        }
    }
}
