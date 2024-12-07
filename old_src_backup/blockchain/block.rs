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
    pub timestamp: u64,
    
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
    
    /// Total resources consumed by transactions in the block
    pub resources_used: u64,
    
    /// Size of the block in bytes
    pub size: u64,
    
    /// Summary of relationship transactions
    pub relationship_updates: RelationshipMetadata,
}

/// Metadata specific to relationship transactions in the block
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMetadata {
    pub contribution_count: u32,
    pub mutual_aid_count: u32,
    pub endorsement_count: u32,
    pub relationship_update_count: u32,
    pub total_participants: u32,
    pub unique_cooperatives: Vec<String>,
}

impl Block {
    /// Creates a new block with the given parameters
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>, proposer: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let relationship_metadata = Self::calculate_relationship_metadata(&transactions);
        let resources_used = transactions.iter().map(|tx| tx.resource_cost).sum();

        let metadata = BlockMetadata {
            consensus_duration_ms: 0,
            validator_count: 0,
            total_voting_power: 0.0,
            resources_used,
            size: 0,
            relationship_updates: relationship_metadata,
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

    /// Creates a genesis block
    pub fn genesis() -> Self {
        Block::new(
            0,
            String::from("0"),
            vec![],
            String::from("genesis")
        )
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
            .as_millis() as u64;

        if self.timestamp > current_time + 5000 { // Allow 5 second drift
            return Err("Block timestamp is in the future".to_string());
        }

        // Verify all transactions
        for tx in &self.transactions {
            if !tx.validate() {
                return Err(format!("Invalid transaction: {}", tx.hash));
            }
        }

        // Verify resource usage
        let calculated_resources: u64 = self.transactions.iter()
            .map(|tx| tx.resource_cost)
            .sum();
        if calculated_resources != self.metadata.resources_used {
            return Err("Resource usage mismatch".to_string());
        }

        // Verify relationship metadata
        let calculated_metadata = Self::calculate_relationship_metadata(&self.transactions);
        if calculated_metadata != self.metadata.relationship_updates {
            return Err("Relationship metadata mismatch".to_string());
        }

        Ok(())
    }

    /// Calculates metadata for relationship transactions in the block
    fn calculate_relationship_metadata(transactions: &[Transaction]) -> RelationshipMetadata {
        let mut metadata = RelationshipMetadata {
            contribution_count: 0,
            mutual_aid_count: 0,
            endorsement_count: 0,
            relationship_update_count: 0,
            total_participants: 0,
            unique_cooperatives: Vec::new(),
        };

        let mut participants = std::collections::HashSet::new();

        for tx in transactions {
            match &tx.transaction_type {
                super::TransactionType::RecordContribution { .. } => {
                    metadata.contribution_count += 1;
                    participants.insert(tx.sender.clone());
                }
                super::TransactionType::RecordMutualAid { receiver, .. } => {
                    metadata.mutual_aid_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(receiver.clone());
                }
                super::TransactionType::UpdateRelationship { member_two, .. } => {
                    metadata.relationship_update_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(member_two.clone());
                }
                super::TransactionType::AddEndorsement { to_did, .. } => {
                    metadata.endorsement_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(to_did.clone());
                }
                _ => {}
            }
        }

        metadata.total_participants = participants.len() as u32;

        metadata
    }

    /// Updates the block's metadata after consensus is reached
    pub fn update_metadata(&mut self, consensus_duration_ms: u64, size: u64) {
        self.metadata.consensus_duration_ms = consensus_duration_ms;
        self.metadata.size = size;
        self.metadata.resources_used = self.transactions.iter()
            .map(|tx| tx.resource_cost)
            .sum();
    }

    /// Gets the total resources used by all transactions in the block
    pub fn total_resources_used(&self) -> u64 {
        self.metadata.resources_used
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
        let mut block = create_test_block();
        assert!(block.verify(None).is_ok());

        // Test invalid hash
        block.hash = "invalid".to_string();
        assert!(block.verify(None).is_err());
    }

    #[test]
    fn test_relationship_metadata() {
        use super::super::TransactionType;
        
        let transactions = vec![
            Transaction::new(
                "did:icn:alice".to_string(),
                TransactionType::RecordContribution {
                    description: "test".to_string(),
                    impact_story: "test".to_string(),
                    context: "test".to_string(),
                    tags: vec!["test".to_string()],
                }
            ),
            Transaction::new(
                "did:icn:bob".to_string(),
                TransactionType::RecordMutualAid {
                    receiver: "did:icn:charlie".to_string(),
                    description: "test".to_string(),
                    impact_story: None,
                    reciprocity_notes: None,
                    tags: vec!["test".to_string()],
                }
            ),
        ];

        let block = Block::new(1, "hash".to_string(), transactions, "proposer".to_string());
        
        assert_eq!(block.metadata.relationship_updates.contribution_count, 1);
        assert_eq!(block.metadata.relationship_updates.mutual_aid_count, 1);
        assert_eq!(block.metadata.relationship_updates.total_participants, 3);
    }
}