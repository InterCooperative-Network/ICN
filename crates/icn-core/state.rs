// File: crates/icn-core/src/state.rs
//
// State management system for the ICN network. Handles state transitions,
// validation, persistence, and synchronization across nodes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use blake3::Hash;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::config::Config;
use crate::metrics::SystemMetrics;

/// Core network state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    /// Current block height
    pub block_height: u64,
    
    /// Hash of the current state root
    pub state_root: Hash,
    
    /// Last block hash
    pub last_block_hash: Hash,
    
    /// Last block timestamp
    pub last_block_time: u64,
    
    /// Active validators and their voting power
    pub validators: HashMap<String, ValidatorState>,
    
    /// Network parameters
    pub params: NetworkParams,
    
    /// State version for migrations
    pub version: u32,
}

/// Validator state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorState {
    /// Validator's DID
    pub did: String,
    
    /// Current voting power
    pub voting_power: u64,
    
    /// Accumulated reputation score
    pub reputation: i64,
    
    /// Last active timestamp
    pub last_active: u64,
    
    /// Consecutive missed rounds
    pub missed_rounds: u32,
}

/// Network parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkParams {
    /// Minimum validator stake
    pub min_stake: u64,
    
    /// Maximum validator count
    pub max_validators: usize,
    
    /// Block time target in seconds
    pub block_time: u64,
    
    /// Maximum transaction size
    pub max_tx_size: usize,
    
    /// Maximum block size
    pub max_block_size: usize,
}

/// State manager handling state transitions and persistence
#[derive(Debug)]
pub struct StateManager {
    /// Current network state
    state: RwLock<NetworkState>,
    
    /// Configuration
    config: Arc<Config>,
    
    /// System metrics
    metrics: Arc<SystemMetrics>,
    
    /// State validation rules
    validators: Vec<Box<dyn StateValidator>>,
}

/// Trait for state validation rules
#[async_trait::async_trait]
pub trait StateValidator: Send + Sync {
    /// Validate a state transition
    async fn validate_transition(
        &self,
        current: &NetworkState,
        next: &NetworkState
    ) -> Result<()>;
}

impl StateManager {
    /// Create a new state manager
    pub async fn new(
        config: Arc<Config>,
        metrics: Arc<SystemMetrics>
    ) -> Result<Self> {
        let initial_state = Self::load_initial_state(&config).await?;
        
        let mut manager = Self {
            state: RwLock::new(initial_state),
            config,
            metrics,
            validators: Vec::new(),
        };
        
        // Register default validators
        manager.register_validator(Box::new(BlockHeightValidator));
        manager.register_validator(Box::new(TimestampValidator));
        manager.register_validator(Box::new(ValidatorSetValidator));
        
        Ok(manager)
    }

    /// Get current network state
    pub async fn get_state(&self) -> NetworkState {
        self.state.read().await.clone()
    }

    /// Attempt to apply a state transition
    pub async fn apply_state(&self, new_state: NetworkState) -> Result<()> {
        let current_state = self.state.read().await;
        
        // Validate state transition
        for validator in &self.validators {
            validator.validate_transition(&current_state, &new_state).await?;
        }
        
        // Calculate and verify state root
        let calculated_root = self.calculate_state_root(&new_state)?;
        if calculated_root != new_state.state_root {
            return Err(Error::validation("Invalid state root hash"));
        }
        
        // Update metrics
        self.metrics.blocks_stored.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Apply new state
        drop(current_state);
        *self.state.write().await = new_state;
        
        debug!("Applied new state at height {}", self.state.read().await.block_height);
        Ok(())
    }

    /// Register a new state validator
    pub fn register_validator(&mut self, validator: Box<dyn StateValidator>) {
        self.validators.push(validator);
    }

    /// Calculate state root hash
    fn calculate_state_root(&self, state: &NetworkState) -> Result<Hash> {
        let mut hasher = blake3::Hasher::new();
        
        // Add state components to hasher in deterministic order
        hasher.update(&state.block_height.to_le_bytes());
        hasher.update(state.last_block_hash.as_bytes());
        hasher.update(&state.last_block_time.to_le_bytes());
        
        // Add sorted validator states
        let mut validators: Vec<_> = state.validators.values().collect();
        validators.sort_by(|a, b| a.did.cmp(&b.did));
        
        for validator in validators {
            hasher.update(validator.did.as_bytes());
            hasher.update(&validator.voting_power.to_le_bytes());
            hasher.update(&validator.reputation.to_le_bytes());
        }
        
        // Add network parameters
        hasher.update(&state.params.min_stake.to_le_bytes());
        hasher.update(&state.params.max_validators.to_le_bytes());
        hasher.update(&state.params.block_time.to_le_bytes());
        
        Ok(hasher.finalize())
    }

    /// Load initial state from storage or create genesis
    async fn load_initial_state(config: &Config) -> Result<NetworkState> {
        // TODO: Implement actual state loading from storage
        // For now, return genesis state
        Ok(NetworkState {
            block_height: 0,
            state_root: Hash::from([0; 32]),
            last_block_hash: Hash::from([0; 32]),
            last_block_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            validators: HashMap::new(),
            params: NetworkParams {
                min_stake: 1000,
                max_validators: 100,
                block_time: 5,
                max_tx_size: 1024 * 1024, // 1MB
                max_block_size: 1024 * 1024 * 10, // 10MB
            },
            version: 1,
        })
    }
}

/// Validates block height transitions
#[derive(Debug)]
struct BlockHeightValidator;

#[async_trait::async_trait]
impl StateValidator for BlockHeightValidator {
    async fn validate_transition(
        &self,
        current: &NetworkState,
        next: &NetworkState
    ) -> Result<()> {
        if next.block_height != current.block_height + 1 {
            return Err(Error::validation(
                "Block height must increase by exactly 1"
            ));
        }
        Ok(())
    }
}

/// Validates block timestamp transitions
#[derive(Debug)]
struct TimestampValidator;

#[async_trait::async_trait]
impl StateValidator for TimestampValidator {
    async fn validate_transition(
        &self,
        current: &NetworkState,
        next: &NetworkState
    ) -> Result<()> {
        if next.last_block_time <= current.last_block_time {
            return Err(Error::validation(
                "Block timestamp must be greater than previous"
            ));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if next.last_block_time > now + 60 {
            return Err(Error::validation(
                "Block timestamp cannot be more than 60 seconds in the future"
            ));
        }
        
        Ok(())
    }
}

/// Validates validator set transitions
#[derive(Debug)]
struct ValidatorSetValidator;

#[async_trait::async_trait]
impl StateValidator for ValidatorSetValidator {
    async fn validate_transition(
        &self,
        _current: &NetworkState,
        next: &NetworkState
    ) -> Result<()> {
        // Check validator count
        if next.validators.len() > next.params.max_validators {
            return Err(Error::validation(
                "Validator set exceeds maximum size"
            ));
        }
        
        // Check validator stakes
        for validator in next.validators.values() {
            if validator.voting_power < next.params.min_stake {
                return Err(Error::validation(
                    "Validator has insufficient stake"
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    async fn setup_test_state() -> (Arc<Config>, Arc<SystemMetrics>) {
        let config = Arc::new(Config::builder()
            .with_test_defaults()
            .build()
            .unwrap());
            
        let metrics = Arc::new(SystemMetrics::new());
        
        (config, metrics)
    }

    #[tokio::test]
    async fn test_state_initialization() {
        let (config, metrics) = setup_test_state().await;
        let manager = StateManager::new(config, metrics).await.unwrap();
        
        let state = manager.get_state().await;
        assert_eq!(state.block_height, 0);
        assert_eq!(state.version, 1);
    }

    #[tokio::test]
    async fn test_state_transition() {
        let (config, metrics) = setup_test_state().await;
        let manager = StateManager::new(config, metrics).await.unwrap();
        
        let mut current = manager.get_state().await;
        let block_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Create valid next state
        let mut next = current.clone();
        next.block_height += 1;
        next.last_block_time = block_time;
        next.state_root = manager.calculate_state_root(&next).unwrap();
        
        assert!(manager.apply_state(next).await.is_ok());
        
        let new_state = manager.get_state().await;
        assert_eq!(new_state.block_height, 1);
    }

    #[tokio::test]
    async fn test_invalid_height_transition() {
        let (config, metrics) = setup_test_state().await;
        let manager = StateManager::new(config, metrics).await.unwrap();
        
        let mut current = manager.get_state().await;
        
        // Try to skip block height
        let mut next = current.clone();
        next.block_height += 2;
        next.state_root = manager.calculate_state_root(&next).unwrap();
        
        assert!(manager.apply_state(next).await.is_err());
    }

    #[tokio::test]
    async fn test_invalid_timestamp_transition() {
        let (config, metrics) = setup_test_state().await;
        let manager = StateManager::new(config, metrics).await.unwrap();
        
        let mut current = manager.get_state().await;
        
        // Try to use past timestamp
        let mut next = current.clone();
        next.block_height += 1;
        next.last_block_time = current.last_block_time - 1;
        next.state_root = manager.calculate_state_root(&next).unwrap();
        
        assert!(manager.apply_state(next).await.is_err());
    }

    #[tokio::test]
    async fn test_validator_set_validation() {
        let (config, metrics) = setup_test_state().await;
        let manager = StateManager::new(config, metrics).await.unwrap();
        
        let mut current = manager.get_state().await;
        
        // Add too many validators
        let mut next = current.clone();
        next.block_height += 1;
        next.last_block_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        for i in 0..next.params.max_validators + 1 {
            next.validators.insert(
                format!("validator-{}", i),
                ValidatorState {
                    did: format!("validator-{}", i),
                    voting_power: next.params.min_stake,
                    reputation: 0,
                    last_active: next.last_block_time,
                    missed_rounds: 0,
                }
            );
        }
        
        next.state_root = manager.calculate_state_root(&next).unwrap();
        assert!(manager.apply_state(next).await.is_err());
    }
}