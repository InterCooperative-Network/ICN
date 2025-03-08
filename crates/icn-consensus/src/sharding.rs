use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;
use icn_types::{Transaction, Block, ShardId, NodeId};

/// Config for the shard manager
#[derive(Clone, Debug)]
pub struct ShardConfig {
    pub shard_count: u32,
    pub shard_capacity: u32,
    pub rebalance_threshold: f64,
}

/// Error types for sharding module
#[derive(Error, Debug)]
pub enum ShardingError {
    #[error("Invalid shard configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Shard not found: {0}")]
    ShardNotFound(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Node not in shard: {0}")]
    NodeNotInShard(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
}

// Simplified ShardManager for direct integration with ProofOfCooperation
pub struct ShardManager {
    pub config: ShardConfig,
    pub shards: HashMap<u32, Vec<Transaction>>,
    current_shard_id: u32,
}

impl ShardManager {
    pub fn new(config: ShardConfig) -> Self {
        // Initialize empty shards
        let mut shards = HashMap::new();
        for i in 0..config.shard_count {
            shards.insert(i, Vec::new());
        }
        
        Self {
            config,
            shards,
            current_shard_id: 0,
        }
    }
    
    // Assign a transaction to a shard
    pub fn assign_transaction(&mut self, transaction: Transaction) -> u32 {
        // Simple round-robin assignment
        let shard_id = self.current_shard_id;
        
        // Add to shard
        self.shards.entry(shard_id).or_insert_with(Vec::new).push(transaction);
        
        // Move to next shard
        self.current_shard_id = (self.current_shard_id + 1) % self.config.shard_count;
        
        shard_id
    }
    
    // Finalize a shard by creating a block from its transactions
    pub fn finalize_shard(&mut self, shard_id: u32) -> Option<Block> {
        if let Some(transactions) = self.shards.get(&shard_id) {
            if transactions.is_empty() {
                return None;
            }
            
            // Create a block from transactions
            let mut tx_ids = Vec::new();
            let mut tx_hashes = Vec::new();
            
            for tx in transactions {
                tx_ids.push(tx.id.clone());
                tx_hashes.push(tx.hash.clone());
            }
            
            // Create simple hash from transaction hashes
            let hash = format!("{:x}", md5::compute(tx_hashes.join("")));
            
            let block = Block {
                height: 0, // This would be set by blockchain
                hash,
                transactions: tx_ids,
                timestamp: chrono::Utc::now(),
                previous_hash: "".to_string(), // This would be set by blockchain
                shard_id: Some(shard_id),
                metadata: BlockMetadata {
                    consensus_duration_ms: 0, // Will be set later
                    validator_signatures: Vec::new(), // Will be set later
                    reputation_changes: Vec::new(), // Will be set later
                },
            };
            
            // Clear the shard's transactions
            self.shards.insert(shard_id, Vec::new());
            
            Some(block)
        } else {
            None
        }
    }
    
    // Balance load across shards
    pub fn rebalance(&mut self) {
        // Find average load
        let total_transactions: usize = self.shards.values().map(|v| v.len()).sum();
        let avg_load = total_transactions as f64 / self.config.shard_count as f64;
        
        // Identify overloaded and underloaded shards
        let mut overloaded = Vec::new();
        let mut underloaded = Vec::new();
        
        for (id, shard) in &self.shards {
            let load = shard.len() as f64;
            let diff = (load - avg_load) / avg_load;
            
            if diff > self.config.rebalance_threshold {
                overloaded.push(*id);
            } else if diff < -self.config.rebalance_threshold {
                underloaded.push(*id);
            }
        }
        
        // Balance if needed
        if !overloaded.is_empty() && !underloaded.is_empty() {
            let mut i = 0;
            let mut j = 0;
            
            while i < overloaded.len() && j < underloaded.len() {
                let over_id = overloaded[i];
                let under_id = underloaded[j];
                
                if let (Some(over_shard), Some(under_shard)) = 
                    (self.shards.get_mut(&over_id), self.shards.get_mut(&under_id)) {
                    
                    let over_len = over_shard.len();
                    let under_len = under_shard.len();
                    
                    // Calculate how many transactions to move
                    let target_len = ((over_len + under_len) as f64 / 2.0).round() as usize;
                    let to_move = over_len - target_len;
                    
                    // Move transactions
                    if to_move > 0 {
                        let mut transactions_to_move = Vec::new();
                        for _ in 0..to_move {
                            if let Some(tx) = over_shard.pop() {
                                transactions_to_move.push(tx);
                            }
                        }
                        under_shard.extend(transactions_to_move);
                    }
                }
                
                i += 1;
                j += 1;
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub consensus_duration_ms: u64,
    pub validator_signatures: Vec<String>,
    pub reputation_changes: Vec<ReputationChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationChange {
    pub did: String,
    pub change: i64,
    pub reason: String,
}

// Implement these types if they're not defined elsewhere
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub hash: String,
    pub transactions: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub previous_hash: String,
    pub shard_id: Option<u32>,
    pub metadata: BlockMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub hash: String,
    pub transaction_type: String,
    pub data: String,
}
