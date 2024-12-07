// src/consensus/state.rs

use crate::state::{StateManager, Migration};
use crate::storage::StorageResult;
use crate::blockchain::{Block, Transaction};
use crate::monitoring::metrics::{MetricsCollector, ConsensusMetrics};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Represents the current consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub current_round: u64,
    pub current_coordinator: String,
    pub active_validators: Vec<String>,
    pub pending_votes: Vec<ValidatorVote>,
    pub last_finalized_block: u64,
    pub round_start_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorVote {
    pub validator: String,
    pub vote: bool,
    pub signature: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Manages consensus-related state
pub struct ConsensusStateManager {
    state_manager: Arc<StateManager>,
    metrics: Arc<MetricsCollector>,
    current_state: Arc<RwLock<ConsensusState>>,
}

impl ConsensusStateManager {
    /// Create a new consensus state manager
    pub async fn new(
        state_manager: Arc<StateManager>,
        metrics: Arc<MetricsCollector>,
    ) -> StorageResult<Self> {
        // Try to load existing consensus state or create new
        let current_state = match state_manager.storage.retrieve::<ConsensusState>("consensus_state").await {
            Ok(state) => state,
            Err(_) => ConsensusState {
                current_round: 0,
                current_coordinator: String::new(),
                active_validators: Vec::new(),
                pending_votes: Vec::new(),
                last_finalized_block: 0,
                round_start_time: chrono::Utc::now(),
            },
        };

        Ok(Self {
            state_manager,
            metrics,
            current_state: Arc::new(RwLock::new(current_state)),
        })
    }

    /// Start a new consensus round
    pub async fn start_round(&self, coordinator: String) -> StorageResult<()> {
        let mut state = self.current_state.write().await;
        
        // Update round state
        state.current_round += 1;
        state.current_coordinator = coordinator;
        state.pending_votes.clear();
        state.round_start_time = chrono::Utc::now();

        // Record metrics
        self.metrics.record_consensus_round_started(
            state.current_round,
            &state.current_coordinator,
        );

        // Persist state
        self.state_manager.begin_batch().await?;
        self.state_manager.batch_store("consensus_state", &*state).await?;
        self.state_manager.commit_batch().await?;

        Ok(())
    }

    /// Record a validator vote
    pub async fn record_vote(&self, vote: ValidatorVote) -> StorageResult<()> {
        let mut state = self.current_state.write().await;
        
        // Add vote
        state.pending_votes.push(vote.clone());

        // Record metrics
        self.metrics.record_vote_cast(
            state.current_round,
            &vote.validator,
            vote.vote,
        );

        // Persist state
        self.state_manager.begin_batch().await?;
        self.state_manager.batch_store("consensus_state", &*state).await?;
        self.state_manager.commit_batch().await?;

        Ok(())
    }

    /// Finalize a consensus round with a new block
    pub async fn finalize_round(&self, block: Block) -> StorageResult<()> {
        self.state_manager.begin_batch().await?;
        
        // Update consensus state
        let mut state = self.current_state.write().await;
        state.last_finalized_block = block.height;
        
        // Store block
        self.state_manager
            .batch_store(&format!("block:{}", block.height), &block)
            .await?;
            
        // Store consensus state
        self.state_manager
            .batch_store("consensus_state", &*state)
            .await?;

        // Record metrics
        let round_duration = chrono::Utc::now()
            .signed_duration_since(state.round_start_time)
            .num_milliseconds();
            
        self.metrics.record_round_finalized(
            state.current_round,
            block.height,
            round_duration,
            block.transactions.len(),
        );

        // Commit all changes
        self.state_manager.commit_batch().await?;

        Ok(())
    }

    /// Get current consensus state
    pub async fn get_current_state(&self) -> ConsensusState {
        self.current_state.read().await.clone()
    }

    /// Check if consensus is achieved
    pub async fn check_consensus(&self, threshold: f64) -> bool {
        let state = self.current_state.read().await;
        
        if state.pending_votes.is_empty() {
            return false;
        }

        let total_votes = state.pending_votes.len() as f64;
        let positive_votes = state.pending_votes
            .iter()
            .filter(|v| v.vote)
            .count() as f64;

        positive_votes / total_votes >= threshold
    }

    /// Verify state consistency
    pub async fn verify_state(&self) -> StorageResult<bool> {
        let state = self.current_state.read().await;
        
        // Verify block continuity
        for height in 1..=state.last_finalized_block {
            let block_key = format!("block:{}", height);
            if !self.state_manager.storage.exists(&block_key).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::postgres::PostgresStorage;
    use crate::monitoring::metrics::MockMetricsCollector;
    use serial_test::serial;

    async fn setup_test_env() -> ConsensusStateManager {
        // Create test dependencies
        let storage = setup_test_storage().await;
        let state_manager = Arc::new(StateManager::new(Arc::new(storage)).await.unwrap());
        let metrics = Arc::new(MetricsCollector::new(Box::new(MockMetricsCollector::new())));

        ConsensusStateManager::new(state_manager, metrics)
            .await
            .expect("Failed to create consensus state manager")
    }

    #[tokio::test]
    #[serial]
    async fn test_consensus_round_lifecycle() {
        let manager = setup_test_env().await;

        // Start round
        manager.start_round("validator1".to_string()).await.unwrap();
        
        // Record votes
        let vote1 = ValidatorVote {
            validator: "validator1".to_string(),
            vote: true,
            signature: "sig1".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let vote2 = ValidatorVote {
            validator: "validator2".to_string(),
            vote: true,
            signature: "sig2".to_string(),
            timestamp: chrono::Utc::now(),
        };

        manager.record_vote(vote1).await.unwrap();
        manager.record_vote(vote2).await.unwrap();

        // Check consensus
        assert!(manager.check_consensus(0.66).await);

        // Finalize round
        let block = Block {
            height: 1,
            hash: "hash1".to_string(),
            previous_hash: "genesis".to_string(),
            timestamp: chrono::Utc::now(),
            transactions: vec![],
            proposer: "validator1".to_string(),
        };

        manager.finalize_round(block).await.unwrap();

        // Verify state
        let final_state = manager.get_current_state().await;
        assert_eq!(final_state.last_finalized_block, 1);
        assert!(manager.verify_state().await.unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_state_persistence() {
        let manager = setup_test_env().await;

        // Create initial state
        manager.start_round("validator1".to_string()).await.unwrap();
        
        // Create new manager instance with same storage
        let storage = manager.state_manager.storage.clone();
        let state_manager = Arc::new(StateManager::new(storage).await.unwrap());
        let metrics = Arc::new(MetricsCollector::new(Box::new(MockMetricsCollector::new())));
        
        let new_manager = ConsensusStateManager::new(state_manager, metrics)
            .await
            .expect("Failed to create new consensus state manager");

        // Verify state was loaded
        let state = new_manager.get_current_state().await;
        assert_eq!(state.current_round, 1);
        assert_eq!(state.current_coordinator, "validator1");
    }
}