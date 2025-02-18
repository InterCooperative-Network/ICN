use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub enum Transaction {
    CoopResource(ResourceTransaction),
    Governance(GovernanceTransaction),
    ValidatorElection(ElectionTransaction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceTransaction {
    pub resource_id: String,
    pub operation: ResourceOperation,
    pub metadata: ResourceMetadata,
    pub did_proof: DidProof,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub storage_type: StorageType,
    pub content_hash: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageType {
    OnChain,
    IPFS(String),
    Filecoin(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidProof {
    pub did: String,
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceTransaction {
    SubmitProposal(VoteProposal),
    CastVote(Vote),
    FinalizeProposal(String), // proposal_id
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageReference {
    pub location: StorageType,
    pub metadata: ResourceMetadata,
    pub access_control: AccessControl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessControl {
    pub owner_did: String,
    pub allowed_coops: HashSet<String>,
    pub permissions: Permissions,
}

impl Transaction {
    pub fn verify_did_auth(&self) -> Result<bool, AuthError> {
        match self {
            Transaction::CoopResource(tx) => Self::verify_did_signature(
                &tx.did_proof.did,
                &tx.did_proof.signature,
                &tx.resource_id
            ),
            Transaction::Governance(tx) => match tx {
                GovernanceTransaction::SubmitProposal(proposal) => {
                    Self::verify_did_signature(
                        &proposal.initiator_did,
                        &proposal.signature,
                        &proposal.proposal_id
                    )
                },
                // ... other governance verification cases
            },
            Transaction::ValidatorElection(tx) => {
                // ... validator election verification
                Ok(true)
            }
        }
    }

    fn verify_did_signature(did: &str, signature: &str, message: &str) -> Result<bool, AuthError> {
        // Implement DID signature verification using did:key or did:web
        // This is a placeholder for the actual verification logic
        Ok(true)
    }
}
