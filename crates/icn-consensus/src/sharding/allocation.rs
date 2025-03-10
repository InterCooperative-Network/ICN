use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{ShardConfig, ShardingError, ShardingResult};
use icn_reputation::ReputationManager;

pub struct ShardAllocation {
    config: ShardConfig,
    reputation_manager: Arc<ReputationManager>,
    validator_assignments: Arc<RwLock<HashMap<String, u32>>>, // validator_id -> shard_id
    shard_validators: Arc<RwLock<HashMap<u32, HashSet<String>>>>, // shard_id -> validator_set
    validator_stats: Arc<RwLock<HashMap<String, ValidatorStats>>>,
}

#[derive(Debug, Clone)]
struct ValidatorStats {
    total_blocks_validated: u64,
    successful_validations: u64,
    failed_validations: u64,
    last_assignment: u64,
    performance_score: f64,
}

impl ValidatorStats {
    fn new() -> Self {
        Self {
            total_blocks_validated: 0,
            successful_validations: 0,
            failed_validations: 0,
            last_assignment: 0,
            performance_score: 1.0,
        }
    }

    fn update_performance(&mut self) {
        if self.total_blocks_validated > 0 {
            self.performance_score = self.successful_validations as f64 / self.total_blocks_validated as f64;
        }
    }
}

impl ShardAllocation {
    pub fn new(config: ShardConfig) -> Self {
        Self {
            config,
            reputation_manager: Arc::new(ReputationManager::new(/* storage interface needed */)),
            validator_assignments: Arc::new(RwLock::new(HashMap::new())),
            shard_validators: Arc::new(RwLock::new(HashMap::new())),
            validator_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn allocate_validators(&self, shard_id: u32) -> ShardingResult<()> {
        let mut shard_validators = self.shard_validators.write().await;
        let validators = shard_validators.entry(shard_id).or_insert_with(HashSet::new);
        
        // Check if we need more validators
        if validators.len() >= self.config.min_validators_per_shard as usize {
            return Ok(());
        }
        
        // Get all available validators
        let available_validators = self.get_available_validators().await?;
        
        // Sort validators by reputation and performance
        let mut ranked_validators = self.rank_validators(&available_validators).await?;
        
        // Allocate validators until minimum requirement is met
        let needed = self.config.min_validators_per_shard as usize - validators.len();
        for _ in 0..needed {
            if let Some(validator_id) = ranked_validators.pop() {
                self.assign_validator(validator_id, shard_id).await?;
            } else {
                return Err(ShardingError::InvalidConfig(
                    "Not enough qualified validators available".to_string()
                ));
            }
        }
        
        Ok(())
    }

    async fn get_available_validators(&self) -> ShardingResult<HashSet<String>> {
        let mut available = HashSet::new();
        let assignments = self.validator_assignments.read().await;
        
        // Get all validators from reputation manager
        let validators = self.reputation_manager.get_all_validators().await
            .map_err(|e| ShardingError::InvalidConfig(e.to_string()))?;
        
        for validator in validators {
            // Check if validator meets minimum reputation requirement
            let reputation = self.reputation_manager.get_reputation(&validator).await
                .map_err(|e| ShardingError::InvalidConfig(e.to_string()))?;
                
            if reputation >= self.config.min_validator_reputation && !assignments.contains_key(&validator) {
                available.insert(validator);
            }
        }
        
        Ok(available)
    }

    async fn rank_validators(&self, validators: &HashSet<String>) -> ShardingResult<Vec<String>> {
        let mut ranked = Vec::new();
        let stats = self.validator_stats.read().await;
        
        for validator in validators {
            let reputation = self.reputation_manager.get_reputation(validator).await
                .map_err(|e| ShardingError::InvalidConfig(e.to_string()))?;
                
            let performance = stats.get(validator)
                .map(|s| s.performance_score)
                .unwrap_or(1.0);
                
            ranked.push((validator.clone(), reputation as f64 * performance));
        }
        
        // Sort by score in descending order
        ranked.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        
        Ok(ranked.into_iter().map(|(id, _)| id).collect())
    }

    async fn assign_validator(&self, validator_id: String, shard_id: u32) -> ShardingResult<()> {
        // Update assignments
        let mut assignments = self.validator_assignments.write().await;
        assignments.insert(validator_id.clone(), shard_id);
        
        // Update shard validators
        let mut shard_validators = self.shard_validators.write().await;
        let validators = shard_validators.entry(shard_id).or_insert_with(HashSet::new);
        validators.insert(validator_id.clone());
        
        // Initialize stats if needed
        let mut stats = self.validator_stats.write().await;
        stats.entry(validator_id).or_insert_with(ValidatorStats::new);
        
        Ok(())
    }

    pub async fn record_validation_result(
        &self,
        validator_id: &str,
        success: bool,
    ) -> ShardingResult<()> {
        let mut stats = self.validator_stats.write().await;
        let stat = stats.entry(validator_id.to_string()).or_insert_with(ValidatorStats::new);
        
        stat.total_blocks_validated += 1;
        if success {
            stat.successful_validations += 1;
        } else {
            stat.failed_validations += 1;
        }
        
        stat.update_performance();
        
        // Check if validator needs to be reallocated due to poor performance
        if stat.performance_score < 0.7 {
            self.reallocate_validator(validator_id).await?;
        }
        
        Ok(())
    }

    async fn reallocate_validator(&self, validator_id: &str) -> ShardingResult<()> {
        // Remove from current shard
        let mut assignments = self.validator_assignments.write().await;
        if let Some(current_shard) = assignments.remove(validator_id) {
            let mut shard_validators = self.shard_validators.write().await;
            if let Some(validators) = shard_validators.get_mut(&current_shard) {
                validators.remove(validator_id);
            }
        }
        
        // Find least loaded shard
        let shard_loads = self.get_shard_loads().await;
        if let Some((&target_shard, _)) = shard_loads.iter().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            self.assign_validator(validator_id.to_string(), target_shard).await?;
        }
        
        Ok(())
    }

    async fn get_shard_loads(&self) -> HashMap<u32, f64> {
        let mut loads = HashMap::new();
        let shard_validators = self.shard_validators.read().await;
        
        for (&shard_id, validators) in shard_validators.iter() {
            loads.insert(shard_id, validators.len() as f64);
        }
        
        loads
    }

    pub async fn get_validator_shard(&self, validator_id: &str) -> Option<u32> {
        let assignments = self.validator_assignments.read().await;
        assignments.get(validator_id).copied()
    }

    pub async fn get_shard_validators(&self, shard_id: u32) -> Option<HashSet<String>> {
        let shard_validators = self.shard_validators.read().await;
        shard_validators.get(&shard_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validator_allocation() {
        let config = ShardConfig::default();
        let allocator = ShardAllocation::new(config);
        
        // Test allocating validators to a shard
        allocator.allocate_validators(0).await.unwrap();
        
        let validators = allocator.get_shard_validators(0).await.unwrap();
        assert!(validators.len() >= config.min_validators_per_shard as usize);
    }
    
    #[tokio::test]
    async fn test_validation_recording() {
        let config = ShardConfig::default();
        let allocator = ShardAllocation::new(config);
        
        // Record some validation results
        allocator.record_validation_result("validator1", true).await.unwrap();
        allocator.record_validation_result("validator1", true).await.unwrap();
        allocator.record_validation_result("validator1", false).await.unwrap();
        
        let stats = allocator.validator_stats.read().await;
        let stat = stats.get("validator1").unwrap();
        assert_eq!(stat.total_blocks_validated, 3);
        assert_eq!(stat.successful_validations, 2);
        assert_eq!(stat.failed_validations, 1);
        assert!(stat.performance_score > 0.6);
    }
} 