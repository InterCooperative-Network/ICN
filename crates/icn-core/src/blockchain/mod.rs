// src/blockchain/mod.rs

mod block;
mod transaction;
mod chain;

// Re-export the core types
pub use block::Block;
pub use transaction::{Transaction, TransactionType};
pub use chain::Blockchain;

// Module-level constants
pub const MAX_BLOCK_SIZE: usize = 1000; // Maximum transactions per block
pub const MIN_REPUTATION_FOR_TXN: i64 = 10; // Minimum reputation to submit transactions