// src/state/merkle_tree.rs
use sha2::{Digest, Sha256};

pub struct MerkleTree {
    leaves: Vec<String>,
    nodes: Vec<String>,
}

impl MerkleTree {
    pub fn new(data: Vec<String>) -> Self {
        // Hash all leaf data
        let leaves: Vec<String> = data.iter()
            .map(|d| Self::hash(d))
            .collect();

        // Create initial nodes from leaves
        let nodes = leaves.clone();

        MerkleTree { leaves, nodes }
    }

    pub fn get_root(&self) -> Option<String> {
        self.nodes.last().cloned()
    }

    pub fn verify(&self, data: &str, proof: &[String]) -> bool {
        let mut hash = Self::hash(data);

        for proof_element in proof {
            hash = Self::combine_hash(&hash, proof_element);
        }

        Some(hash) == self.get_root()
    }

    fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn combine_hash(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn build(&mut self) {
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                match chunk {
                    [left, right] => {
                        let combined = Self::combine_hash(left, right);
                        next_level.push(combined);
                    }
                    [left] => {
                        // Odd number of nodes, promote the last one
                        next_level.push(left.clone());
                    }
                    _ => unreachable!(),
                }
            }

            self.nodes.extend(next_level.clone());
            current_level = next_level;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let data = vec![
            "data1".to_string(),
            "data2".to_string(),
            "data3".to_string(),
            "data4".to_string(),
        ];

        let mut tree = MerkleTree::new(data);
        tree.build();

        assert!(tree.get_root().is_some());
    }
}