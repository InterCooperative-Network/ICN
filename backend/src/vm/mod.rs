// src/vm/mod.rs

pub mod opcode;
pub mod contract;
pub mod execution_context;
pub mod cooperative_metadata;
pub mod event;
pub mod operations;
pub mod vm;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub use contract::Contract;
pub use vm::VM;
pub use execution_context::ExecutionContext;
pub use event::Event;
pub use operations::Operation;
pub use std::result::Result as OperationResult;

#[derive(Debug, Clone, PartialEq)]
pub enum VMError {
    StackUnderflow,
    StackOverflow,
    DivisionByZero,
    InvalidMemoryAccess,
    InvalidJumpDestination,
    InsufficientPermissions,
    InsufficientReputation,
    InvalidOperand,
    ExecutionLimitExceeded,
    OutOfMemory,
    InvalidMemoryAddress,
    ValidationError,
    Custom(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::StackOverflow => write!(f, "Stack overflow"),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::InvalidMemoryAccess => write!(f, "Invalid memory access"),
            VMError::InvalidJumpDestination => write!(f, "Invalid jump destination"),
            VMError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            VMError::InsufficientReputation => write!(f, "Insufficient reputation"),
            VMError::InvalidOperand => write!(f, "Invalid operand"),
            VMError::ExecutionLimitExceeded => write!(f, "Execution limit exceeded"),
            VMError::OutOfMemory => write!(f, "Out of memory"),
            VMError::InvalidMemoryAddress => write!(f, "Invalid memory address"),
            VMError::ValidationError => write!(f, "Validation failed"),
            VMError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<VMError> for String {
    fn from(error: VMError) -> String {
        error.to_string()
    }
}

pub type VMResult<T> = Result<T, VMError>;

#[derive(Default)]
pub struct VMState {
    /// Current stack
    pub stack: Vec<i64>,
    
    /// Memory storage
    pub memory: HashMap<String, i64>,
    
    /// Events emitted during execution
    pub events: Vec<Event>,
    
    /// Current instruction pointer
    pub instruction_pointer: usize,
    
    /// Reputation scores for participating DIDs
    pub reputation_context: HashMap<String, i64>,
    
    /// Currently executing DID
    pub caller_did: String,
    
    /// Current block number
    pub block_number: u64,
    
    /// Current timestamp
    pub timestamp: u64,
    
    /// Available permissions
    pub permissions: Vec<String>,
    
    /// Maximum memory usage in bytes
    pub memory_limit: u64,
    
    /// Counter for generating unique memory addresses
    pub memory_address_counter: AtomicU64,
}

impl VMState {
    pub fn new(caller_did: String, block_number: u64, timestamp: u64) -> Self {
        VMState {
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
        }
    }

    pub fn next_memory_address(&self) -> u64 {
        self.memory_address_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_reputation(&self) -> i64 {
        self.reputation_context.get(&self.caller_did).copied().unwrap_or(0)
    }

    pub fn incr_memory_usage(&mut self, size: u64) -> Result<(), VMError> {
        let current_usage = self.memory.len() as u64 * std::mem::size_of::<i64>() as u64;
        if current_usage + size > self.memory_limit {
            Err(VMError::OutOfMemory)
        } else {
            Ok(())
        }
    }

    pub fn get_stack(&self) -> &[i64] {
        &self.stack
    }

    pub fn get_memory(&self) -> &HashMap<String, i64> {
        &self.memory
    }

    pub fn get_events(&self) -> &[Event] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_state_new() {
        let state = VMState::new(
            "test_did".to_string(),
            1,
            1000
        );
        assert_eq!(state.caller_did, "test_did");
        assert_eq!(state.block_number, 1);
        assert_eq!(state.timestamp, 1000);
    }

    #[test]
    fn test_memory_limit() {
        let mut state = VMState::new("test_did".to_string(), 1, 1000);
        state.memory_limit = 100;
        assert!(state.incr_memory_usage(200).is_err());
        assert!(state.incr_memory_usage(50).is_ok());
    }
}