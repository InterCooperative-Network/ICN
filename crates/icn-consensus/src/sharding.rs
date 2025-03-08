use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;
use icn_types::{Transaction, Block, ShardId, NodeId};

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

/// Shard assignment method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardAssignmentMethod {
    /// Random distribution of transactions to shards
    Random,
    
    /// Distribution based on transaction content (e.g., by account prefix)
    ContentBased,
    
    /// Distribution based on geographic location
    GeographicLocation,
    
    /// Distribution based on transaction type
    TransactionType,
    
    /// Custom assignment method
    Custom(String),
}

/// Configuration for the sharding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardingConfig {
    /// Number of shards to create
    pub num_shards: usize,
    
    /// Minimum number of nodes per shard
    pub min_nodes_per_shard: usize,
    
    /// Maximum number of nodes per shard
    pub max_nodes_per_shard: usize,
    
    /// Method used to assign transactions to shards
    pub assignment_method: ShardAssignmentMethod,
    
    /// Whether to enable cross-shard transactions
    pub enable_cross_shard: bool,
    
    /// Maximum transactions per shard block
    pub max_tx_per_block: usize,
    
    /// Reshard frequency (in blocks)
    pub reshard_frequency: Option<u64>,
}

/// Information about a shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    /// Unique ID of the shard
    pub id: ShardId,
    
    /// Nodes participating in this shard
    pub nodes: Vec<NodeId>,
    
    /// Current leader node for this shard
    pub leader: Option<NodeId>,
    
    /// Last block height for this shard
    pub last_block_height: u64,
    
    /// Number of pending transactions in this shard
    pub pending_transactions: usize,
    
    /// Hash of the last block in this shard
    pub last_block_hash: Option<String>,
    
    /// When this shard was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Whether this shard is active
    pub active: bool,
}

/// Transaction with shard assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardedTransaction {
    /// The transaction itself
    pub transaction: Transaction,
    
    /// ID of the shard this transaction is assigned to
    pub shard_id: ShardId,
    
    /// Whether this is a cross-shard transaction
    pub is_cross_shard: bool,
    
    /// Related shard IDs (for cross-shard transactions)
    pub related_shards: Vec<ShardId>,
    
    /// Assignment timestamp
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

/// Status of a transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction has been received but not yet assigned to a shard
    Received,
    
    /// Transaction has been assigned to a shard but not yet processed
    Assigned,
    
    /// Transaction is being processed
    Processing,
    
    /// Transaction has been processed and included in a block
    Included,
    
    /// Transaction has been finalized and confirmed
    Confirmed,
    
    /// Transaction failed to process
    Failed,
}

/// Results of a resharding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReshardingResults {
    /// Number of shards before resharding
    pub previous_shard_count: usize,
    
    /// Number of shards after resharding
    pub new_shard_count: usize,
    
    /// Number of nodes reassigned
    pub nodes_reassigned: usize,
    
    /// Number of transactions reassigned
    pub transactions_reassigned: usize,
    
    /// Time taken for resharding (in milliseconds)
    pub time_taken_ms: u64,
}

/// Manages transaction sharding for scalable processing
pub struct ShardingManager {
    /// Configuration for the sharding system
    config: RwLock<ShardingConfig>,
    
    /// Map of shards
    shards: RwLock<HashMap<ShardId, ShardInfo>>,
    
    /// Map of transactions to shards
    transaction_assignments: RwLock<HashMap<String, ShardedTransaction>>,
    
    /// Status of each transaction
    transaction_status: RwLock<HashMap<String, TransactionStatus>>,
    
    /// Nodes participating in sharding
    nodes: RwLock<Vec<NodeId>>,
    
    /// Queue of unassigned transactions
    unassigned_transactions: Mutex<VecDeque<Transaction>>,
    
    /// Cross-shard transaction coordination
    cross_shard_coordinator: RwLock<CrossShardCoordinator>,
}

/// Coordinates cross-shard transactions
pub struct CrossShardCoordinator {
    /// Map of transaction ID to involved shards
    transaction_shards: HashMap<String, HashSet<ShardId>>,
    
    /// Map of transaction ID to shard commitments
    commitments: HashMap<String, HashMap<ShardId, bool>>,
}

impl ShardingManager {
    /// Create a new ShardingManager with the given configuration
    pub fn new(config: ShardingConfig) -> Self {
        Self {
            config: RwLock::new(config),
            shards: RwLock::new(HashMap::new()),
            transaction_assignments: RwLock::new(HashMap::new()),
            transaction_status: RwLock::new(HashMap::new()),
            nodes: RwLock::new(Vec::new()),
            unassigned_transactions: Mutex::new(VecDeque::new()),
            cross_shard_coordinator: RwLock::new(CrossShardCoordinator {
                transaction_shards: HashMap::new(),
                commitments: HashMap::new(),
            }),
        }
    }

    /// Initialize shards based on the configuration and available nodes
    pub async fn initialize_shards(&self) -> Result<Vec<ShardId>, ShardingError> {
        let config = self.config.read().await;
        let nodes = self.nodes.read().await;
        
        // Ensure we have enough nodes for the requested number of shards
        if nodes.len() < config.num_shards * config.min_nodes_per_shard {
            return Err(ShardingError::InvalidConfiguration(format!(
                "Not enough nodes ({}) for {} shards with minimum {} nodes each",
                nodes.len(),
                config.num_shards,
                config.min_nodes_per_shard
            )));
        }
        
        // Create shards
        let mut shards = self.shards.write().await;
        shards.clear();
        
        let mut shard_ids = Vec::with_capacity(config.num_shards);
        let nodes_per_shard = nodes.len() / config.num_shards;
        
        for i in 0..config.num_shards {
            let shard_id = ShardId::new();
            let shard_nodes: Vec<NodeId> = nodes[(i * nodes_per_shard)..((i + 1) * nodes_per_shard)]
                .to_vec();
            
            // Create ShardInfo
            let shard_info = ShardInfo {
                id: shard_id.clone(),
                nodes: shard_nodes,
                leader: None,
                last_block_height: 0,
                pending_transactions: 0,
                last_block_hash: None,
                created_at: chrono::Utc::now(),
                active: true,
            };
            
            shards.insert(shard_id.clone(), shard_info);
            shard_ids.push(shard_id);
        }
        
        Ok(shard_ids)
    }

    /// Register a node with the sharding system
    pub async fn register_node(&self, node_id: NodeId) {
        let mut nodes = self.nodes.write().await;
        if !nodes.contains(&node_id) {
            nodes.push(node_id);
        }
    }

    /// Assign a transaction to a shard
    pub async fn assign_transaction(&self, transaction: Transaction) -> Result<ShardId, ShardingError> {
        let config = self.config.read().await;
        let shards = self.shards.read().await;
        
        // Check if we have any shards
        if shards.is_empty() {
            // Queue the transaction for later assignment
            let mut unassigned = self.unassigned_transactions.lock().await;
            unassigned.push_back(transaction.clone());
            
            // Set status to Received
            let mut status = self.transaction_status.write().await;
            status.insert(transaction.id.clone(), TransactionStatus::Received);
            
            return Err(ShardingError::InvalidConfiguration("No shards available".to_string()));
        }
        
        // Choose a shard based on the assignment method
        let shard_id = match config.assignment_method {
            ShardAssignmentMethod::Random => {
                // Simple random assignment
                let shard_ids: Vec<_> = shards.keys().cloned().collect();
                let random_index = rand::random::<usize>() % shard_ids.len();
                shard_ids[random_index].clone()
            },
            ShardAssignmentMethod::ContentBased => {
                // Use transaction content hash to determine shard
                let hash = format!("{:x}", md5::compute(transaction.data.as_bytes()));
                let hash_num = u64::from_str_radix(&hash[0..16], 16).unwrap_or(0);
                let shard_ids: Vec<_> = shards.keys().cloned().collect();
                let index = (hash_num % shard_ids.len() as u64) as usize;
                shard_ids[index].clone()
            },
            ShardAssignmentMethod::TransactionType => {
                // Use transaction type to determine shard
                let tx_type = transaction.transaction_type.clone();
                let mut hash = 0;
                for c in tx_type.chars() {
                    hash = ((hash << 5) - hash) + c as u64;
                    hash &= hash;
                }
                let shard_ids: Vec<_> = shards.keys().cloned().collect();
                let index = (hash % shard_ids.len() as u64) as usize;
                shard_ids[index].clone()
            },
            _ => {
                // Default to random for other methods
                let shard_ids: Vec<_> = shards.keys().cloned().collect();
                let random_index = rand::random::<usize>() % shard_ids.len();
                shard_ids[random_index].clone()
            },
        };
        
        // Check if this is a cross-shard transaction
        let is_cross_shard = false; // For simplicity, we assume false here
        let related_shards = Vec::new(); // No related shards for now
        
        // Create sharded transaction
        let sharded_tx = ShardedTransaction {
            transaction: transaction.clone(),
            shard_id: shard_id.clone(),
            is_cross_shard,
            related_shards,
            assigned_at: chrono::Utc::now(),
        };
        
        // Store the assignment
        let mut assignments = self.transaction_assignments.write().await;
        assignments.insert(transaction.id.clone(), sharded_tx);
        
        // Update transaction status
        let mut status = self.transaction_status.write().await;
        status.insert(transaction.id.clone(), TransactionStatus::Assigned);
        
        // Update shard info
        let mut shards = self.shards.write().await;
        if let Some(shard) = shards.get_mut(&shard_id) {
            shard.pending_transactions += 1;
        }
        
        Ok(shard_id)
    }

    /// Get all transactions assigned to a specific shard
    pub async fn get_shard_transactions(&self, shard_id: &ShardId) -> Result<Vec<Transaction>, ShardingError> {
        let shards = self.shards.read().await;
        if !shards.contains_key(shard_id) {
            return Err(ShardingError::ShardNotFound(shard_id.to_string()));
        }
        
        let assignments = self.transaction_assignments.read().await;
        let transactions: Vec<Transaction> = assignments.values()
            .filter(|tx| &tx.shard_id == shard_id)
            .map(|tx| tx.transaction.clone())
            .collect();
        
        Ok(transactions)
    }

    /// Process pending transactions for a given shard
    pub async fn process_shard_transactions(&self, shard_id: &ShardId, node_id: &NodeId) 
        -> Result<Vec<Transaction>, ShardingError> 
    {
        // Check if node is in the shard
        let shards = self.shards.read().await;
        let shard = shards.get(shard_id)
            .ok_or_else(|| ShardingError::ShardNotFound(shard_id.to_string()))?;
            
        if !shard.nodes.contains(node_id) {
            return Err(ShardingError::NodeNotInShard(node_id.to_string()));
        }
        
        // Get transactions for the shard
        let config = self.config.read().await;
        let assignments = self.transaction_assignments.read().await;
        
        let mut transactions: Vec<Transaction> = assignments.values()
            .filter(|tx| &tx.shard_id == shard_id)
            .map(|tx| tx.transaction.clone())
            .collect();
            
        // Limit number of transactions to process
        if transactions.len() > config.max_tx_per_block {
            transactions.truncate(config.max_tx_per_block);
        }
        
        // Update status for all these transactions
        let mut status = self.transaction_status.write().await;
        for tx in &transactions {
            status.insert(tx.id.clone(), TransactionStatus::Processing);
        }
        
        Ok(transactions)
    }

    /// Mark transactions as included in a block
    pub async fn mark_transactions_included(&self, transaction_ids: &[String], block: &Block) 
        -> Result<(), ShardingError> 
    {
        // Update transaction status
        let mut status = self.transaction_status.write().await;
        for tx_id in transaction_ids {
            status.insert(tx_id.clone(), TransactionStatus::Included);
        }
        
        // Get shard ID for updating block info
        let assignments = self.transaction_assignments.read().await;
        if let Some(first_tx) = transaction_ids.first().and_then(|id| assignments.get(id)) {
            let shard_id = &first_tx.shard_id;
            
            // Update shard info
            let mut shards = self.shards.write().await;
            if let Some(shard) = shards.get_mut(shard_id) {
                shard.last_block_height = block.height;
                shard.last_block_hash = Some(block.hash.clone());
                shard.pending_transactions -= transaction_ids.len().min(shard.pending_transactions);
            }
        }
        
        Ok(())
    }

    /// Mark transactions as confirmed
    pub async fn mark_transactions_confirmed(&self, transaction_ids: &[String]) -> Result<(), ShardingError> {
        // Update transaction status
        let mut status = self.transaction_status.write().await;
        for tx_id in transaction_ids {
            status.insert(tx_id.clone(), TransactionStatus::Confirmed);
        }
        
        Ok(())
    }

    /// Get the status of a transaction
    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, ShardingError> {
        let status = self.transaction_status.read().await;
        status.get(transaction_id)
            .cloned()
            .ok_or_else(|| ShardingError::TransactionNotFound(transaction_id.to_string()))
    }

    /// Get information about a shard
    pub async fn get_shard_info(&self, shard_id: &ShardId) -> Result<ShardInfo, ShardingError> {
        let shards = self.shards.read().await;
        shards.get(shard_id)
            .cloned()
            .ok_or_else(|| ShardingError::ShardNotFound(shard_id.to_string()))
    }

    /// Get all active shards
    pub async fn get_active_shards(&self) -> Vec<ShardInfo> {
        let shards = self.shards.read().await;
        shards.values()
            .filter(|s| s.active)
            .cloned()
            .collect()
    }

    /// Set the leader for a shard
    pub async fn set_shard_leader(&self, shard_id: &ShardId, node_id: &NodeId) -> Result<(), ShardingError> {
        let mut shards = self.shards.write().await;
        let shard = shards.get_mut(shard_id)
            .ok_or_else(|| ShardingError::ShardNotFound(shard_id.to_string()))?;
            
        if !shard.nodes.contains(node_id) {
            return Err(ShardingError::NodeNotInShard(node_id.to_string()));
        }
        
        shard.leader = Some(node_id.clone());
        
        Ok(())
    }

    /// Reshard the system (redistribute nodes across shards)
    pub async fn reshard(&self) -> Result<ReshardingResults, ShardingError> {
        let start_time = std::time::Instant::now();
        let config = self.config.read().await;
        
        // Record previous state for result stats
        let previous_shards = {
            let shards = self.shards.read().await;
            shards.len()
        };
        
        // Get current nodes
        let nodes = self.nodes.read().await;
        
        // Calculate number of nodes per shard
        let nodes_per_shard = nodes.len() / config.num_shards;
        if nodes_per_shard < config.min_nodes_per_shard {
            return Err(ShardingError::InvalidConfiguration(format!(
                "Not enough nodes for resharding: {} nodes for {} shards (minimum {} per shard)",
                nodes.len(),
                config.num_shards,
                config.min_nodes_per_shard
            )));
        }
        
        // Create new shards
        let mut new_shards = HashMap::new();
        for i in 0..config.num_shards {
            let shard_id = ShardId::new();
            let shard_nodes: Vec<NodeId> = nodes[(i * nodes_per_shard)..((i + 1) * nodes_per_shard)]
                .to_vec();
            
            // Create ShardInfo
            let shard_info = ShardInfo {
                id: shard_id.clone(),
                nodes: shard_nodes,
                leader: None,
                last_block_height: 0,
                pending_transactions: 0,
                last_block_hash: None,
                created_at: chrono::Utc::now(),
                active: true,
            };
            
            new_shards.insert(shard_id, shard_info);
        }
        
        // Count how many nodes were reassigned
        let mut nodes_reassigned = 0;
        {
            let old_shards = self.shards.read().await;
            for old_shard in old_shards.values() {
                for node_id in &old_shard.nodes {
                    let mut found_in_same_shard = false;
                    for new_shard in new_shards.values() {
                        if new_shard.nodes.contains(node_id) {
                            if old_shard.id == new_shard.id {
                                found_in_same_shard = true;
                            }
                            break;
                        }
                    }
                    if !found_in_same_shard {
                        nodes_reassigned += 1;
                    }
                }
            }
        }
        
        // Reassign transactions to new shards
        let old_assignments = {
            let assignments = self.transaction_assignments.read().await;
            assignments.clone()
        };
        
        let mut new_assignments = HashMap::with_capacity(old_assignments.len());
        let mut tx_reassigned = 0;
        
        for (tx_id, old_assignment) in old_assignments {
            let tx = old_assignment.transaction.clone();
            
            // Choose a new shard for this transaction
            let new_shard_id = match config.assignment_method {
                ShardAssignmentMethod::Random => {
                    let shard_ids: Vec<_> = new_shards.keys().cloned().collect();
                    let random_index = rand::random::<usize>() % shard_ids.len();
                    shard_ids[random_index].clone()
                },
                ShardAssignmentMethod::ContentBased => {
                    let hash = format!("{:x}", md5::compute(tx.data.as_bytes()));
                    let hash_num = u64::from_str_radix(&hash[0..16], 16).unwrap_or(0);
                    let shard_ids: Vec<_> = new_shards.keys().cloned().collect();
                    let index = (hash_num % shard_ids.len() as u64) as usize;
                    shard_ids[index].clone()
                },
                ShardAssignmentMethod::TransactionType => {
                    let tx_type = tx.transaction_type.clone();
                    let mut hash = 0;
                    for c in tx_type.chars() {
                        hash = ((hash << 5) - hash) + c as u64;
                        hash &= hash;
                    }
                    let shard_ids: Vec<_> = new_shards.keys().cloned().collect();
                    let index = (hash % shard_ids.len() as u64) as usize;
                    shard_ids[index].clone()
                },
                _ => {
                    let shard_ids: Vec<_> = new_shards.keys().cloned().collect();
                    let random_index = rand::random::<usize>() % shard_ids.len();
                    shard_ids[random_index].clone()
                },
            };
            
            // Check if the transaction got reassigned
            if new_shard_id != old_assignment.shard_id {
                tx_reassigned += 1;
            }
            
            // Update transaction assignment
            let is_cross_shard = old_assignment.is_cross_shard;
            let related_shards = old_assignment.related_shards.clone();
            
            let new_assignment = ShardedTransaction {
                transaction: tx,
                shard_id: new_shard_id.clone(),
                is_cross_shard,
                related_shards,
                assigned_at: chrono::Utc::now(),
            };
            
            // Increment pending transaction count for the shard
            if let Some(shard) = new_shards.get_mut(&new_shard_id) {
                shard.pending_transactions += 1;
            }
            
            new_assignments.insert(tx_id, new_assignment);
        }
        
        // Update shards and assignments
        {
            let mut shards = self.shards.write().await;
            *shards = new_shards;
        }
        
        {
            let mut assignments = self.transaction_assignments.write().await;
            *assignments = new_assignments;
        }
        
        // Calculate elapsed time
        let elapsed = start_time.elapsed().as_millis() as u64;
        
        // Create and return results
        let results = ReshardingResults {
            previous_shard_count: previous_shards,
            new_shard_count: config.num_shards,
            nodes_reassigned,
            transactions_reassigned: tx_reassigned,
            time_taken_ms: elapsed,
        };
        
        Ok(results)
    }

    /// Check if it's time to reshard
    pub async fn check_reshard_needed(&self) -> bool {
        let config = self.config.read().await;
        
        // If resharding is disabled, return false
        if config.reshard_frequency.is_none() {
            return false;
        }
        
        let reshard_frequency = config.reshard_frequency.unwrap();
        
        // Check if any shard has reached the reshard frequency
        let shards = self.shards.read().await;
        for shard in shards.values() {
            if shard.last_block_height % reshard_frequency == 0 && shard.last_block_height > 0 {
                return true;
            }
        }
        
        false
    }

    /// Process unassigned transactions
    pub async fn process_unassigned_transactions(&self) -> Result<usize, ShardingError> {
        let mut unassigned = self.unassigned_transactions.lock().await;
        let count = unassigned.len();
        
        if count == 0 {
            return Ok(0);
        }
        
        let mut processed = 0;
        while let Some(tx) = unassigned.pop_front() {
            if let Err(e) = self.assign_transaction(tx).await {
                // If assignment fails, push back to queue
                unassigned.push_back(tx);
                break;
            }
            processed += 1;
        }
        
        Ok(processed)
    }
}

/// CrossShardCoordinator implementation
impl CrossShardCoordinator {
    /// Register a cross-shard transaction
    pub fn register_transaction(&mut self, tx_id: String, shards: Vec<ShardId>) {
        let shard_set: HashSet<_> = shards.into_iter().collect();
        self.transaction_shards.insert(tx_id.clone(), shard_set);
        
        // Initialize commitments
        let mut shard_commitments = HashMap::new();
        for shard_id in self.transaction_shards.get(&tx_id).unwrap() {
            shard_commitments.insert(shard_id.clone(), false);
        }
        self.commitments.insert(tx_id, shard_commitments);
    }
    
    /// Record a commitment from a shard
    pub fn record_commitment(&mut self, tx_id: &str, shard_id: &ShardId, committed: bool) -> bool {
        if let Some(shard_commitments) = self.commitments.get_mut(tx_id) {
            if let Some(commitment) = shard_commitments.get_mut(shard_id) {
                *commitment = committed;
                
                // Check if all shards have committed
                return shard_commitments.values().all(|&c| c);
            }
        }
        false
    }
    
    /// Check if transaction is ready for commitment
    pub fn is_transaction_ready(&self, tx_id: &str) -> bool {
        if let Some(shard_commitments) = self.commitments.get(tx_id) {
            return !shard_commitments.is_empty() && shard_commitments.values().all(|&c| c);
        }
        false
    }
    
    /// Get all involved shards for a transaction
    pub fn get_transaction_shards(&self, tx_id: &str) -> Option<Vec<ShardId>> {
        self.transaction_shards.get(tx_id)
            .map(|set| set.iter().cloned().collect())
    }
    
    /// Remove transaction records
    pub fn remove_transaction(&mut self, tx_id: &str) {
        self.transaction_shards.remove(tx_id);
        self.commitments.remove(tx_id);
    }
}
