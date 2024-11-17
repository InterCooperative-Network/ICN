// src/vm/mod.rs 

pub mod opcode;
pub mod contract;
pub mod execution_context;
pub mod cooperative_metadata;
pub mod event;
pub mod operations;
pub mod vm;

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