use serde::{Serialize, Deserialize};
use icn_zkp::zk_snark;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationScore {
    pub score: i64,
}

impl ReputationScore {
    pub fn generate_proof(&self) -> Vec<u8> {
        // Generate zk-SNARK proof for the reputation score
        zk_snark::generate_proof(self.score)
    }

    pub fn verify_proof(proof: &[u8], expected_score: i64) -> bool {
        // Verify zk-SNARK proof for the reputation score
        zk_snark::verify_proof(proof, expected_score)
    }
}
