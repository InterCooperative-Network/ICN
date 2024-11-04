use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Transfer {
        receiver: String,
        amount: u64,
    },
    ContractExecution {
        contract_id: String,
        input_data: HashMap<String, i64>, // Optional input data for the contract
    },
    // ... other transaction types
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,       // DID of sender
    pub transaction_type: TransactionType,
    pub timestamp: u128,      // Transaction timestamp
    pub hash: String,         // Hash to secure transaction
}

impl Transaction {
    /// Creates a new transaction and calculates its hash
    pub fn new(sender: String, transaction_type: TransactionType) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = Self::calculate_transaction_hash(&sender, &transaction_type, timestamp);

        Transaction {
            sender,
            transaction_type,
            timestamp,
            hash,
        }
    }

    /// Calculate the transaction hash based on transaction details
    fn calculate_transaction_hash(sender: &str, transaction_type: &TransactionType, timestamp: u128) -> String {
        let mut hasher = Sha256::new();
        let transaction_data = match transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                format!("Transfer:{}:{}:{}", sender, receiver, amount)
            }
            TransactionType::ContractExecution { contract_id, input_data } => {
                format!("ContractExecution:{}:{:?}", contract_id, input_data)
            }
            // Handle other transaction types
        };
        hasher.update(format!("{}{}{}", sender, transaction_data, timestamp));
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
