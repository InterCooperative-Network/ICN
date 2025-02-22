use std::collections::HashMap;
use icn_zkp::{ProofVerifier, RollupBatch, ZKProof};

pub struct ProposalContract {
    proposals: HashMap<String, Proposal>,
    vote_batches: Vec<RollupBatch>,
    min_quorum: u32,
    verifier: ProofVerifier,
}

pub struct Proposal {
    id: String,
    creator: String,
    voting_ends_at: u64,
    votes: HashMap<String, bool>,
    rollup_root: Option<[u8; 32]>,
    status: ProposalStatus,
    vote_count: VoteCount,
}

struct VoteCount {
    approve: u32,
    reject: u32,
    total: u32,
}

#[derive(PartialEq)]
enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Finalized,
}

impl ProposalContract {
    pub fn new(min_quorum: u32) -> Self {
        Self {
            proposals: HashMap::new(),
            vote_batches: Vec::new(),
            min_quorum,
            verifier: ProofVerifier::new(),
        }
    }

    pub fn submit_vote_batch(&mut self, batch: RollupBatch) -> Result<(), &'static str> {
        // Verify ZK proof for vote batch
        if !self.verifier.verify_proof(&batch.proof) {
            return Err("Invalid vote batch proof");
        }
        
        self.vote_batches.push(batch);
        Ok(())
    }

    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<bool, &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal not active");
        }

        // Process any pending vote batches
        self.process_vote_batches(proposal_id)?;

        // Check if voting period has ended
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time < proposal.voting_ends_at {
            return Err("Voting period still active");
        }

        // Tally final votes
        let approved = self.tally_votes(proposal);
        proposal.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        Ok(approved)
    }

    fn process_vote_batches(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        for batch in self.vote_batches.drain(..) {
            // Verify batch belongs to this proposal
            if batch.proposal_id != proposal_id {
                continue;
            }

            // Update vote counts from batch
            for vote in batch.votes {
                if !proposal.votes.contains_key(&vote.voter) {
                    proposal.vote_count.total += 1;
                    if vote.approve {
                        proposal.vote_count.approve += 1;
                    } else {
                        proposal.vote_count.reject += 1;
                    }
                    proposal.votes.insert(vote.voter, vote.approve);
                }
            }

            // Update rollup root
            proposal.rollup_root = Some(batch.rollup_root);
        }

        Ok(())
    }

    fn tally_votes(&self, proposal: &Proposal) -> bool {
        // Check quorum
        if proposal.vote_count.total < self.min_quorum {
            return false;
        }

        // Calculate approval percentage
        let approval_percentage = (proposal.vote_count.approve as f64 / proposal.vote_count.total as f64) * 100.0;
        approval_percentage >= 66.67 // 2/3 majority required
    }
}
