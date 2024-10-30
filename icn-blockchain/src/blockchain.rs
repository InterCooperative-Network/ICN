use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

/// A struct representing a block in the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub data: String,
    pub hash: String,
}

impl Block {
    /// Creates a new block with a given index, previous hash, and data.
    /// Calculates the hash based on these parameters.
    pub fn new(index: u64, previous_hash: String, data: String) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = calculate_hash(index, &previous_hash, timestamp, &data);

        Block {
            index,
            previous_hash,
            timestamp,
            data,
            hash,
        }
    }
}

/// Calculates the hash for a block based on its contents.
fn calculate_hash(index: u64, previous_hash: &str, timestamp: u128, data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, data));
    let result = hasher.finalize();
    format!("{:x}", result)
}
