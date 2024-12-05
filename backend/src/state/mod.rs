// backend/src/state/mod.rs

mod merkle_tree;
mod persistence;
mod validation;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Invalid state transition")]
    InvalidTransition,
    #[error("State validation failed: {0}")]
    ValidationFailed(String),
    #[error("Merkle proof verification failed")]
    ProofVerificationFailed,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Concurrent modification detected")]
    ConcurrencyError,
}

pub type StateResult<T> = Result<T, StateError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateTransition {
    pub previous_root: String,
    pub next_root: String,
    pub changes: Vec<StateChange>,
    pub timestamp: u64,
    pub validator_signatures: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateChange {
    pub key: String,
    pub value: String,
    pub proof: Vec<String>,
}

pub struct StateManager {
    current_state: Arc<RwLock<SystemState>>,
    merkle_tree: merkle_tree::MerkleTree,
    persistence: persistence::StatePersistence,
    validator: validation::StateValidator,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemState {
    pub root_hash: String,
    pub block_height: u64,
    pub timestamp: u64,
    pub values: std::collections::HashMap<String, String>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            current_state: Arc::new(RwLock::new(SystemState::default())),
            merkle_tree: merkle_tree::MerkleTree::default(),
            persistence: persistence::StatePersistence::new(),
            validator: validation::StateValidator::new(),
        }
    }

    pub async fn apply_transition(&mut self, transition: StateTransition) -> StateResult<()> {
        // Validate transition
        self.validator.validate_transition(&transition)?;

        // Verify merkle proofs
        for change in &transition.changes {
            if !self.merkle_tree.validate_proof(&change.value, &transition.next_root, change.proof.clone()) {
                return Err(StateError::ProofVerificationFailed);
            }
        }

        // Acquire write lock and update state
        let mut state = self.current_state.write().await;
        
        // Ensure no concurrent modifications
        if state.root_hash != transition.previous_root {
            return Err(StateError::ConcurrencyError);
        }

        // Apply changes
        for change in transition.changes {
            state.values.insert(change.key, change.value);
        }

        // Update state metadata
        state.root_hash = transition.next_root;
        state.block_height += 1;
        state.timestamp = transition.timestamp;

        // Persist state
        self.persistence.store_transition(&transition)?;

        Ok(())
    }

    pub async fn get_value(&self, key: &str) -> StateResult<Option<String>> {
        let state = self.current_state.read().await;
        Ok(state.values.get(key).cloned())
    }

    pub async fn get_proof(&self, key: &str) -> StateResult<Vec<String>> {
        let state = self.current_state.read().await;
        if let Some(value) = state.values.get(key) {
            Ok(self.merkle_tree.generate_proof(format!("{}:{}", key, value)))
        } else {
            Ok(vec![])
        }
    }

    pub async fn verify_state(&self) -> StateResult<bool> {
        let state = self.current_state.read().await;
        self.validator.verify_invariants(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_state_transitions() {
        let mut manager = StateManager::new();
        
        let transition = StateTransition {
            previous_root: "0".to_string(),
            next_root: "1".to_string(),
            changes: vec![
                StateChange {
                    key: "test_key".to_string(),
                    value: "test_value".to_string(),
                    proof: vec![],
                }
            ],
            timestamp: 1000,
            validator_signatures: vec![],
        };

        assert!(manager.apply_transition(transition).await.is_ok());
        
        let value = manager.get_value("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    async fn test_concurrent_modifications() {
        let mut manager = StateManager::new();
        
        // Apply first transition
        let transition1 = StateTransition {
            previous_root: "0".to_string(),
            next_root: "1".to_string(),
            changes: vec![],
            timestamp: 1000,
            validator_signatures: vec![],
        };
        
        assert!(manager.apply_transition(transition1).await.is_ok());

        // Try to apply transition with old root
        let transition2 = StateTransition {
            previous_root: "0".to_string(), // Should be "1"
            next_root: "2".to_string(),
            changes: vec![],
            timestamp: 1001,
            validator_signatures: vec![],
        };
        
        assert!(matches!(
            manager.apply_transition(transition2).await,
            Err(StateError::ConcurrencyError)
        ));
    }
}