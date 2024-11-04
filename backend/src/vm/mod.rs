// backend/src/vm/mod.rs

pub mod opcode;
pub mod contract;
pub mod vm;
pub mod execution_context;
pub mod cooperative_metadata;
pub mod event;

pub use contract::Contract;
pub use vm::VM;
pub use execution_context::ExecutionContext;