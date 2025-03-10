use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use icn_types::Transaction;
use super::{ShardingError, ShardingResult};

/// Strategies for routing transactions to shards
#[derive(Debug, Clone, Copy)]
pub enum RoutingStrategy {
    /// Route based on transaction hash
    Hash,
    
    /// Route based on sender address
    Sender,
    
    /// Route based on receiver address
    Receiver,
    
    /// Route based on transaction type
    TransactionType,
    
    /// Route to least loaded shard
    LeastLoaded,
}

pub struct ShardRouter {
    strategy: RoutingStrategy,
    route_cache: Arc<RwLock<HashMap<String, u32>>>,
    load_stats: Arc<RwLock<HashMap<u32, f64>>>,
}

impl ShardRouter {
    pub fn new() -> Self {
        Self {
            strategy: RoutingStrategy::Hash,
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            load_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_strategy(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            load_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn route_transaction(&self, transaction: &Transaction) -> ShardingResult<u32> {
        // Check cache first
        if let Some(shard_id) = self.check_cache(transaction).await {
            return Ok(shard_id);
        }
        
        // Route based on strategy
        let shard_id = match self.strategy {
            RoutingStrategy::Hash => self.route_by_hash(transaction).await?,
            RoutingStrategy::Sender => self.route_by_sender(transaction).await?,
            RoutingStrategy::Receiver => self.route_by_receiver(transaction).await?,
            RoutingStrategy::TransactionType => self.route_by_type(transaction).await?,
            RoutingStrategy::LeastLoaded => self.route_by_load().await?,
        };
        
        // Update cache
        self.update_cache(transaction, shard_id).await;
        
        Ok(shard_id)
    }
    
    async fn check_cache(&self, transaction: &Transaction) -> Option<u32> {
        let cache = self.route_cache.read().await;
        cache.get(&transaction.id).copied()
    }
    
    async fn update_cache(&self, transaction: &Transaction, shard_id: u32) {
        let mut cache = self.route_cache.write().await;
        cache.insert(transaction.id.clone(), shard_id);
        
        // Limit cache size
        if cache.len() > 10000 {
            // Remove oldest entries
            let keys: Vec<_> = cache.keys().take(1000).cloned().collect();
            for key in keys {
                cache.remove(&key);
            }
        }
    }
    
    async fn route_by_hash(&self, transaction: &Transaction) -> ShardingResult<u32> {
        // Use transaction ID as hash
        let hash = transaction.id.as_bytes();
        let shard_count = self.load_stats.read().await.len() as u32;
        
        if shard_count == 0 {
            return Err(ShardingError::InvalidConfig("No shards available".to_string()));
        }
        
        // Simple hash-based routing
        let mut hash_value = 0u32;
        for byte in hash {
            hash_value = hash_value.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        
        Ok(hash_value % shard_count)
    }
    
    async fn route_by_sender(&self, transaction: &Transaction) -> ShardingResult<u32> {
        let sender_hash = transaction.sender.as_bytes();
        let shard_count = self.load_stats.read().await.len() as u32;
        
        if shard_count == 0 {
            return Err(ShardingError::InvalidConfig("No shards available".to_string()));
        }
        
        // Hash sender address
        let mut hash_value = 0u32;
        for byte in sender_hash {
            hash_value = hash_value.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        
        Ok(hash_value % shard_count)
    }
    
    async fn route_by_receiver(&self, transaction: &Transaction) -> ShardingResult<u32> {
        let receiver_hash = transaction.receiver.as_bytes();
        let shard_count = self.load_stats.read().await.len() as u32;
        
        if shard_count == 0 {
            return Err(ShardingError::InvalidConfig("No shards available".to_string()));
        }
        
        // Hash receiver address
        let mut hash_value = 0u32;
        for byte in receiver_hash {
            hash_value = hash_value.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        
        Ok(hash_value % shard_count)
    }
    
    async fn route_by_type(&self, transaction: &Transaction) -> ShardingResult<u32> {
        let type_hash = transaction.transaction_type.as_bytes();
        let shard_count = self.load_stats.read().await.len() as u32;
        
        if shard_count == 0 {
            return Err(ShardingError::InvalidConfig("No shards available".to_string()));
        }
        
        // Hash transaction type
        let mut hash_value = 0u32;
        for byte in type_hash {
            hash_value = hash_value.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        
        Ok(hash_value % shard_count)
    }
    
    async fn route_by_load(&self) -> ShardingResult<u32> {
        let loads = self.load_stats.read().await;
        
        if loads.is_empty() {
            return Err(ShardingError::InvalidConfig("No shards available".to_string()));
        }
        
        // Find shard with minimum load
        let (shard_id, _) = loads.iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
            
        Ok(*shard_id)
    }
    
    pub async fn update_load(&self, shard_id: u32, load: f64) {
        let mut loads = self.load_stats.write().await;
        loads.insert(shard_id, load);
    }
    
    pub async fn get_load(&self, shard_id: u32) -> Option<f64> {
        let loads = self.load_stats.read().await;
        loads.get(&shard_id).copied()
    }
    
    pub async fn get_all_loads(&self) -> HashMap<u32, f64> {
        self.load_stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hash_routing() {
        let router = ShardRouter::with_strategy(RoutingStrategy::Hash);
        
        // Add some shards
        router.update_load(0, 0.5).await;
        router.update_load(1, 0.3).await;
        router.update_load(2, 0.7).await;
        
        let transaction = Transaction {
            id: "test_tx".to_string(),
            sender: "sender".to_string(),
            receiver: "receiver".to_string(),
            amount: 100,
            transaction_type: "transfer".to_string(),
            timestamp: 0,
            signature: None,
            metadata: Default::default(),
        };
        
        let shard_id = router.route_transaction(&transaction).await.unwrap();
        assert!(shard_id < 3);
    }
    
    #[tokio::test]
    async fn test_load_based_routing() {
        let router = ShardRouter::with_strategy(RoutingStrategy::LeastLoaded);
        
        // Add shards with different loads
        router.update_load(0, 0.8).await;
        router.update_load(1, 0.3).await;
        router.update_load(2, 0.5).await;
        
        let transaction = Transaction::default();
        let shard_id = router.route_transaction(&transaction).await.unwrap();
        
        // Should route to shard 1 (least loaded)
        assert_eq!(shard_id, 1);
    }
    
    #[tokio::test]
    async fn test_cache() {
        let router = ShardRouter::new();
        
        // Add a shard
        router.update_load(0, 0.5).await;
        
        let transaction = Transaction {
            id: "cached_tx".to_string(),
            ..Default::default()
        };
        
        // First routing should calculate
        let shard_id1 = router.route_transaction(&transaction).await.unwrap();
        
        // Second routing should use cache
        let shard_id2 = router.route_transaction(&transaction).await.unwrap();
        
        assert_eq!(shard_id1, shard_id2);
    }
} 