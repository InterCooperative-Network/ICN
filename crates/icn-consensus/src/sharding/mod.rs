use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use icn_types::{Block, Transaction};
use crate::validation::ValidationError;
use crate::proof_of_cooperation::ProofOfCooperation;

mod allocation;
mod rebalancing;
mod routing;
mod cross_shard;

pub use allocation::ShardAllocation;
pub use rebalancing::RebalancingStrategy;
pub use routing::ShardRouter;
pub use cross_shard::{CrossShardConsensus, CrossShardTransaction, CrossShardStatus};

#[derive(Error, Debug)]
pub enum ShardingError {
    #[error("Invalid shard configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Shard not found: {0}")]
    ShardNotFound(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    
    #[error("Rebalancing error: {0}")]
    RebalancingError(String),
}

pub type ShardingResult<T> = Result<T, ShardingError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Initial number of shards
    pub initial_shards: u32,
    
    /// Maximum number of shards allowed
    pub max_shards: u32,
    
    /// Minimum number of validators per shard
    pub min_validators_per_shard: u32,
    
    /// Maximum transactions per shard
    pub max_transactions_per_shard: u32,
    
    /// Load threshold for triggering rebalancing (0.0 - 1.0)
    pub rebalancing_threshold: f64,
    
    /// Minimum reputation required to be a shard validator
    pub min_validator_reputation: i64,
    
    /// Cross-shard transaction timeout in seconds
    pub cross_shard_timeout: u64,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            initial_shards: 4,
            max_shards: 16,
            min_validators_per_shard: 4,
            max_transactions_per_shard: 1000,
            rebalancing_threshold: 0.7,
            min_validator_reputation: 100,
            cross_shard_timeout: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub id: u32,
    pub validator_set: HashSet<String>,
    pub transaction_count: u32,
    pub load_factor: f64,
    pub last_block: Option<Block>,
    pub creation_time: u64,
}

pub struct ShardManager {
    config: ShardConfig,
    shards: Arc<RwLock<HashMap<u32, ShardInfo>>>,
    router: Arc<ShardRouter>,
    allocator: Arc<ShardAllocation>,
    rebalancer: Arc<RebalancingStrategy>,
    consensus: Arc<ProofOfCooperation>,
    cross_shard: Arc<CrossShardConsensus>,
}

impl ShardManager {
    pub fn new(
        config: ShardConfig,
        consensus: Arc<ProofOfCooperation>,
    ) -> ShardingResult<Self> {
        if config.initial_shards > config.max_shards {
            return Err(ShardingError::InvalidConfig(
                "Initial shard count exceeds maximum".to_string()
            ));
        }
        
        let router = Arc::new(ShardRouter::new());
        let allocator = Arc::new(ShardAllocation::new(config.clone()));
        let rebalancer = Arc::new(RebalancingStrategy::new(config.clone()));
        let cross_shard = Arc::new(CrossShardConsensus::new(config.clone(), consensus.clone()));
        
        let mut manager = Self {
            config,
            shards: Arc::new(RwLock::new(HashMap::new())),
            router,
            allocator,
            rebalancer,
            consensus,
            cross_shard,
        };
        
        // Initialize shards
        manager.initialize_shards()?;
        
        Ok(manager)
    }
    
    async fn initialize_shards(&mut self) -> ShardingResult<()> {
        let mut shards = self.shards.write().await;
        
        for i in 0..self.config.initial_shards {
            let shard = ShardInfo {
                id: i,
                validator_set: HashSet::new(),
                transaction_count: 0,
                load_factor: 0.0,
                last_block: None,
                creation_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            shards.insert(i, shard);
        }
        
        Ok(())
    }
    
    pub async fn assign_transaction(&self, transaction: Transaction) -> ShardingResult<u32> {
        // Determine target shard using router
        let shard_id = self.router.route_transaction(&transaction).await?;
        
        // Check if this is a cross-shard transaction
        if let Some(target_shard) = self.is_cross_shard_transaction(&transaction).await {
            // Handle cross-shard transaction
            let completion_rx = self.cross_shard.begin_transaction(
                transaction.clone(),
                shard_id,
                target_shard,
            ).await?;

            // Wait for completion or timeout
            tokio::select! {
                status = completion_rx => {
                    match status {
                        Ok(CrossShardStatus::Committed) => Ok(shard_id),
                        Ok(status) => Err(ShardingError::ConsensusError(
                            format!("Cross-shard transaction failed: {:?}", status)
                        )),
                        Err(e) => Err(ShardingError::ConsensusError(
                            format!("Cross-shard transaction error: {}", e)
                        )),
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.config.cross_shard_timeout)) => {
                    Err(ShardingError::ConsensusError("Cross-shard transaction timed out".to_string()))
                }
            }
        } else {
            // Handle single-shard transaction
            let mut shards = self.shards.write().await;
            let shard = shards.get_mut(&shard_id).ok_or_else(|| {
                ShardingError::ShardNotFound(format!("Shard {} not found", shard_id))
            })?;
            
            shard.transaction_count += 1;
            shard.load_factor = shard.transaction_count as f64 / self.config.max_transactions_per_shard as f64;
            
            // Check if rebalancing is needed
            if shard.load_factor > self.config.rebalancing_threshold {
                self.trigger_rebalancing().await?;
            }
            
            Ok(shard_id)
        }
    }
    
    /// Check if a transaction needs cross-shard handling
    async fn is_cross_shard_transaction(&self, transaction: &Transaction) -> Option<u32> {
        // Get source and target shards
        let source_shard = self.router.route_transaction(transaction).await.ok()?;
        let target_shard = self.router.route_by_receiver(transaction).await.ok()?;

        if source_shard != target_shard {
            Some(target_shard)
        } else {
            None
        }
    }

    /// Get cross-shard transaction status
    pub async fn get_transaction_status(&self, tx_id: &str) -> Option<CrossShardStatus> {
        self.cross_shard.get_transaction_status(tx_id).await
    }

    /// Get all active cross-shard transactions
    pub async fn get_active_cross_shard_transactions(&self) -> Vec<CrossShardTransaction> {
        self.cross_shard.get_active_transactions().await
    }

    /// Maintenance task to check for timed out transactions
    pub async fn check_transaction_timeouts(&self) -> ShardingResult<()> {
        self.cross_shard.check_timeouts().await
    }
    
    pub async fn add_validator(&self, shard_id: u32, validator_id: String) -> ShardingResult<()> {
        let mut shards = self.shards.write().await;
        let shard = shards.get_mut(&shard_id).ok_or_else(|| {
            ShardingError::ShardNotFound(format!("Shard {} not found", shard_id))
        })?;
        
        shard.validator_set.insert(validator_id);
        Ok(())
    }
    
    pub async fn remove_validator(&self, shard_id: u32, validator_id: &str) -> ShardingResult<()> {
        let mut shards = self.shards.write().await;
        let shard = shards.get_mut(&shard_id).ok_or_else(|| {
            ShardingError::ShardNotFound(format!("Shard {} not found", shard_id))
        })?;
        
        shard.validator_set.remove(validator_id);
        
        // Check if we need to add more validators
        if shard.validator_set.len() < self.config.min_validators_per_shard as usize {
            self.allocator.allocate_validators(shard_id).await?;
        }
        
        Ok(())
    }
    
    pub async fn finalize_shard_block(&self, shard_id: u32) -> ShardingResult<Block> {
        let mut shards = self.shards.write().await;
        let shard = shards.get_mut(&shard_id).ok_or_else(|| {
            ShardingError::ShardNotFound(format!("Shard {} not found", shard_id))
        })?;
        
        // Create new block
        let block = Block {
            index: shard.last_block.as_ref().map(|b| b.index + 1).unwrap_or(0),
            previous_hash: shard.last_block.as_ref().map(|b| b.hash.clone()).unwrap_or_default(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            transactions: Vec::new(), // Will be filled by consensus
            hash: String::new(),      // Will be set by consensus
            proposer: String::new(),  // Will be set by consensus
            signatures: Vec::new(),    // Will be filled during consensus
        };
        
        // Run consensus
        let finalized_block = self.consensus.finalize_block(block).await
            .map_err(|e| ShardingError::ConsensusError(e.to_string()))?;
            
        // Update shard state
        shard.last_block = Some(finalized_block.clone());
        shard.transaction_count = 0;
        shard.load_factor = 0.0;
        
        Ok(finalized_block)
    }
    
    async fn trigger_rebalancing(&self) -> ShardingResult<()> {
        // Get current shard states
        let shards = self.shards.read().await;
        
        // Check if we can create new shards
        if shards.len() < self.config.max_shards as usize {
            self.rebalancer.create_new_shard().await?;
        } else {
            // Rebalance existing shards
            self.rebalancer.rebalance_shards().await?;
        }
        
        Ok(())
    }
    
    pub async fn get_shard_info(&self, shard_id: u32) -> ShardingResult<ShardInfo> {
        let shards = self.shards.read().await;
        shards.get(&shard_id)
            .cloned()
            .ok_or_else(|| ShardingError::ShardNotFound(format!("Shard {} not found", shard_id)))
    }
    
    pub async fn get_shard_count(&self) -> usize {
        self.shards.read().await.len()
    }
    
    pub async fn get_total_load(&self) -> f64 {
        let shards = self.shards.read().await;
        let total_load: f64 = shards.values().map(|s| s.load_factor).sum();
        total_load / shards.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_shard_initialization() {
        let config = ShardConfig::default();
        let consensus = Arc::new(ProofOfCooperation::new(/* ... */));
        let manager = ShardManager::new(config.clone(), consensus).unwrap();
        
        assert_eq!(manager.get_shard_count().await, config.initial_shards as usize);
    }
    
    #[tokio::test]
    async fn test_transaction_assignment() {
        let config = ShardConfig::default();
        let consensus = Arc::new(ProofOfCooperation::new(/* ... */));
        let manager = ShardManager::new(config.clone(), consensus).unwrap();
        
        let transaction = Transaction::default();
        let shard_id = manager.assign_transaction(transaction).await.unwrap();
        
        let shard = manager.get_shard_info(shard_id).await.unwrap();
        assert_eq!(shard.transaction_count, 1);
    }
    
    #[tokio::test]
    async fn test_validator_management() {
        let config = ShardConfig::default();
        let consensus = Arc::new(ProofOfCooperation::new(/* ... */));
        let manager = ShardManager::new(config.clone(), consensus).unwrap();
        
        let validator_id = "validator1".to_string();
        manager.add_validator(0, validator_id.clone()).await.unwrap();
        
        let shard = manager.get_shard_info(0).await.unwrap();
        assert!(shard.validator_set.contains(&validator_id));
        
        manager.remove_validator(0, &validator_id).await.unwrap();
        let shard = manager.get_shard_info(0).await.unwrap();
        assert!(!shard.validator_set.contains(&validator_id));
    }
} 