//! ICN Storage implementation
//! 
//! This crate provides persistent storage and state management for the ICN network.

mod error;
pub mod storage;
pub mod state;
#[cfg(test)]
mod tests;

pub use error::{StorageError, StorageResult};
pub use storage::{StorageManager, StorageConfig};
pub use state::{StateManager, NetworkState};

/// Initialize the storage system
pub async fn init(config: StorageConfig) -> StorageResult<StorageManager> {
    let storage = StorageManager::new(config).await?;
    
    // Run any pending migrations
    storage.run_migrations().await?;
    
    Ok(storage)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_storage_initialization() {
        let config = StorageConfig::default();
        let storage = init(config).await.unwrap();
        assert!(storage.get_latest_block_height().await.unwrap() >= 0);
    }
}