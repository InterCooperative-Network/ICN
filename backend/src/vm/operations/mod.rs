// src/vm/operations/mod.rs

// Change privacy levels of modules
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

pub use stack::StackOperation;
pub use arithmetic::ArithmeticOperation;
pub use cooperative::CooperativeOperation;
pub use governance::GovernanceOperation;
pub use reputation::ReputationOperation;
pub use relationship::RelationshipOperation;
pub use system::SystemOperation;
pub use data::DataOperation;
pub use memory::MemoryOperation;
pub use network::NetworkOperation;
pub use federation::FederationOperation;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::vm::{VMError, VMState, VMResult, Event};

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
    state.events.push(Event {
        event_type,
        cooperative_id: String::new(),
        data,
        timestamp: state.timestamp,
    });
}