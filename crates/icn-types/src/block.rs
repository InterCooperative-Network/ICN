// File: crates/icn-types/src/block.rs
//
// Block type definitions and implementations for the Inter-Cooperative Network.
// This module contains the core block structure, block header, and related types
// used throughout the blockchain system.

use std::time::{SystemTime, UNIX_EPOCH};
use blake3::Hash;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::transaction::Transaction;
use crate::identity::DID;
use crate::Validate;

/// A block in the ICN blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header containing metadata
    pub header: BlockHeader,
    
    /// List of transactions in this block
    pub transactions: Vec<Transaction>,
    
    /// Block proposer's DID
    pub proposer: DID,
    
    /// Proposer's signature of the block header
    pub signature: Vec<u8>,
}

/// Block header containing block metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    
    /// Previous block hash
    pub prev_hash: Hash,
    
    /// Merkle root of transactions
    pub tx_root: Hash,
    
    /// Timestamp in seconds since UNIX epoch
    pub timestamp: u64,
    
    /// Block version
    pub version: u32,
    
    /// Additional block metadata
    pub metadata: BlockMetadata,
}

/// Additional block metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Total transactions in block
    pub tx_count: usize,
    
    /// Total size of block in bytes
    pub size: usize,
    
    /// Gas used by transactions
    pub gas_used: u64,
    
    /// Block proposer's reputation score
    pub proposer_reputation: i64,
    
    /// Network parameters hash at this block
    pub params_hash: Hash,
    
    /// State root after applying this block
    pub state_root: Hash,
}

impl Block {
    /// Creates a new block with the given parameters
    /// 
    /// # Arguments
    /// * `height` - Block height
    /// * `prev_hash` - Hash of the previous block
    /// * `transactions` - Vector of transactions to include
    /// * `proposer` - DID of the block proposer
    /// * `params_hash` - Hash of current network parameters
    /// * `state_root` - Expected state root after applying block
    pub fn new(
        height: u64,
        prev_hash: Hash,
        transactions: Vec<Transaction>,
        proposer: DID,
        params_hash: Hash,
        state_root: Hash,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let tx_root = Self::calculate_merkle_root(&transactions);
        
        let size = transactions.iter()
            .map(|tx| tx.encoded_size())
            .sum();
            
        let gas_used = transactions.iter()
            .map(|tx| tx.estimated_gas())
            .sum();
            
        let header = BlockHeader {
            height,
            prev_hash,
            tx_root,
            timestamp,
            version: 1,
            metadata: BlockMetadata {
                tx_count: transactions.len(),
                size,
                gas_used,
                proposer_reputation: 0, // Set by consensus
                params_hash,
                state_root,
            },
        };
        
        Self {
            header,
            transactions,
            proposer,
            signature: Vec::new(), // Set by proposer
        }
    }

    /// Calculates the cryptographic hash of this block
    pub fn hash(&self) -> Hash {
        let mut hasher = blake3::Hasher::new();
        
        // Add header fields
        hasher.update(&self.header.height.to_le_bytes());
        hasher.update(self.header.prev_hash.as_bytes());
        hasher.update(self.header.tx_root.as_bytes());
        hasher.update(&self.header.timestamp.to_le_bytes());
        hasher.update(&self.header.version.to_le_bytes());
        
        // Add metadata
        hasher.update(&self.header.metadata.tx_count.to_le_bytes());
        hasher.update(&self.header.metadata.size.to_le_bytes());
        hasher.update(&self.header.metadata.gas_used.to_le_bytes());
        hasher.update(&self.header.metadata.proposer_reputation.to_le_bytes());
        hasher.update(self.header.metadata.params_hash.as_bytes());
        hasher.update(self.header.metadata.state_root.as_bytes());
        
        // Add proposer
        hasher.update(self.proposer.as_bytes());
        
        // Add signature if present
        if !self.signature.is_empty() {
            hasher.update(&self.signature);
        }
        
        hasher.finalize()
    }

    /// Signs this block with the provided key pair
    /// 
    /// # Arguments
    /// * `key_pair` - The key pair to sign with
    pub fn sign(&mut self, key_pair: &crate::identity::KeyPair) -> Result<()> {
        let message = self.hash();
        self.signature = key_pair.sign(message.as_bytes())?;
        Ok(())
    }

    /// Verifies the block signature using the proposer's public key
    /// 
    /// # Arguments
    /// * `public_key` - The proposer's public key
    pub fn verify_signature(&self, public_key: &crate::identity::PublicKey) -> Result<bool> {
        if self.signature.is_empty() {
            return Ok(false);
        }

        let message = self.hash();
        Ok(public_key.verify(message.as_bytes(), &self.signature)?)
    }

    /// Calculates merkle root of the transaction list
    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::from([0; 32]);
        }

        // Get transaction hashes
        let mut hashes: Vec<Hash> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();

        // Build merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::with_capacity((hashes.len() + 1) / 2);
            
            for chunk in hashes.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(chunk[0].as_bytes());
                
                if chunk.len() > 1 {
                    hasher.update(chunk[1].as_bytes());
                } else {
                    hasher.update(chunk[0].as_bytes()); // Duplicate odd nodes
                }
                
                next_level.push(hasher.finalize());
            }
            
            hashes = next_level;
        }

        hashes[0]
    }

    /// Gets the total fees for all transactions in the block
    pub fn total_fees(&self) -> u64 {
        self.transactions.iter()
            .map(|tx| tx.fee)
            .sum()
    }

    /// Gets the block size in bytes
    pub fn size(&self) -> usize {
        self.header.metadata.size
    }

    /// Gets the total gas used by all transactions
    pub fn gas_used(&self) -> u64 {
        self.header.metadata.gas_used
    }

    /// Gets a reference to a transaction by index
    pub fn get_transaction(&self, index: usize) -> Option<&Transaction> {
        self.transactions.get(index)
    }

    /// Gets a reference to a transaction by hash
    pub fn get_transaction_by_hash(&self, hash: Hash) -> Option<&Transaction> {
        self.transactions.iter()
            .find(|tx| tx.hash() == hash)
    }
}

impl Validate for Block {
    fn validate(&self) -> Result<()> {
        // Validate block version
        if self.header.version == 0 {
            return Err(Error::Validation("Invalid block version".into()));
        }

        // Validate height
        if self.header.height == 0 && !self.header.prev_hash.as_bytes().iter().all(|&x| x == 0) {
            return Err(Error::Validation("Genesis block must have zero prev_hash".into()));
        }

        // Validate timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if self.header.timestamp > now + 60 {
            return Err(Error::Validation("Block timestamp too far in future".into()));
        }

        if self.header.timestamp == 0 {
            return Err(Error::Validation("Block timestamp cannot be zero".into()));
        }

        // Validate transactions
        if self.transactions.is_empty() {
            return Err(Error::Validation("Block must contain transactions".into()));
        }

        if self.header.metadata.tx_count != self.transactions.len() {
            return Err(Error::Validation("Transaction count mismatch".into()));
        }

        // Validate merkle root
        let calculated_root = Self::calculate_merkle_root(&self.transactions);
        if calculated_root != self.header.tx_root {
            return Err(Error::Validation("Invalid transaction merkle root".into()));
        }

        // Validate individual transactions
        for tx in &self.transactions {
            tx.validate()?;
        }

        // Validate metadata
        if self.header.metadata.size == 0 {
            return Err(Error::Validation("Block size cannot be zero".into()));
        }

        if self.header.metadata.params_hash == Hash::from([0; 32]) {
            return Err(Error::Validation("Parameters hash cannot be zero".into()));
        }

        if self.header.metadata.state_root == Hash::from([0; 32]) {
            return Err(Error::Validation("State root cannot be zero".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::KeyPair;
    
    fn create_test_transaction() -> Transaction {
        Transaction::new(
            DID::from("did:icn:test"),
            vec![0, 1, 2, 3],
            1000,
        )
    }

    fn create_test_block() -> Block {
        let transactions = vec![create_test_transaction()];
        let proposer = DID::from("did:icn:test");
        let params_hash = Hash::from([1; 32]);
        let state_root = Hash::from([2; 32]);
        
        Block::new(
            1,
            Hash::from([0; 32]),
            transactions,
            proposer,
            params_hash,
            state_root,
        )
    }

    #[test]
    fn test_block_creation() {
        let block = create_test_block();
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.header.prev_hash, Hash::from([0; 32]));
        assert_eq!(block.header.metadata.tx_count, 1);
        assert!(block.header.timestamp > 0);
        assert!(block.signature.is_empty());
    }

    #[test]
    fn test_block_hash() {
        let block = create_test_block();
        let hash1 = block.hash();
        
        // Same block should produce same hash
        assert_eq!(block.hash(), hash1);
        
        // Different blocks should have different hashes
        let mut block2 = create_test_block();
        block2.header.height = 2;
        assert_ne!(block2.hash(), hash1);
    }

    #[test]
    fn test_block_signing() {
        let key_pair = KeyPair::generate();
        let mut block = create_test_block();
        
        assert!(block.sign(&key_pair).is_ok());
        assert!(!block.signature.is_empty());
        assert!(block.verify_signature(&key_pair.public_key()).unwrap());
        
        // Wrong key should fail verification
        let wrong_key = KeyPair::generate().public_key();
        assert!(!block.verify_signature(&wrong_key).unwrap());
    }

    #[test]
    fn test_merkle_root() {
        let txs = vec![
            create_test_transaction(),
            create_test_transaction(),
            create_test_transaction(),
        ];
        
        let root = Block::calculate_merkle_root(&txs);
        assert_ne!(root, Hash::from([0; 32]));
        
        // Empty transaction list should have zero root
        let empty_root = Block::calculate_merkle_root(&[]);
        assert_eq!(empty_root, Hash::from([0; 32]));
        
        // Single transaction should work
        let single_root = Block::calculate_merkle_root(&[create_test_transaction()]);
        assert_ne!(single_root, Hash::from([0; 32]));
    }

    #[test]
    fn test_block_validation() {
        let mut block = create_test_block();
        assert!(block.validate().is_ok());
        
        // Test invalid cases
        let mut invalid = block.clone();
        invalid.header.metadata.tx_count = 0;
        assert!(invalid.validate().is_err());
        
        let mut invalid = block.clone();
        invalid.header.version = 0;
        assert!(invalid.validate().is_err());
        
        let mut invalid = block.clone();
        invalid.header.metadata.params_hash = Hash::from([0; 32]);
        assert!(invalid.validate().is_err());
        
        let mut invalid = block;
        invalid.header.metadata.state_root = Hash::from([0; 32]);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_block_transaction_access() {
        let block = create_test_block();
        let tx = block.get_transaction(0).unwrap();
        assert_eq!(tx.hash(), block.transactions[0].hash());
        
        assert!(block.get_transaction(1).is_none());
        
        let tx_hash = block.transactions[0].hash();
        let found_tx = block.get_transaction_by_hash(tx_hash).unwrap();
        assert_eq!(found_tx.hash(), tx_hash);
        
        assert!(block.get_transaction_by_hash(Hash::from([0; 32])).is_none());
    }

    #[test]
    fn test_block_fees_and_gas() {
        let block = create_test_block();
        assert_eq!(block.total_fees(), 1000);
        assert!(block.gas_used() > 0);
        assert!(block.size() > 0);
    }
}