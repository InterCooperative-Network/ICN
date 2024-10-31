// src/blockchain.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>, // Vector to hold transactions
    pub hash: String,
}

impl Block {
    /// Creates a new block with a list of transactions and calculates its hash.
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = calculate_hash(index, &previous_hash, timestamp, &transactions);

        Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash,
        }
    }
}

/// Calculates a hash for the block based on its contents.
fn calculate_hash(index: u64, previous_hash: &str, timestamp: u128, transactions: &Vec<Transaction>) -> String {
    let mut hasher = Sha256::new();
    let transaction_data = serde_json::to_string(transactions).expect("Failed to serialize transactions");
    hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, transaction_data));
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
}

impl Blockchain {
    /// Initializes a new blockchain with a genesis block.
    pub fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), vec![]);
        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: vec![],
            difficulty: 2,
        }
    }

    /// Adds a new transaction to the list of pending transactions.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    /// Finalizes a new block with pending transactions.
    pub fn finalize_block(&mut self) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
        );

        self.chain.push(new_block);
        self.pending_transactions.clear();
    }
}
