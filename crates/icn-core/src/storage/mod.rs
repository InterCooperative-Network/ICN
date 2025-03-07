use async_trait::async_trait;
use icn_types::{Block, Transaction, StorageError};
use std::collections::HashMap;
use std::sync::RwLock;

#[async_trait]
pub trait StorageInterface: Send + Sync {
    async fn store_block(&self, block: &Block) -> Result<(), StorageError>;
    async fn get_block(&self, block_id: &str) -> Result<Block, StorageError>;
    async fn store_transaction(&self, transaction: &Transaction) -> Result<(), StorageError>;
    async fn get_transaction(&self, transaction_id: &str) -> Result<Transaction, StorageError>;
}

// Helper function instead of implementing on StorageError directly
fn create_db_error(msg: impl Into<String>) -> StorageError {
    StorageError::DatabaseError(msg.into())
}

pub struct MemoryStorage {
    blocks: RwLock<HashMap<String, Block>>,
    transactions: RwLock<HashMap<String, Transaction>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StorageInterface for MemoryStorage {
    async fn store_block(&self, block: &Block) -> Result<(), StorageError> {
        let mut blocks = self.blocks.write().map_err(|_| create_db_error("Lock error"))?;
        blocks.insert(block.hash.clone(), block.clone());
        Ok(())
    }

    async fn get_block(&self, block_id: &str) -> Result<Block, StorageError> {
        let blocks = self.blocks.read().map_err(|_| create_db_error("Lock error"))?;
        blocks.get(block_id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound("Block not found".into()))
    }

    async fn store_transaction(&self, transaction: &Transaction) -> Result<(), StorageError> {
        let mut transactions = self.transactions.write().map_err(|_| create_db_error("Lock error"))?;
        transactions.insert(transaction.id.clone(), transaction.clone());
        Ok(())
    }

    async fn get_transaction(&self, transaction_id: &str) -> Result<Transaction, StorageError> {
        let transactions = self.transactions.read().map_err(|_| create_db_error("Lock error"))?;
        transactions.get(transaction_id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound("Transaction not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new();
        
        let block = Block::default(); // Assuming Block has Default implementation
        storage.store_block(&block).await.unwrap();
        let retrieved = storage.get_block(&block.hash).await.unwrap();
        assert_eq!(block.hash, retrieved.hash);

        let tx = Transaction::default(); // Assuming Transaction has Default implementation
        storage.store_transaction(&tx).await.unwrap();
        let retrieved = storage.get_transaction(&tx.id).await.unwrap();
        assert_eq!(tx.id, retrieved.id);
    }
}
