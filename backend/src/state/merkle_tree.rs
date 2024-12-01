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
    /// Create a new Merkle tree with initial data
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

    /// Add a new leaf to the Merkle tree
    pub fn add_leaf(&mut self, data: &str) {
        let hash = Self::hash(data);
        self.leaves.push(hash.clone());
        self.nodes = Self::build_tree(&self.leaves);
        self.height = self.nodes.len().ilog2() as usize;
    }

    /// Get the root hash of the tree
    pub fn root(&self) -> Option<&String> {
        self.nodes.first()
    }

    /// Generate a proof for a given leaf
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

    /// Validate a proof for a given leaf and root
    pub fn validate_proof(leaf: &str, root: &str, proof: Vec<String>) -> bool {
        let mut hash = Self::hash(leaf);
        for sibling in proof {
            hash = if hash < sibling {
                Self::hash(&(hash + &sibling))
            } else {
                Self::hash(&(sibling + &hash))
            };
        }
        &hash == root
    }

    /// Helper function to hash data
    fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Build the tree nodes from leaves
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_creation() {
        let data = vec!["data1".to_string(), "data2".to_string(), "data3".to_string()];
        let tree = MerkleTree::new(data.clone());
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_merkle_tree_proof() {
        let data = vec!["data1".to_string(), "data2".to_string(), "data3".to_string()];
        let mut tree = MerkleTree::new(data.clone());

        let leaf = &data[1];
        let proof = tree.generate_proof(1);
        assert!(MerkleTree::validate_proof(leaf, tree.root().unwrap(), proof));
    }

    #[test]
    fn test_add_leaf() {
        let mut tree = MerkleTree::new(vec!["data1".to_string()]);
        tree.add_leaf("data2");
        assert!(tree.root().is_some());
        assert_eq!(tree.leaves.len(), 2);
    }
}
