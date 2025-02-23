use groth16::{aggregate_proofs, verify_aggregate_proof};

pub struct BatchVerifier {
    proofs: Vec<SnarkProof>,
    max_batch_size: usize,
}

impl BatchVerifier {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            proofs: Vec::new(),
            max_batch_size,
        }
    }

    pub fn add_proof(&mut self, proof: SnarkProof) {
        self.proofs.push(proof);
        if self.proofs.len() >= self.max_batch_size {
            self.verify_batch();
        }
    }

    pub fn verify_batch(&mut self) -> bool {
        let batch = std::mem::take(&mut self.proofs);
        let aggregated = aggregate_proofs(&batch);
        verify_aggregate_proof(&aggregated)
    }
}
