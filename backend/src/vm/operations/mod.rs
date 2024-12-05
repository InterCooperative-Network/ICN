// src/vm/operations/mod.rs

use std::collections::HashMap;
use crate::vm::{VMError, VMResult};
use crate::vm::event::Event;

// Re-export operation modules
pub mod stack;
pub mod arithmetic;
pub mod cooperative;
pub mod governance;
pub mod reputation;
pub mod relationship;
pub mod system;
pub mod data;
pub mod memory;
pub mod network;
pub mod federation;

// Re-export necessary operation types
pub use stack::StackOperation;
pub use arithmetic::ArithmeticOperation;
pub use system::SystemOperation;
pub use relationship::RelationshipOperation;
pub use memory::MemoryOperation;

/// VM state structure
#[derive(Default)]
pub struct VMState {
    pub stack: Vec<i64>,
    pub memory: HashMap<String, i64>,
    pub events: Vec<Event>,
    pub instruction_pointer: usize,
    pub reputation_context: HashMap<String, i64>,
    pub caller_did: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub permissions: Vec<String>,
    pub memory_limit: u64,
    pub memory_address_counter: AtomicU64,
    // Add missing fields
    pub state_tree: MerkleTree,
    pub state_updates: HashMap<String, String>,
}

/// Trait for implementable VM operations
pub trait Operation {
    /// Execute the operation on the given state
    fn execute(&self, state: &mut VMState) -> VMResult<()>;
    
    /// Get the resource cost of this operation
    fn resource_cost(&self) -> u64;
    
    /// Get required permissions for this operation
    fn required_permissions(&self) -> Vec<String>;
}

/// Helper function to check stack has enough items
pub fn ensure_stack_size(stack: &[i64], required: usize) -> VMResult<()> {
    if stack.len() < required {
        Err(VMError::StackUnderflow)
    } else {
        Ok(())
    }
}

/// Helper function to check caller permissions
pub fn ensure_permissions(required: &[String], available: &[String]) -> VMResult<()> {
    for perm in required {
        if !available.contains(perm) {
            return Err(VMError::InsufficientPermissions);
        }
    }
    Ok(())
}

/// Helper function to check reputation requirements 
pub fn ensure_reputation(required: i64, available: i64) -> VMResult<()> {
    if available < required {
        Err(VMError::InsufficientReputation)
    } else {
        Ok(())
    }
}

/// Helper function to emit an event
pub fn emit_event(state: &mut VMState, event_type: String, data: HashMap<String, String>) {
    let context = crate::vm::event::EventContext {
        triggered_by: state.caller_did.clone(),
        block_number: state.block_number,
        source_module: "vm".to_string(),
        transaction_id: None,
    };

    state.events.push(Event {
        event_type,
        cooperative_id: String::new(),
        data,
        timestamp: state.timestamp,
        context: Some(context)
    });
}

pub fn validate_state_update(key: &str, value: &str, state: &VMState) -> VMResult<()> {
    // Validate state update
    if key.is_empty() || value.is_empty() {
        return Err(VMError::InvalidOperand);
    }

    // Generate and verify state proof
    if let Some(root) = state.get_state_root() {
        let proof = state.state_tree.generate_proof(state.state_updates.len());
        if !MerkleTree::validate_proof(&format!("{}:{}", key, value), &root, proof) {
            return Err(VMError::ValidationError);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_stack_size() {
        let stack = vec![1, 2, 3];
        assert!(ensure_stack_size(&stack, 3).is_ok());
        assert!(ensure_stack_size(&stack, 4).is_err());
    }

    #[test]
    fn test_ensure_permissions() {
        let required = vec!["test.permission".to_string()];
        let available = vec!["test.permission".to_string()];
        assert!(ensure_permissions(&required, &available).is_ok());

        let available = vec!["other.permission".to_string()];
        assert!(ensure_permissions(&required, &available).is_err());
    }

    #[test]
    fn test_ensure_reputation() {
        assert!(ensure_reputation(10, 20).is_ok());
        assert!(ensure_reputation(20, 10).is_err());
    }
}