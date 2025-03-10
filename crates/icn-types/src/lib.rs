//! Core types for the ICN (Inter-Cooperative Network) system
//! 
//! This crate provides the fundamental types used across all ICN modules.
//! It serves as a central repository for shared data structures, ensuring
//! consistency across the codebase.

use std::time::SystemTime;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::task;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use thiserror::Error;
use async_trait::async_trait;

mod errors;
pub use errors::{IcnError, IcnResult, log_error};

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Invalid block index")]
    InvalidIndex,
    #[error("Invalid previous hash")]
    InvalidPreviousHash,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Invalid proposer")]
    InvalidProposer,
    #[error("Consensus error")]
    ConsensusError,
    #[error("Database error")]
    DatabaseError,
}

#[derive(Debug)]
pub struct ResourceDebt {
    pub cpu_debt: u64,
    pub memory_debt: u64,
    pub bandwidth_debt: u64,
}

/// Represents a block in the blockchain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub hash: String,
    pub proposer: String,
    pub metadata: BlockMetadata,
    pub signatures: Vec<BlockSignature>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockSignature {
    pub validator_did: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub voting_power: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub height: u64,
    pub state_root: String,
    pub transaction_root: String,
    pub consensus_data: HashMap<String, String>,
    pub validator_signatures: Vec<String>,
    pub consensus_duration_ms: u64,
    pub validator_count: u32,
    pub total_voting_power: f64,
    pub size: u64,
    pub resources_used: u64,
    pub relationship_updates: Option<RelationshipMetadata>,
    pub fault_tolerance: Option<u32>,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self {
            height: 0,
            state_root: String::new(),
            transaction_root: String::new(),
            consensus_data: HashMap::new(),
            validator_signatures: Vec::new(),
            consensus_duration_ms: 0,
            validator_count: 0,
            total_voting_power: 0.0,
            size: 0,
            resources_used: 0,
            relationship_updates: None,
            fault_tolerance: None,
        }
    }
}

impl BlockMetadata {
    pub fn with_bft_info(&mut self, quorum_size: u32, fault_tolerance: u32) {
        self.validator_count = quorum_size;
        self.fault_tolerance = Some(fault_tolerance);
        self.consensus_duration_ms = 0; // Will be set during finalization
    }

    pub fn is_bft_valid(&self) -> bool {
        if let Some(fault_tolerance) = self.fault_tolerance {
            // Check if we have enough validators (3f + 1)
            self.validator_count >= (fault_tolerance * 3) + 1
        } else {
            false
        }
    }
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

lazy_static! {
    static ref TRANSACTION_CACHE: Mutex<HashMap<String, bool>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
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
            fault_tolerance: None,
            height: 0,
            state_root: String::new(),
            transaction_root: String::new(),
            consensus_data: HashMap::new(),
            validator_signatures: Vec::new(),
        };

        let mut block = Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash: String::new(),
            proposer,
            metadata,
            signatures: Vec::new(),
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
        
        hasher.update(self.index.to_string());
        hasher.update(&self.previous_hash);
        hasher.update(self.timestamp.to_string());
        
        for tx in &self.transactions {
            if let Ok(tx_json) = serde_json::to_string(tx) {
                hasher.update(tx_json);
            }
        }
        
        hasher.update(&self.proposer);
        
        format!("{:x}", hasher.finalize())
    }

    /// Adds a validator's signature to the block
    pub async fn add_signature(&mut self, validator_did: String, signature: String, voting_power: f64) -> bool {
        // Check if validator has already signed
        if self.signatures.iter().any(|s| s.validator_did == validator_did) {
            return false;
        }

        let signature_task = task::spawn(async move {
            BlockSignature {
                validator_did,
                signature,
                timestamp: Utc::now(),
                voting_power,
            }
        });

        let new_signature = signature_task.await.unwrap();
        self.signatures.push(new_signature);

        // Update metadata
        self.metadata.validator_count = self.signatures.len() as u32;
        self.metadata.total_voting_power = self.signatures.iter()
            .map(|s| s.voting_power)
            .sum();

        true
    }

    /// Verifies the block's integrity
    pub async fn verify(&self, previous_block: Option<&Block>) -> Result<(), BlockError> {
        // Verify hash
        if self.hash != self.calculate_hash() {
            return Err(BlockError::InvalidTransaction);
        }

        // Verify previous block linkage
        if let Some(prev) = previous_block {
            if self.previous_hash != prev.hash {
                return Err(BlockError::InvalidPreviousHash);
            }
            if self.index != prev.index + 1 {
                return Err(BlockError::InvalidIndex);
            }
            if self.timestamp <= prev.timestamp {
                return Err(BlockError::InvalidTransaction);
            }
        }

        // Verify timestamp is not in the future
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if self.timestamp > current_time + 5000 { // Allow 5 second drift
            return Err(BlockError::InvalidTransaction);
        }

        // Validate transactions
        self.validate_transactions().await?;

        // Verify resource usage
        let calculated_resources: u64 = self.transactions.iter()
            .map(|tx| tx.resource_cost)
            .sum();
        if calculated_resources != self.metadata.resources_used {
            return Err(BlockError::InvalidTransaction);
        }

        // Verify relationship metadata
        let calculated_metadata = Self::calculate_relationship_metadata(&self.transactions);
        if calculated_metadata != self.metadata.relationship_updates {
            return Err(BlockError::InvalidTransaction);
        }

        Ok(())
    }

    /// Validates the transactions in the block
    async fn validate_transactions(&self) -> Result<(), BlockError> {
        for tx in &self.transactions {
            let mut cache = TRANSACTION_CACHE.lock().unwrap();
            
            // Check if transaction is already processed
            if cache.contains_key(&tx.hash) {
                return Err(BlockError::InvalidTransaction);
            }
            
            // Validate the transaction
            if !tx.validate() {
                return Err(BlockError::InvalidTransaction);
            }
            
            // Add to cache
            cache.insert(tx.hash.clone(), true);
        }
        Ok(())
    }

    /// Calculates metadata for relationship transactions in the block
    fn calculate_relationship_metadata(transactions: &[Transaction]) -> Option<RelationshipMetadata> {
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
                TransactionType::RecordContribution { .. } => {
                    metadata.contribution_count += 1;
                    participants.insert(tx.sender.clone());
                }
                TransactionType::RecordMutualAid { receiver, .. } => {
                    metadata.mutual_aid_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(receiver.clone());
                }
                TransactionType::UpdateRelationship { member_two, .. } => {
                    metadata.relationship_update_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(member_two.clone());
                }
                TransactionType::AddEndorsement { to_did, .. } => {
                    metadata.endorsement_count += 1;
                    participants.insert(tx.sender.clone());
                    participants.insert(to_did.clone());
                }
                _ => {}
            }
        }

        metadata.total_participants = participants.len() as u32;

        Some(metadata)
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

    /// Finalizes the block and ensures all validations pass
    pub async fn finalize(&mut self) -> Result<(), BlockError> {
        self.verify(None).await?;
        
        let resource_usage: u64 = self.transactions.iter()
            .map(|tx| tx.resource_cost)
            .sum();
            
        self.metadata.resources_used = resource_usage;
        
        self.metadata.size = bincode::serialize(&self)
            .map_err(|_| BlockError::InvalidTransaction)?
            .len() as u64;
            
        self.hash = self.calculate_hash();

        Ok(())
    }

    /// Initiates a consensus round among the validators
    pub async fn start_consensus_round(&mut self) -> Result<(), BlockError> {
        // Simulate consensus process
        let validator = "validator1";
        let signature = format!("signature_of_{}", validator);
        
        if !self.add_signature(validator.to_string(), signature, 1.0).await {
            return Err(BlockError::InvalidTransaction);
        }

        Ok(())
    }

    /// Records a validator's vote on the block
    pub async fn vote_on_block(&mut self, validator_did: String, vote: bool) -> Result<(), BlockError> {
        if !vote {
            return Err(BlockError::InvalidTransaction);
        }

        let signature = "signature".to_string(); // In real implementation, this would be a proper signature
        
        if !self.add_signature(validator_did, signature, 1.0).await {
            return Err(BlockError::InvalidTransaction);
        }

        Ok(())
    }

    pub fn update_validator_metadata(&mut self, validator: String, reputation: i64) {
        let validator_meta = BlockSignature {
            validator_did: validator,
            signature: String::new(),
            timestamp: Utc::now(),
            voting_power: reputation as f64,
        };
        self.signatures.push(validator_meta);
        
        // Update validator count and total voting power
        self.metadata.validator_count = self.signatures.len() as u32;
        self.metadata.total_voting_power = self.signatures.iter()
            .map(|s| s.voting_power)
            .sum();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    // Resource transfer between members
    Transfer {
        receiver: String,
        amount: u64,
    },
    
    // Smart contract execution
    ContractExecution {
        contract_id: String,
        input_data: std::collections::HashMap<String, i64>,
    },
    
    // Relationship management
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    
    RecordMutualAid {
        receiver: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
    
    UpdateRelationship {
        member_two: String,
        relationship_type: String,
        story: String,
        interaction: Option<String>,
    },
    
    AddEndorsement {
        to_did: String,
        content: String,
        context: String,
        skills: Vec<String>,
    },
}

impl TransactionType {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionType::Transfer { .. } => "Transfer",
            TransactionType::ContractExecution { .. } => "ContractExecution",
            TransactionType::RecordContribution { .. } => "RecordContribution",
            TransactionType::RecordMutualAid { .. } => "RecordMutualAid",
            TransactionType::UpdateRelationship { .. } => "UpdateRelationship",
            TransactionType::AddEndorsement { .. } => "AddEndorsement",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub transaction_type: TransactionType,
    pub timestamp: i64,
    pub hash: String,
    pub signature: Option<String>,
    pub resource_cost: u64,
    pub resource_priority: u8,
    pub zk_snark_proof: Option<String>,
}

impl Transaction {
    pub fn new(sender: String, transaction_type: TransactionType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let receiver = match &transaction_type {
            TransactionType::Transfer { receiver, .. } => receiver.clone(),
            TransactionType::RecordMutualAid { receiver, .. } => receiver.clone(),
            TransactionType::UpdateRelationship { member_two, .. } => member_two.clone(),
            TransactionType::AddEndorsement { to_did, .. } => to_did.clone(),
            _ => String::new(),
        };

        let amount = match &transaction_type {
            TransactionType::Transfer { amount, .. } => *amount,
            _ => 0,
        };

        let hash = Self::calculate_transaction_hash(&sender, &transaction_type, timestamp);
        let resource_cost = Self::calculate_resource_cost(&transaction_type);

        Transaction {
            id: String::new(),
            sender,
            receiver,
            amount,
            transaction_type,
            timestamp: timestamp as i64,
            hash,
            signature: None,
            resource_cost,
            resource_priority: 5, // Default priority
            zk_snark_proof: None,
        }
    }

    pub fn calculate_transaction_hash(sender: &str, transaction_type: &TransactionType, timestamp: u128) -> String {
        let mut hasher = Sha256::new();
        let transaction_data = match transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                format!("Transfer:{}:{}:{}", sender, receiver, amount)
            },
            TransactionType::ContractExecution { contract_id, input_data } => {
                format!("ContractExecution:{}:{:?}", contract_id, input_data)
            },
            TransactionType::RecordContribution { description, impact_story, context, tags } => {
                format!("Contribution:{}:{}:{}:{:?}", description, impact_story, context, tags)
            },
            TransactionType::RecordMutualAid { receiver, description, impact_story, reciprocity_notes, tags } => {
                format!("MutualAid:{}:{}:{:?}:{:?}:{:?}", receiver, description, impact_story, reciprocity_notes, tags)
            },
            TransactionType::UpdateRelationship { member_two, relationship_type, story, interaction } => {
                format!("Relationship:{}:{}:{}:{:?}", member_two, relationship_type, story, interaction)
            },
            TransactionType::AddEndorsement { to_did, content, context, skills } => {
                format!("Endorsement:{}:{}:{}:{:?}", to_did, content, context, skills)
            },
        };
        hasher.update(format!("{}{}{}", sender, transaction_data, timestamp));
        format!("{:x}", hasher.finalize())
    }

    pub fn calculate_resource_cost(transaction_type: &TransactionType) -> u64 {
        match transaction_type {
            TransactionType::Transfer { amount, .. } => {
                100 + (amount / 100)
            },
            TransactionType::ContractExecution { input_data, .. } => {
                200 + (input_data.len() as u64 * 10)
            },
            TransactionType::RecordContribution { description, impact_story, tags, .. } => {
                let content_length = (description.len() + impact_story.len()) as u64;
                50 + (content_length / 100) + (tags.len() as u64 * 5)
            },
            TransactionType::RecordMutualAid { description, tags, .. } => {
                75 + (description.len() as u64 / 100) + (tags.len() as u64 * 5)
            },
            TransactionType::UpdateRelationship { story, .. } => {
                100 + (story.len() as u64 / 100)
            },
            TransactionType::AddEndorsement { content, skills, .. } => {
                   60 + (content.len() as u64 / 100) + (skills.len() as u64 * 10)
            },
        }
    }

    pub fn validate(&self) -> bool {
        if self.sender.is_empty() {
            return false;
        }
        match &self.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                !receiver.is_empty() && *amount > 0 && self.sender != *receiver
            },
            TransactionType::ContractExecution { contract_id, input_data } => {
                !contract_id.is_empty() && !input_data.is_empty()
            },
            TransactionType::RecordContribution { description, impact_story, context, tags } => {
                !description.is_empty() && !impact_story.is_empty() && !context.is_empty() && !tags.is_empty()
            },
            TransactionType::RecordMutualAid { receiver, description, tags, .. } => {
                !receiver.is_empty() && !description.is_empty() && !tags.is_empty()
            },
            TransactionType::UpdateRelationship { member_two, relationship_type, story, .. } => {
                !member_two.is_empty() && !relationship_type.is_empty() && !story.is_empty()
            },
            TransactionType::AddEndorsement { to_did, content, context, skills } => {
                !to_did.is_empty() && !content.is_empty() && !context.is_empty() && !skills.is_empty()
            },
        }
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.resource_priority = priority.min(10);
    }

    pub fn get_timestamp_ms(&self) -> u128 {
        self.timestamp as u128
    }

    pub fn get_sender(&self) -> &str {
        &self.sender
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_else(|_| Vec::new())
    }
    
    pub fn get_zk_snark_inputs(&self) -> Vec<u8> {
        // A simple implementation that concatenates transaction data
        let mut inputs = Vec::new();
        inputs.extend_from_slice(self.sender.as_bytes());
        inputs.extend_from_slice(self.receiver.as_bytes());
        inputs.extend_from_slice(&self.amount.to_le_bytes());
        inputs.extend_from_slice(self.hash.as_bytes());
        inputs
    }
}

#[derive(Debug, Clone)]
pub struct AuthError(String);

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidRegistryTransaction {
    pub registry_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttestMembershipTransaction {
    pub member_did: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutualCreditTransaction {
    pub sender_did: String,
    pub receiver_did: String,
    pub amount: i64, // positive value; sender's balance decreases and receiver's increases
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceTransaction {
    pub proposal_id: String,
    pub initiator_did: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceTransaction {
    pub resource_id: String,
    pub operation: String,
    pub metadata: String,
    pub did_proof: DidProof,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidProof {
    pub did: String,
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionTransaction {
    pub candidate_id: String,
    pub votes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteProposal {
    pub proposal_id: String,
    pub initiator_did: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub voter_did: String,
    pub proposal_id: String,
    pub approve: bool,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidRegistry;

impl DidRegistry {
    pub fn new() -> Self {
        DidRegistry
    }

    pub fn get_did(&self, _did: &str) -> Option<DidDocument> {
        // Placeholder implementation
        Some(DidDocument { is_verified: true })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidDocument {
    pub is_verified: bool,
}

impl Transaction {
    pub fn verify_did_signature(_did: &str, _signature: &str, _message: &str) -> Result<bool, AuthError> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationType {
    Cooperative,
    Community,
    Hybrid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTerms {
    pub minimum_reputation: i64,
    pub resource_sharing_policies: String,
    pub governance_rules: String,
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationOperation {
    InitiateFederation {
        federation_type: FederationType,
        partner_id: String,
        terms: FederationTerms,
    },
    JoinFederation {
        federation_id: String,
        commitment: Vec<String>,
    },
    LeaveFederation {
        federation_id: String,
        reason: String,
    },
    ProposeAction {
        federation_id: String,
        action_type: String,
        description: String,
        resources: HashMap<String, u64>,
    },
    VoteOnProposal {
        federation_id: String,
        proposal_id: String,
        approve: bool,
        notes: Option<String>,
    },
    ShareResources {
        federation_id: String,
        resource_type: String,
        amount: u64,
        recipient_id: String,
    },
    UpdateFederationTerms {
        federation_id: String,
        new_terms: FederationTerms,
    },
}

/// Represents a unique identifier for a federation
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FederationId(pub String);

/// Represents a unique identifier for a cooperative
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CooperativeId(pub String);

/// Represents a member's identity within the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MemberId {
    pub did: String,
    pub cooperative_id: CooperativeId,
}

/// Represents the status of a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationStatus {
    Active,
    Suspended,
    Inactive,
}

/// Represents a governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: MemberId,
    pub created_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub votes: HashMap<MemberId, VoteChoice>,
}

/// Represents the status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

/// Represents a member's reputation score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    pub member_id: MemberId,
    pub score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Represents a resource in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub owner: CooperativeId,
    pub resource_type: String,
    pub metadata: HashMap<String, String>,
    pub availability: ResourceAvailability,
}

/// Represents the availability status of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAvailability {
    Available,
    InUse,
    Maintenance,
    Offline,
}

/// Storage-related error types
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("IPFS error: {0}")]
    IpfsError(String),
    
    #[error("Storage reference already exists")]
    ReferenceAlreadyExists,
    
    #[error("Storage reference not found")]
    ReferenceNotFound,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageReference {
    pub id: String,
    pub storage_type: StorageType,
    pub location: String,
    pub status: StorageStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    Distributed,
    Cloud,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageStatus {
    Available,
    Unavailable,
    Syncing,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a value with the given key
    async fn store(&self, data: &[u8]) -> Result<String, StorageError>;
    
    /// Retrieve a value by key
    async fn retrieve(&self, id: &str) -> Result<Vec<u8>, StorageError>;
    
    /// Delete a value by key
    async fn delete(&self, key: &str) -> StorageResult<()>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend_type: String,
    pub cache_size: usize,
    pub cache_ttl_seconds: u64,
    pub ipfs_url: String,
    pub database_url: Option<String>,
}

/// Runtime-related error types
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Invalid state")]
    InvalidState,
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("DSL error: {0}")]
    DslError(String),
    
    #[error("Contract error: {0}")]
    ContractError(String),
}

impl From<RuntimeError> for IcnError {
    fn from(err: RuntimeError) -> Self {
        IcnError::RuntimeError(err.to_string())
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Represents the execution context for runtime operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub transaction: Option<Transaction>,
    pub block: Option<Block>,
    pub state: HashMap<String, Vec<u8>>,
    pub metadata: HashMap<String, String>,
}

/// Represents a validation check in the runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub condition: String,
    pub action: String,
}

/// Represents state validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValidation {
    pub current: Option<String>,
    pub expected: Option<String>,
    pub transitions: Vec<String>,
}

/// Represents a validation node in the DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationNode {
    pub pre_checks: Vec<Check>,
    pub post_checks: Vec<Check>,
    pub state_validation: Option<StateValidation>,
}

/// Represents a governance node in the DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNode {
    pub rules: Vec<String>,
    pub voting_config: HashMap<String, String>,
    pub permissions: HashMap<String, Vec<String>>,
}

/// Represents a marketplace node in the DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceNode {
    pub rules: Vec<String>,
    pub pricing_model: String,
    pub constraints: Vec<String>,
}

/// Runtime configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub vm_type: String,
    pub max_execution_time: u64,
    pub max_memory: u64,
    pub enable_debugging: bool,
    pub log_level: String,
}

#[derive(Debug, Clone)]
pub struct ReputationError {
    pub message: String,
}

impl std::fmt::Display for ReputationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ReputationError {}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub message: String,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExecutionError {}

#[derive(Debug, Clone)]
pub struct ContractInput {
    pub contract_id: String,
    pub method: String,
    pub args: Vec<u8>,
}

#[async_trait::async_trait]
pub trait RuntimeInterface: Send + Sync {
    async fn execute_transaction(&self, transaction: &Transaction) -> Result<(), RuntimeError>;
    async fn execute_block(&self, block: &Block) -> Result<(), RuntimeError>;
    async fn execute_contract(&self, input: ContractInput) -> Result<Vec<u8>, ExecutionError>;
    async fn get_contract_state(&self, contract_id: &str) -> Result<Vec<u8>, ExecutionError>;
}
