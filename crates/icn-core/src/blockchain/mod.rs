use std::sync::Arc;
use tokio::sync::RwLock;
use icn_types::{Block, Transaction, BlockError};
use zk_snarks::verify_proof; // Import zk-SNARK verification function

pub struct Blockchain {
    chain: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Self {
            chain: Arc::new(RwLock::new(vec![genesis])),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_block(&self, block: Block) -> Result<(), BlockError> {
        let mut chain = self.chain.write().await;
        if let Some(previous_block) = chain.last() {
            block.verify(Some(previous_block)).await?;
        }
        chain.push(block);
        Ok(())
    }

    pub async fn add_transaction(&self, transaction: Transaction) -> Result<(), String> {
        if let Some(proof) = &transaction.zk_snark_proof {
            if !verify_proof(proof) {
                return Err("Invalid zk-SNARK proof".to_string());
            }
        }
        let mut pending = self.pending_transactions.write().await;
        pending.push(transaction);
        Ok(())
    }

    pub async fn get_pending_transactions(&self) -> Vec<Transaction> {
        self.pending_transactions.read().await.clone()
    }

    pub async fn clear_pending_transactions(&self) {
        self.pending_transactions.write().await.clear();
    }

    pub async fn get_latest_block(&self) -> Block {
        self.chain.read().await.last().unwrap().clone()
    }

    pub async fn verify_zk_snark_proof(&self, proof: &str) -> Result<bool, String> {
        if !verify_proof(proof) {
            return Err("Invalid zk-SNARK proof".to_string());
        }
        Ok(true)
    }
}
