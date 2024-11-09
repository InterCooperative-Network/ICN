// src/blockchain/transaction.rs

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
        input_data: HashMap<String, i64>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub transaction_type: TransactionType,
    pub timestamp: u128,
    pub hash: String,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl Transaction {
    pub fn new(sender: String, transaction_type: TransactionType) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = Self::calculate_transaction_hash(&sender, &transaction_type, timestamp);

        Transaction {
            sender,
            transaction_type,
            timestamp,
            hash,
            gas_limit: 21000, // Base gas limit for standard transactions
            gas_price: 1,     // Base gas price unit
        }
    }

    fn calculate_transaction_hash(sender: &str, transaction_type: &TransactionType, timestamp: u128) -> String {
        let mut hasher = Sha256::new();
        let transaction_data = match transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                format!("Transfer:{}:{}:{}", sender, receiver, amount)
            }
            TransactionType::ContractExecution { contract_id, input_data } => {
                format!("ContractExecution:{}:{:?}", contract_id, input_data)
            }
        };
        hasher.update(format!("{}{}{}", sender, transaction_data, timestamp));
        format!("{:x}", hasher.finalize())
    }

    pub fn gas_used(&self) -> u64 {
        match &self.transaction_type {
            TransactionType::Transfer { .. } => {
                // Base cost for transfer
                self.gas_limit.min(21000)
            }
            TransactionType::ContractExecution { input_data, .. } => {
                // Base cost plus data size cost
                let base_cost = 21000;
                let data_cost = input_data.len() as u64 * 68; // 68 gas per data item
                self.gas_limit.min(base_cost + data_cost)
            }
        }
    }

    pub fn validate(&self) -> bool {
        // Ensure sender is not empty
        if self.sender.is_empty() {
            return false;
        }

        // Validate based on transaction type
        match &self.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                !receiver.is_empty() && *amount > 0
            }
            TransactionType::ContractExecution { contract_id, input_data } => {
                !contract_id.is_empty() && !input_data.is_empty()
            }
        }
    }

    pub fn get_total_gas_cost(&self) -> u64 {
        self.gas_used() * self.gas_price
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.sender.as_bytes());
        bytes.extend(self.hash.as_bytes());
        bytes.extend(&self.timestamp.to_be_bytes());
        bytes
    }

    pub fn get_timestamp_ms(&self) -> u128 {
        self.timestamp
    }

    pub fn get_sender(&self) -> &str {
        &self.sender
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_transaction_creation() {
        let sender = "did:icn:sender".to_string();
        let transaction = Transaction::new(
            sender.clone(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        );

        assert_eq!(transaction.sender, sender);
        assert!(!transaction.hash.is_empty());
        assert!(transaction.validate());
    }

    #[test]
    fn test_contract_execution_transaction() {
        let mut input_data = HashMap::new();
        input_data.insert("param1".to_string(), 42);
        
        let transaction = Transaction::new(
            "did:icn:sender".to_string(),
            TransactionType::ContractExecution {
                contract_id: "contract123".to_string(),
                input_data,
            },
        );

        assert!(transaction.validate());
        assert!(transaction.gas_used() > 21000); // Should be more than base cost
    }

    #[test]
    fn test_invalid_transaction() {
        let transaction = Transaction::new(
            "".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 0,
            },
        );

        assert!(!transaction.validate());
    }

    #[test]
    fn test_gas_calculation() {
        let mut input_data = HashMap::new();
        input_data.insert("param1".to_string(), 42);
        input_data.insert("param2".to_string(), 43);
        
        let transaction = Transaction::new(
            "did:icn:sender".to_string(),
            TransactionType::ContractExecution {
                contract_id: "contract123".to_string(),
                input_data,
            },
        );

        assert!(transaction.gas_used() > 0);
        assert_eq!(transaction.get_total_gas_cost(), transaction.gas_used() * transaction.gas_price);
    }
}