//src/vm/mod.rs

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

// Re-export core operation traits
pub use operations::Operation;
pub use operations::OperationResult;

// Error type for VM operations
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
            VMError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

pub type VMResult<T> = Result<T, VMError>;

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
    pub permissions: Vec<String>,
    pub memory_limit: u64,
    memory_address_counter: AtomicU64,
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
}