use std::collections::HashMap;
use icn_zkp::{ProofVerifier, RollupBatch, ZKProof, VerificationKey};
use ethers::prelude::*;

pub struct ProposalContract {
    proposals: HashMap<String, Proposal>,
    vote_batches: Vec<RollupBatch>,
    min_quorum: u32,
    verifier: ProofVerifier,
    verification_key: VerificationKey,
    contract_address: Address,
    client: Provider<Http>,
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
    pub fn new(min_quorum: u32, verification_key: VerificationKey, contract_address: Address) -> Self {
        Self {
            proposals: HashMap::new(),
            vote_batches: Vec::new(),
            min_quorum,
            verifier: ProofVerifier::new(),
            verification_key,
            contract_address,
            client: Provider::<Http>::try_from(
                "http://localhost:8545"
            ).expect("could not instantiate HTTP Provider"),
        }
    }

    pub async fn submit_vote_batch(&mut self, batch: RollupBatch) -> Result<(), String> {
        // First verify the ZK proof locally
        if !self.verifier.verify_proof(&batch.proof) {
            return Err("Invalid vote batch proof".to_string());
        }

        // Create contract call to submit batch
        let data = ethers::abi::encode(&[
            Token::Bytes(batch.rollup_root.to_vec()),
            Token::Bytes(batch.proof.to_vec())
        ]);

        let tx = TransactionRequest::new()
            .to(self.contract_address)
            .data(data)
            .into();

        // Submit transaction
        match self.client.send_transaction(tx, None).await {
            Ok(tx_hash) => {
                // Wait for confirmation
                let receipt = self.client.get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("Transaction not found")?;

                if receipt.status.unwrap() == U64::from(1) {
                    self.vote_batches.push(batch);
                    Ok(())
                } else {
                    Err("Transaction failed".to_string())
                }
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn execute_proposal(&mut self, proposal_id: &str) -> Result<bool, String> {
        // Create call to execute proposal on-chain
        let data = ethers::abi::encode(&[Token::String(proposal_id.to_string())]);

        let tx = TransactionRequest::new()
            .to(self.contract_address)
            .data(data)
            .into();

        match self.client.send_transaction(tx, None).await {
            Ok(tx_hash) => {
                let receipt = self.client.get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("Transaction not found")?;

                // Parse result from logs
                if let Some(logs) = receipt.logs.get(0) {
                    let topics = logs.topics.clone();
                    if topics.len() >= 2 {
                        let approved = topics[1] == H256::from([1u8; 32]);
                        
                        // Update local state
                        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
                            proposal.status = if approved {
                                ProposalStatus::Approved
                            } else {
                                ProposalStatus::Rejected
                            };
                        }
                        
                        return Ok(approved);
                    }
                }
                Err("Could not parse result".to_string())
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn handle_zk_snark_proof_verification(&self, proof: &ZKProof) -> Result<bool, String> {
        if self.verifier.verify_proof(proof) {
            Ok(true)
        } else {
            Err("Invalid zk-SNARK proof".to_string())
        }
    }
}
