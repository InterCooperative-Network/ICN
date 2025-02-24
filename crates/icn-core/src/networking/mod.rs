use std::sync::Arc;
use tokio::sync::broadcast;
use icn_types::{Block, Transaction};
use async_trait::async_trait;

#[async_trait]
pub trait NetworkInterface: Send + Sync {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn broadcast_block(&self, block: Block) -> Result<(), String>;
    async fn broadcast_transaction(&self, transaction: Transaction) -> Result<(), String>;
}

pub struct NetworkManager {
    block_tx: broadcast::Sender<Block>,
    transaction_tx: broadcast::Sender<Transaction>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let (block_tx, _) = broadcast::channel(100);
        let (transaction_tx, _) = broadcast::channel(100);
        Self {
            block_tx,
            transaction_tx,
        }
    }
}

#[async_trait]
impl NetworkInterface for NetworkManager {
    async fn start(&self) -> Result<(), String> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    async fn broadcast_block(&self, block: Block) -> Result<(), String> {
        self.block_tx.send(block)
            .map_err(|e| format!("Failed to broadcast block: {}", e))?;
        Ok(())
    }

    async fn broadcast_transaction(&self, transaction: Transaction) -> Result<(), String> {
        self.transaction_tx.send(transaction)
            .map_err(|e| format!("Failed to broadcast transaction: {}", e))?;
        Ok(())
    }
}
