mod proposals;

use proposals::{ProposalContract, Proposal};
use icn_zkp::{RollupBatch, VerificationKey};
use ethers::prelude::*;

pub struct CooperativeContract {
    proposal_contract: ProposalContract,
    contract_address: Address,
    // ...existing code...
}

impl CooperativeContract {
    pub fn new(contract_address: Address, verification_key: VerificationKey) -> Self {
        Self {
            proposal_contract: ProposalContract::new(
                3,
                verification_key,
                contract_address
            ),
            contract_address,
            // ...existing code...
        }
    }

    pub async fn submit_vote_batch(&mut self, batch: RollupBatch) -> Result<(), String> {
        self.proposal_contract.submit_vote_batch(batch).await
    }

    pub async fn execute_proposal(&mut self, proposal_id: &str) -> Result<bool, String> {
        self.proposal_contract.execute_proposal(proposal_id).await
    }
}
