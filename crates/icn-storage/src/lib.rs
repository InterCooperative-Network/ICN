// src/lib.rs
mod error;
pub mod storage;
pub mod state;
#[cfg(test)]
mod tests;

pub use error::{StorageError, StorageResult};
pub use storage::StorageManager;
pub use state::StateManager;