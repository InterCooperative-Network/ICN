use std::collections::HashMap;
use async_trait::async_trait;
use icn_types::{StorageError, StorageReference, StorageStatus, StorageType, Block};

#[async_trait]
pub trait StorageInterface: Send + Sync {
    async fn store_block(&self, block: Block) -> Result<(), StorageError>;
    async fn get_block(&self, hash: &str) -> Result<Block, StorageError>;
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, StorageError>;
}

pub struct StorageManager {
    references: HashMap<String, StorageReference>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            references: HashMap::new()
        }
    }
}

#[async_trait]
impl StorageInterface for StorageManager {
    async fn store_block(&self, _block: Block) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_block(&self, _hash: &str) -> Result<Block, StorageError> {
        unimplemented!()
    }

    async fn store(&self, _key: &str, _data: &[u8]) -> Result<(), StorageError> {
        Ok(())
    }

    async fn retrieve(&self, _key: &str) -> Result<Vec<u8>, StorageError> {
        unimplemented!()
    }
}
