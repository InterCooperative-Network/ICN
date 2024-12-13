use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use crate::state::merkle_tree::MerkleTree;
use crate::vm::{VMError, VMResult};
use crate::vm::event::Event;

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
    pub state_tree: MerkleTree,
    pub state_updates: HashMap<String, String>,
}

impl VMState {
    pub fn new(caller_did: String, block_number: u64, timestamp: u64) -> Self {
        let mut state = VMState {
            stack: Vec::new(),
            memory: HashMap::new(),
            events: Vec::new(),
            instruction_pointer: 0,
            reputation_context: HashMap::new(),
            caller_did,
            block_number,
            timestamp,
            permissions: Vec::new(),
            memory_limit: 1024 * 1024, // 1MB default limit
            memory_address_counter: AtomicU64::new(0),
            state_tree: MerkleTree::default(),
            state_updates: HashMap::new(),
        };
        
        state.state_tree.add_leaf(&format!("init:{}", timestamp));
        state
    }

    pub fn record_state_update(&mut self, key: String, value: String) {
        self.state_updates.insert(key.clone(), value.clone());
        self.state_tree.add_leaf(&format!("{}:{}", key, value));
    }

    pub fn get_state_root(&self) -> Option<String> {
        self.state_tree.root().cloned()
    }
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
    if key.is_empty() || value.is_empty() {
        return Err(VMError::InvalidOperand);
    }

    if let Some(root) = state.get_state_root() {
        let proof = state.state_tree.generate_proof(state.state_updates.len());
        if !MerkleTree::validate_proof(&format!("{}:{}", key, value), &root, proof) {
            return Err(VMError::ValidationError);
        }
    }

