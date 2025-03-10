use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{ShardConfig, ShardingError, ShardingResult, ShardInfo};

pub struct RebalancingStrategy {
    config: ShardConfig,
    shard_states: Arc<RwLock<HashMap<u32, ShardInfo>>>,
    rebalancing_history: Arc<RwLock<Vec<RebalancingEvent>>>,
}

#[derive(Debug, Clone)]
struct RebalancingEvent {
    timestamp: u64,
    event_type: RebalancingType,
    affected_shards: Vec<u32>,
    metrics_before: RebalancingMetrics,
    metrics_after: RebalancingMetrics,
}

#[derive(Debug, Clone)]
enum RebalancingType {
    ShardCreation,
    ShardMerge,
    LoadBalancing,
    ValidatorReassignment,
}

#[derive(Debug, Clone)]
struct RebalancingMetrics {
    average_load: f64,
    load_variance: f64,
    min_validators: usize,
    max_validators: usize,
}

impl RebalancingStrategy {
    pub fn new(config: ShardConfig) -> Self {
        Self {
            config,
            shard_states: Arc::new(RwLock::new(HashMap::new())),
            rebalancing_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_new_shard(&self) -> ShardingResult<u32> {
        let mut states = self.shard_states.write().await;
        
        // Get next shard ID
        let new_shard_id = states.keys().max().map(|id| id + 1).unwrap_or(0);
        
        if new_shard_id >= self.config.max_shards {
            return Err(ShardingError::InvalidConfig(
                "Maximum number of shards reached".to_string()
            ));
        }
        
        // Create metrics before change
        let metrics_before = self.calculate_metrics(&states).await;
        
        // Create new shard
        let new_shard = ShardInfo {
            id: new_shard_id,
            validator_set: HashSet::new(),
            transaction_count: 0,
            load_factor: 0.0,
            last_block: None,
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        states.insert(new_shard_id, new_shard);
        
        // Calculate metrics after change
        let metrics_after = self.calculate_metrics(&states).await;
        
        // Record rebalancing event
        self.record_event(
            RebalancingType::ShardCreation,
            vec![new_shard_id],
            metrics_before,
            metrics_after,
        ).await;
        
        Ok(new_shard_id)
    }

    pub async fn rebalance_shards(&self) -> ShardingResult<()> {
        let mut states = self.shard_states.write().await;
        
        // Calculate current metrics
        let metrics_before = self.calculate_metrics(&states).await;
        
        // Find overloaded and underloaded shards
        let (overloaded, underloaded) = self.identify_imbalanced_shards(&states).await;
        
        if overloaded.is_empty() && underloaded.is_empty() {
            return Ok(());
        }
        
        // Rebalance transactions between shards
        let mut affected_shards = Vec::new();
        for &overloaded_id in &overloaded {
            if let Some(&underloaded_id) = underloaded.first() {
                self.transfer_load(overloaded_id, underloaded_id, &mut states).await?;
                affected_shards.push(overloaded_id);
                affected_shards.push(underloaded_id);
            }
        }
        
        // Calculate metrics after rebalancing
        let metrics_after = self.calculate_metrics(&states).await;
        
        // Record rebalancing event
        self.record_event(
            RebalancingType::LoadBalancing,
            affected_shards,
            metrics_before,
            metrics_after,
        ).await;
        
        Ok(())
    }

    async fn identify_imbalanced_shards(
        &self,
        states: &HashMap<u32, ShardInfo>,
    ) -> (Vec<u32>, Vec<u32>) {
        let mut overloaded = Vec::new();
        let mut underloaded = Vec::new();
        
        let avg_load = states.values()
            .map(|s| s.load_factor)
            .sum::<f64>() / states.len() as f64;
            
        for (id, shard) in states {
            let load_diff = (shard.load_factor - avg_load).abs();
            if load_diff > self.config.rebalancing_threshold {
                if shard.load_factor > avg_load {
                    overloaded.push(*id);
                } else {
                    underloaded.push(*id);
                }
            }
        }
        
        // Sort by load difference
        overloaded.sort_by(|a, b| {
            let load_a = states[a].load_factor;
            let load_b = states[b].load_factor;
            load_b.partial_cmp(&load_a).unwrap()
        });
        
        underloaded.sort_by(|a, b| {
            let load_a = states[a].load_factor;
            let load_b = states[b].load_factor;
            load_a.partial_cmp(&load_b).unwrap()
        });
        
        (overloaded, underloaded)
    }

    async fn transfer_load(
        &self,
        from_shard: u32,
        to_shard: u32,
        states: &mut HashMap<u32, ShardInfo>,
    ) -> ShardingResult<()> {
        let from_state = states.get_mut(&from_shard)
            .ok_or_else(|| ShardingError::ShardNotFound(from_shard.to_string()))?;
            
        let to_state = states.get_mut(&to_shard)
            .ok_or_else(|| ShardingError::ShardNotFound(to_shard.to_string()))?;
            
        // Calculate transfer amount
        let transfer_amount = ((from_state.transaction_count as f64 - to_state.transaction_count as f64) / 2.0) as u32;
        
        // Update transaction counts
        from_state.transaction_count -= transfer_amount;
        to_state.transaction_count += transfer_amount;
        
        // Update load factors
        from_state.load_factor = from_state.transaction_count as f64 / self.config.max_transactions_per_shard as f64;
        to_state.load_factor = to_state.transaction_count as f64 / self.config.max_transactions_per_shard as f64;
        
        Ok(())
    }

    async fn calculate_metrics(&self, states: &HashMap<u32, ShardInfo>) -> RebalancingMetrics {
        let loads: Vec<_> = states.values().map(|s| s.load_factor).collect();
        let validator_counts: Vec<_> = states.values().map(|s| s.validator_set.len()).collect();
        
        let avg_load = if !loads.is_empty() {
            loads.iter().sum::<f64>() / loads.len() as f64
        } else {
            0.0
        };
        
        let load_variance = if !loads.is_empty() {
            loads.iter()
                .map(|l| (l - avg_load).powi(2))
                .sum::<f64>() / loads.len() as f64
        } else {
            0.0
        };
        
        RebalancingMetrics {
            average_load: avg_load,
            load_variance,
            min_validators: validator_counts.iter().min().copied().unwrap_or(0),
            max_validators: validator_counts.iter().max().copied().unwrap_or(0),
        }
    }

    async fn record_event(
        &self,
        event_type: RebalancingType,
        affected_shards: Vec<u32>,
        metrics_before: RebalancingMetrics,
        metrics_after: RebalancingMetrics,
    ) {
        let event = RebalancingEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type,
            affected_shards,
            metrics_before,
            metrics_after,
        };
        
        let mut history = self.rebalancing_history.write().await;
        history.push(event);
        
        // Keep only last 100 events
        if history.len() > 100 {
            history.remove(0);
        }
    }

    pub async fn get_rebalancing_history(&self) -> Vec<RebalancingEvent> {
        self.rebalancing_history.read().await.clone()
    }

    pub async fn get_shard_metrics(&self) -> RebalancingMetrics {
        let states = self.shard_states.read().await;
        self.calculate_metrics(&states).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shard_creation() {
        let config = ShardConfig::default();
        let strategy = RebalancingStrategy::new(config);
        
        let shard_id = strategy.create_new_shard().await.unwrap();
        assert_eq!(shard_id, 0);
        
        let states = strategy.shard_states.read().await;
        assert_eq!(states.len(), 1);
    }
    
    #[tokio::test]
    async fn test_load_balancing() {
        let config = ShardConfig::default();
        let strategy = RebalancingStrategy::new(config);
        
        // Create two shards with imbalanced loads
        let mut states = strategy.shard_states.write().await;
        
        let mut shard1 = ShardInfo {
            id: 0,
            validator_set: HashSet::new(),
            transaction_count: 800,
            load_factor: 0.8,
            last_block: None,
            creation_time: 0,
        };
        
        let mut shard2 = ShardInfo {
            id: 1,
            validator_set: HashSet::new(),
            transaction_count: 200,
            load_factor: 0.2,
            last_block: None,
            creation_time: 0,
        };
        
        states.insert(0, shard1);
        states.insert(1, shard2);
        drop(states);
        
        // Rebalance shards
        strategy.rebalance_shards().await.unwrap();
        
        // Check if loads are more balanced
        let states = strategy.shard_states.read().await;
        let shard1 = states.get(&0).unwrap();
        let shard2 = states.get(&1).unwrap();
        
        assert!(
            (shard1.load_factor - shard2.load_factor).abs() < config.rebalancing_threshold,
            "Loads should be more balanced after rebalancing"
        );
    }
} 