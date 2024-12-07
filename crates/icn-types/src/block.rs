use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub proposer: String,
    pub metadata: HashMap<String, String>,
}

impl Block {
    pub fn new(height: u64, previous_hash: String, proposer: String) -> Self {
        let timestamp = chrono::Utc::now();
        let mut block = Self {
            height,
            hash: String::new(),
            previous_hash,
            timestamp,
            proposer,
            metadata: HashMap::new(),
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}", 
            self.height,
            self.previous_hash,
            self.timestamp.timestamp(),
            self.proposer
        ));
        hex::encode(hasher.finalize())
    }
}
