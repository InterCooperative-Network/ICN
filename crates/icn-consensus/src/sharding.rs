use std::collections::{HashMap, HashSet};
use rand::Rng;
use serde::{Serialize, Deserialize};
use md5::Md5;
use digest::{Digest, Update};
use icn_types::{Block, Transaction, ShardId, NodeId};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ShardConfig {
    pub shard_count: u32,
    pub shard_capacity: u32,
    pub rebalance_threshold: f32,
}

/// Error types for sharding module
#[derive(Error, Debug)]
pub enum ShardingError {
    #[error("Invalid shard configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Shard not found: {0}")]
    ShardNotFound(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Node not in shard: {0}")]
    NodeNotInShard(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
}

pub struct ShardManager {
    pub config: ShardConfig,
    pub shards: HashMap<u32, Vec<Transaction>>,
    transaction_assignments: HashMap<String, u32>, // tx_hash -> shard_id
    load_metrics: HashMap<u32, f32>,
}

impl ShardManager {
    pub fn new(config: ShardConfig) -> Self {
        let mut shards = HashMap::new();
        for i in 0..config.shard_count {
            shards.insert(i, Vec::new());
        }
        
        Self {
            config,
            shards,
            transaction_assignments: HashMap::new(),
            load_metrics: HashMap::new(),
        }
    }
    
    pub fn assign_transaction(&mut self, transaction: Transaction) -> u32 {
        // Use transaction hash to determine shard
        let tx_hash = self.hash_transaction(&transaction);
        let shard_id = self.get_shard_for_hash(&tx_hash);
        
        // Store transaction in shard
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            shard.push(transaction);
            self.transaction_assignments.insert(tx_hash, shard_id);
            self.update_load_metrics(shard_id);
        }
        
        shard_id
    }
    
    pub fn finalize_shard(&mut self, shard_id: u32) -> Option<Block> {
        if let Some(transactions) = self.shards.get_mut(&shard_id) {
            if transactions.is_empty() {
                return None;
            }
            
            // Create Merkle tree of transaction hashes
            let tx_hashes: Vec<String> = transactions.iter()
                .map(|tx| self.hash_transaction(tx))
                .collect();
                
            let merkle_root = self.create_merkle_root(&tx_hashes);
            
            // Create new block
            let block = Block {
                shard_id,
                transactions: transactions.drain(..).collect(),
                merkle_root,
                timestamp: chrono::Utc::now().timestamp(),
                metadata: BlockMetadata {
                    consensus_duration_ms: 0,
                    shard_size: tx_hashes.len() as u32,
                },   
            };
            
            // Clear transaction assignments for this shard
            self.transaction_assignments.retain(|_, &mut sid| sid != shard_id);
            self.update_load_metrics(shard_id);
            
            Some(block)
        } else {
            None
        }
    }
    
    pub fn rebalance(&mut self) {
        // Calculate load factors
        let mut loads = Vec::new();
        for shard_id in 0..self.config.shard_count {
            if let Some(load) = self.load_metrics.get(&shard_id) {
                loads.push((*load, shard_id));
            }
        }
        
        // Sort by load
        loads.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Check if rebalancing is needed
        if loads.len() >= 2 {
            let (max_load, overloaded_shard) = loads[0];
            let (min_load, underloaded_shard) = loads[loads.len() - 1];
            
            if (max_load - min_load) > self.config.rebalance_threshold {
                // Move transactions from overloaded to underloaded shard
                if let Some(source_shard) = self.shards.get_mut(&overloaded_shard) {
                    if let Some(target_shard) = self.shards.get_mut(&underloaded_shard) {
                        let transfer_count = (source_shard.len() - target_shard.len()) / 2;
                        let mut transferred = 0;
                        
                        while transferred < transfer_count && !source_shard.is_empty() {
                            if let Some(tx) = source_shard.pop() {
                                let tx_hash = self.hash_transaction(&tx);
                                target_shard.push(tx);
                                self.transaction_assignments.insert(tx_hash, underloaded_shard);
                                transferred += 1;
                            }
                        }
                        
                        // Update load metrics
                        self.update_load_metrics(overloaded_shard);
                        self.update_load_metrics(underloaded_shard);
                    }
                }
            }
        }
    }
    
    fn hash_transaction(&self, transaction: &Transaction) -> String {
        let mut hasher = Md5::new();
        hasher.update(format!("{:?}", transaction).as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn get_shard_for_hash(&self, hash: &str) -> u32 {
        // Use first 4 bytes of hash as u32
        let bytes = hex::decode(&hash[0..8]).unwrap();
        let shard_hash = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        shard_hash % self.config.shard_count
    }
    
    fn create_merkle_root(&self, tx_hashes: &[String]) -> String {
        if tx_hashes.is_empty() {
            return String::from("0");
        }
        
        let mut current_level = tx_hashes.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for pair in current_level.chunks(2) {
                let combined = if pair.len() == 2 {
                    format!("{}{}", pair[0], pair[1])
                } else {
                    pair[0].clone()
                };
                
                let mut hasher = Md5::new();
                hasher.update(combined.as_bytes());
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            
            current_level = next_level;
        }
        
        current_level[0].clone()
    }
    
    fn update_load_metrics(&mut self, shard_id: u32) {
        if let Some(shard) = self.shards.get(&shard_id) {
            let load_factor = shard.len() as f32 / self.config.shard_capacity as f32;
            self.load_metrics.insert(shard_id, load_factor);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub consensus_duration_ms: u64,
    pub shard_size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationChange {
    pub did: String,
    pub change: i64,
    pub reason: String,
}

pub struct Shard {
    pub id: String,
    pub nodes: Vec<String>,
    pub transactions: HashMap<String, Transaction>,
}

impl Shard {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: String) {
        self.nodes.push(node_id);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.insert(transaction.id.clone(), transaction);
    }
}
