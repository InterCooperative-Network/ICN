// src/consensus/state_sync.rs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::broadcast;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::blockchain::Block;
use crate::consensus::types::{ConsensusError, NetworkState};
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};
use crate::websocket::WebSocketHandler;

/// Represents a state diff between two network states
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDiff {
    pub from_height: u64,
    pub to_height: u64,
    pub block_hashes: Vec<String>,
    pub state_updates: HashMap<String, String>,
    pub validator_updates: HashMap<String, i64>,
    pub timestamp: DateTime<Utc>,
}

/// Manages state synchronization across the network
pub struct StateSynchronizer {
    /// Current network state
    current_state: NetworkState,
    
    /// Pending state updates
    pending_updates: HashMap<u64, StateDiff>,
    
    /// Set of validators we're currently syncing with
    sync_peers: HashSet<String>,
    
    /// WebSocket handler for network communication
    ws_handler: Arc<WebSocketHandler>,
    
    /// Event broadcaster
    event_tx: broadcast::Sender<StateEvent>,
    
    /// Last successful sync timestamp
    last_sync: DateTime<Utc>,
    
    /// Verification checkpoints
    checkpoints: Vec<(u64, String)>, // (height, state_root)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateEvent {
    SyncStarted {
        target_height: u64,
        peer_count: usize,
    },
    SyncProgress {
        current_height: u64,
        target_height: u64,
        validated_states: u64,
    },
    SyncCompleted {
        final_height: u64,
        duration_ms: u64,
    },
    SyncFailed {
        error: String,
        retry_count: u32,
    },
}

impl StateSynchronizer {
    pub fn new(
        initial_state: NetworkState,
        ws_handler: Arc<WebSocketHandler>
    ) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        StateSynchronizer {
            current_state: initial_state,
            pending_updates: HashMap::new(),
            sync_peers: HashSet::new(),
            ws_handler,
            event_tx,
            last_sync: Utc::now(),
            checkpoints: Vec::new(),
        }
    }

    /// Starts state synchronization with network
    pub async fn start_sync(&mut self, target_height: u64) -> Result<(), ConsensusError> {
        let start_time = Utc::now();
        
        // Notify sync start
        let _ = self.event_tx.send(StateEvent::SyncStarted {
            target_height,
            peer_count: self.sync_peers.len(),
        });

        // Request state diffs from peers
        for peer_did in &self.sync_peers {
            self.request_state_diff(peer_did, self.current_state.block_height, target_height).await?;
        }

        // Track sync progress
        let mut current_height = self.current_state.block_height;
        while current_height < target_height {
            if let Some(diff) = self.pending_updates.get(&(current_height + 1)) {
                // Validate state transition
                self.validate_state_diff(diff)?;
                
                // Apply validated diff
                self.apply_state_diff(diff).await?;
                
                // Update progress
                current_height += 1;
                let _ = self.event_tx.send(StateEvent::SyncProgress {
                    current_height,
                    target_height,
                    validated_states: current_height - self.current_state.block_height,
                });
            }
        }

        // Record checkpoint
        self.checkpoints.push((
            self.current_state.block_height,
            self.current_state.state_root.clone()
        ));

        // Calculate duration
        let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

        // Notify completion
        let _ = self.event_tx.send(StateEvent::SyncCompleted {
            final_height: current_height,
            duration_ms: duration,
        });

        self.last_sync = Utc::now();
        Ok(())
    }

    /// Validates a state diff before applying
    fn validate_state_diff(&self, diff: &StateDiff) -> Result<(), ConsensusError> {
        // Verify continuity
        if diff.from_height != self.current_state.block_height {
            return Err(ConsensusError::InvalidStateTransition);
        }

        // Verify block hashes
        for hash in &diff.block_hashes {
            if !self.verify_block_hash(hash) {
                return Err(ConsensusError::InvalidBlockHash);
            }
        }

        // Verify validator updates
        for (did, reputation) in &diff.validator_updates {
            if !self.verify_validator_update(did, *reputation) {
                return Err(ConsensusError::InvalidValidatorUpdate);
            }
        }

        Ok(())
    }

    /// Applies a validated state diff
    async fn apply_state_diff(&mut self, diff: &StateDiff) -> Result<(), ConsensusError> {
        // Update state root
        self.current_state.state_root = self.calculate_new_state_root(&diff.state_updates);
        
        // Update block height
        self.current_state.block_height = diff.to_height;
        
        // Update validator set
        for (did, reputation) in &diff.validator_updates {
            if let Some(validator) = self.current_state.validator_set
                .iter_mut()
                .find(|v| v.did == *did)
            {
                validator.reputation = *reputation;
            }
        }

        // Update timestamp
        self.current_state.timestamp = Utc::now().timestamp();
        
        // Notify state update
        self.ws_handler.broadcast_state_update(&self.current_state);

        Ok(())
    }

    /// Requests state diff from a peer
    async fn request_state_diff(
        &self,
        peer_did: &str,
        from_height: u64,
        to_height: u64
    ) -> Result<(), ConsensusError> {
        // TODO: Implement actual network request
        // For now just return success
        Ok(())
    }

    // Helper methods
    fn verify_block_hash(&self, hash: &str) -> bool {
        // TODO: Implement actual hash verification
        true
    }

    fn verify_validator_update(&self, did: &str, reputation: i64) -> bool {
        // TODO: Implement actual validator update verification
        true
    }

    fn calculate_new_state_root(&self, updates: &HashMap<String, String>) -> String {
        // TODO: Implement merkle root calculation
        "new_state_root".to_string()
    }

    // Public getters
    pub fn get_current_state(&self) -> &NetworkState {
        &self.current_state
    }

    pub fn get_sync_status(&self) -> (u64, DateTime<Utc>) {
        (self.current_state.block_height, self.last_sync)
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<StateEvent> {
        self.event_tx.subscribe()
    }
}

impl EnergyAware for StateSynchronizer {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record state size
        let state_size = std::mem::size_of::<NetworkState>() as u64;
        monitor.record_memory_operation(state_size);
        
        // Record pending updates size
        let updates_size = (self.pending_updates.len() * std::mem::size_of::<StateDiff>()) as u64;
        monitor.record_storage_operation(updates_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_sync() -> StateSynchronizer {
        let initial_state = NetworkState {
            block_height: 0,
            state_root: "genesis".to_string(),
            validator_set: Vec::new(),
            timestamp: Utc::now().timestamp(),
            merkle_proof: Vec::new(),
        };

        StateSynchronizer::new(
            initial_state,
            Arc::new(WebSocketHandler::new())
        )
    }

    #[tokio::test]
    async fn test_sync_start() {
        let mut sync = setup_test_sync();
        let result = sync.start_sync(10).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_diff_validation() {
        let sync = setup_test_sync();
        
        let valid_diff = StateDiff {
            from_height: 0,
            to_height: 1,
            block_hashes: vec!["valid_hash".to_string()],
            state_updates: HashMap::new(),
            validator_updates: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        assert!(sync.validate_state_diff(&valid_diff).is_ok());
    }

    #[tokio::test]
    async fn test_state_diff_application() {
        let mut sync = setup_test_sync();
        
        let diff = StateDiff {
            from_height: 0,
            to_height: 1,
            block_hashes: vec!["valid_hash".to_string()],
            state_updates: HashMap::new(),
            validator_updates: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        assert!(sync.apply_state_diff(&diff).await.is_ok());
        assert_eq!(sync.get_current_state().block_height, 1);
    }
}