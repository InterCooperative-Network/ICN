// src/vm/operations/mod.rs

mod stack;
mod arithmetic;
mod cooperative;
mod governance;
mod reputation;
mod relationship;
mod system;

pub use stack::StackOperation;
pub use arithmetic::ArithmeticOperation;
pub use cooperative::CooperativeOperation;
pub use governance::GovernanceOperation;
pub use reputation::ReputationOperation;
pub use relationship::RelationshipOperation;
pub use system::SystemOperation;

use std::collections::HashMap;
use crate::vm::{VMError, VMResult, Event};

/// Trait for implementable VM operations
pub trait Operation {
    /// Execute the operation on the given state
    fn execute(&self, state: &mut VMState) -> VMResult<()>;
    
    /// Get the resource cost of this operation
    fn resource_cost(&self) -> u64;
    
    /// Get required permissions for this operation
    fn required_permissions(&self) -> Vec<String>;
}

/// Represents the complete state of the VM during execution
pub struct VMState {
    pub stack: Vec<i64>,
    pub memory: HashMap<String, i64>,
    pub events: Vec<Event>,
    pub instruction_pointer: usize,
    pub reputation_context: HashMap<String, i64>,
    pub caller_did: String,
    pub block_number: u64,
    pub timestamp: u64,
}

/// Result type for operation execution
pub type OperationResult = VMResult<()>;

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
        cooperative_id: String::new(), // Set as needed
        data,
        timestamp: state.timestamp,
    });
}