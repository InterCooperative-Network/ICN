mod proposals;

use proposals::{ProposalContract, Proposal};
use icn_zkp::RollupBatch;

pub struct CooperativeContract {
    proposal_contract: ProposalContract,
    // ...existing code...
}

impl CooperativeContract {
    pub fn new() -> Self {
        Self {
            proposal_contract: ProposalContract::new(3), // Minimum 3 votes required
            // ...existing code...
        }
    }

    pub fn submit_vote_batch(&mut self, batch: RollupBatch) -> Result<(), &'static str> {
        self.proposal_contract.submit_vote_batch(batch)
    }

    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<bool, &'static str> {
        self.proposal_contract.execute_proposal(proposal_id)
    }
}
