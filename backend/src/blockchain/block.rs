// src/blockchain/block.rs
use sha2::Digest;
use serde::{Serialize, Deserialize};
use super::Transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let mut hasher = sha2::Sha256::new();
        let transaction_data = serde_json::to_string(&transactions).unwrap_or_default();
        hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, transaction_data));
        let hash = format!("{:x}", hasher.finalize());

        Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash,
        }
    }
}