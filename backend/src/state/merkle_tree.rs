// backend/src/state/merkle_tree.rs

use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MerkleTree {
    leaves: Vec<String>,
    nodes: Vec<String>,
}

impl MerkleTree {
    pub fn new(data: Vec<String>) -> Self {
        let leaves = data.iter()
            .map(|d| Self::hash(d))
            .collect();
        let nodes = Self::build_tree(&leaves);
        
        MerkleTree { leaves, nodes }
    }

    pub fn add_leaf(&mut self, data: &str) {
        let hash = Self::hash(data);
        self.leaves.push(hash);
        self.nodes = Self::build_tree(&self.leaves);
    }

    pub fn root(&self) -> Option<&String> {
        self.nodes.first()
    }

    pub fn generate_proof(&self, index: usize) -> Vec<String> {
        if index >= self.leaves.len() {
            return vec![];
        }
        
        let mut proof = Vec::new();
        let mut idx = index + self.leaves.len() - 1;
        
        while idx > 0 {
            let sibling = if idx % 2 == 0 { idx - 1 } else { idx + 1 };
            if sibling < self.nodes.len() {
                proof.push(self.nodes[sibling].clone());
            }
            idx = (idx - 1) / 2;
        }
        
        proof
    }

    pub fn validate_proof(leaf: &str, root: &str, proof: Vec<String>) -> bool {
        let mut hash = Self::hash(leaf);
        
        for sibling in proof {
            hash = if hash < sibling {
                Self::hash(&format!("{}{}", hash, sibling))
            } else {
                Self::hash(&format!("{}{}", sibling, hash))
            };
        }
        
        &hash == root
    }

    fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn build_tree(leaves: &[String]) -> Vec<String> {
        if leaves.is_empty() {
            return vec![];
        }

        let mut nodes = leaves.to_vec();
        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in nodes.chunks(2) {
                let left = &chunk[0];
                let right = chunk.get(1).unwrap_or(left);
                next_level.push(Self::hash(&format!("{}{}", left, right)));
            }
            nodes = next_level;
        }
        nodes
    }
}