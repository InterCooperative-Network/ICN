use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,       // DID of sender
    pub receiver: String,     // DID of receiver
    pub amount: u64,          // Amount/value exchanged
    pub timestamp: u128,      // Transaction timestamp
    pub hash: String,         // Hash to secure transaction
}

impl Transaction {
    /// Creates a new transaction and calculates its hash
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = calculate_transaction_hash(&sender, &receiver, amount, timestamp);

        Transaction {
            sender,
            receiver,
            amount,
            timestamp,
            hash,
        }
    }
}

/// Calculate the transaction hash based on transaction details
fn calculate_transaction_hash(sender: &str, receiver: &str, amount: u64, timestamp: u128) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}", sender, receiver, amount, timestamp));
    let result = hasher.finalize();
    format!("{:x}", result)
}