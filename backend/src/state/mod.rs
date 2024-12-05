// src/state/mod.rs
pub mod merkle_tree;

// src/state/merkle_tree.rs
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    leaves: Vec<String>,
    nodes: Vec<String>,
    height: usize,
}

impl MerkleTree {
    pub fn new(data: Vec<String>) -> Self {
        let leaves = data.iter().map(|d| Self::hash(d)).collect::<Vec<_>>();
        let nodes = Self::build_tree(&leaves);
        let height = if leaves.is_empty() { 0 } else { nodes.len().ilog2() as usize };
        MerkleTree {
            leaves,
            nodes,
            height,
        }
    }

    pub fn add_leaf(&mut self, data: &str) {
        let hash = Self::hash(data);
        self.leaves.push(hash.clone());
        self.nodes = Self::build_tree(&self.leaves);
        self.height = self.nodes.len().ilog2() as usize;
    }

    pub fn root(&self) -> Option<&String> {
        self.nodes.first()
    }

    pub fn generate_proof(&self, index: usize) -> Vec<String> {
        if index >= self.leaves.len() {
            return vec![];
        }
        let mut proof = vec![];
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

    pub fn validate_proof(data: &str, root: &str, proof: Vec<String>) -> bool {
        let mut hash = Self::hash(data);
        for sibling in proof {
            hash = if hash < sibling {
                Self::hash(&(hash + &sibling))
            } else {
                Self::hash(&(sibling + &hash))
            };
        }
        &hash == root
    }

    fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn build_tree(leaves: &[String]) -> Vec<String> {
        if leaves.is_empty() {
            return vec![];
        }

        let mut nodes = leaves.to_vec();
        while nodes.len() > 1 {
            let mut next_level = vec![];
            for i in (0..nodes.len()).step_by(2) {
                let left = &nodes[i];
                let right = if i + 1 < nodes.len() { &nodes[i + 1] } else { left };
                next_level.push(Self::hash(&(left.clone() + right)));
            }
            nodes = next_level;
        }
        nodes
    }
}