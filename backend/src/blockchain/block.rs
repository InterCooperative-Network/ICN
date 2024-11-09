// src/blockchain/block.rs

use std::time::SystemTime;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use super::Transaction;

/// Represents a block in the blockchain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Sequential index of this block in the chain
    pub index: u64,
    
    /// Hash of the previous block
    pub previous_hash: String,
    
    /// Unix timestamp in milliseconds when block was created
    pub timestamp: u128,
    
    /// List of transactions included in this block
    pub transactions: Vec<Transaction>,
    
    /// Hash of this block's contents
    pub hash: String,
    
    /// The DID of the validator that proposed this block
    pub proposer: String,
    
    /// Collection of validator signatures approving this block
    pub signatures: Vec<BlockSignature>,
    
    /// Metadata about the block creation
    pub metadata: BlockMetadata,
}

/// Signature from a validator approving a block
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockSignature {
    /// DID of the signing validator
    pub validator_did: String,
    
    /// The signature itself
    pub signature: String,
    
    /// Timestamp when signature was created
    pub timestamp: DateTime<Utc>,
    
    /// Voting power of the validator at time of signing
    pub voting_power: f64,
}

/// Additional metadata about block creation and validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Time taken to reach consensus (milliseconds)
    pub consensus_duration_ms: u64,
    
    /// Number of validators that participated
    pub validator_count: u32,
    
    /// Total voting power that approved the block
    pub total_voting_power: f64,
    
    /// Gas used by transactions in the block
    pub gas_used: u64,
    
    /// Size of the block in bytes
    pub size: u64,
}

impl Block {
    /// Creates a new block with the given parameters
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>, proposer: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let metadata = BlockMetadata {
            consensus_duration_ms: 0,
            validator_count: 0,
            total_voting_power: 0.0,
            gas_used: transactions.iter().map(|tx| tx.gas_used()).sum(),
            size: 0,
        };

        let mut block = Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash: String::new(),
            proposer,
            signatures: Vec::new(),
            metadata,
        };

        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of the block's contents
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Add block header fields
        hasher.update(self.index.to_string());
        hasher.update(&self.previous_hash);
        hasher.update(self.timestamp.to_string());
        
        // Add transaction hashes
        for tx in &self.transactions {
            hasher.update(&tx.hash);
        }
        
        // Add proposer
        hasher.update(&self.proposer);
        
        // Convert hash to hex string
        format!("{:x}", hasher.finalize())
    }

    /// Adds a validator's signature to the block
    pub fn add_signature(&mut self, validator_did: String, signature: String, voting_power: f64) -> bool {
        // Check if validator has already signed
        if self.signatures.iter().any(|s| s.validator_did == validator_did) {
            return false;
        }

        self.signatures.push(BlockSignature {
            validator_did,
            signature,
            timestamp: Utc::now(),
            voting_power,
        });

        // Update metadata
        self.metadata.validator_count = self.signatures.len() as u32;
        self.metadata.total_voting_power = self.signatures.iter()
            .map(|s| s.voting_power)
            .sum();

        true
    }

    /// Verifies the block's integrity
    pub fn verify(&self, previous_block: Option<&Block>) -> Result<(), String> {
        // Verify hash
        if self.hash != self.calculate_hash() {
            return Err("Invalid block hash".to_string());
        }

        // Verify previous hash and index if we have a parent block
        if let Some(prev) = previous_block {
            if self.previous_hash != prev.hash {
                return Err("Previous hash mismatch".to_string());
            }

            if self.index != prev.index + 1 {
                return Err("Invalid block index".to_string());
            }

            if self.timestamp <= prev.timestamp {
                return Err("Invalid timestamp".to_string());
            }
        }

        // Verify timestamp is not in the future
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        if self.timestamp > current_time + 5000 { // Allow 5 second drift
            return Err("Block timestamp is in the future".to_string());
        }

        // Verify all transactions
        for tx in &self.transactions {
            if !tx.validate() {
                return Err(format!("Invalid transaction: {}", tx.get_hash()));
            }
        }

        Ok(())
    }

    /// Updates the block's metadata after consensus is reached
    pub fn update_metadata(&mut self, consensus_duration_ms: u64, size: u64) {
        self.metadata.consensus_duration_ms = consensus_duration_ms;
        self.metadata.size = size;
        self.metadata.gas_used = self.transactions.iter()
            .map(|tx| tx.gas_used())
            .sum();
    }

    /// Gets the total gas used by all transactions in the block
    pub fn total_gas_used(&self) -> u64 {
        self.metadata.gas_used
    }

    /// Gets the number of transactions in the block
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Gets the block size in bytes
    pub fn size(&self) -> u64 {
        self.metadata.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block() -> Block {
        Block::new(
            1,
            "previous_hash".to_string(),
            vec![],
            "did:icn:proposer".to_string()
        )
    }

    #[test]
    fn test_block_creation() {
        let block = create_test_block();
        assert_eq!(block.index, 1);
        assert_eq!(block.previous_hash, "previous_hash");
        assert!(!block.hash.is_empty());
        assert_eq!(block.proposer, "did:icn:proposer");
    }

    #[test]
    fn test_block_hash_calculation() {
        let block = create_test_block();
        let hash = block.hash.clone();
        assert_eq!(block.calculate_hash(), hash);
    }

    #[test]
    fn test_signature_addition() {
        let mut block = create_test_block();
        assert!(block.add_signature(
            "did:icn:validator1".to_string(),
            "signature1".to_string(),
            0.5
        ));

        // Try adding same validator again
        assert!(!block.add_signature(
            "did:icn:validator1".to_string(),
            "signature2".to_string(),
            0.5
        ));

        assert_eq!(block.signatures.len(), 1);
        assert_eq!(block.metadata.validator_count, 1);
        assert_eq!(block.metadata.total_voting_power, 0.5);
    }

    #[test]
    fn test_block_verification() {
        let mut prev_block = create_test_block();
        prev_block.index = 0;

        let block = Block::new(
            1,
            prev_block.hash.clone(),
            vec![],
            "did:icn:proposer".to_string()
        );

        assert!(block.verify(Some(&prev_block)).is_ok());
    }

    #[test]
    fn test_invalid_previous_hash() {
        let prev_block = create_test_block();
        let block = Block::new(
            2,
            "wrong_hash".to_string(),
            vec![],
            "did:icn:proposer".to_string()
        );

        assert!(block.verify(Some(&prev_block)).is_err());
    }

    #[test]
    fn test_metadata_update() {
        let mut block = create_test_block();
        block.update_metadata(1000, 1024);

        assert_eq!(block.metadata.consensus_duration_ms, 1000);
        assert_eq!(block.metadata.size, 1024);
    }
}