use crate::zk_snarks::ZkProof;
use icn_types::{Block, Transaction, BlockError, RuntimeInterface};
use crate::StorageInterface;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct Blockchain {
    storage: Box<dyn StorageInterface>,
    runtime: Box<dyn RuntimeInterface>,
    chain: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl Blockchain {
    pub fn new(storage: Box<dyn StorageInterface>, runtime: Box<dyn RuntimeInterface>) -> Self {
        Self {
            storage,
            runtime,
            chain: Arc::new(RwLock::new(Vec::new())),
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
        if let Some(proof) = transaction.proof {
            let inputs = transaction.get_zk_snark_inputs();
            let zk_proof = ZkProof {
                proof_data: proof,
                public_inputs: inputs,
            };
            if !crate::zk_snarks::verify_proof(&zk_proof, &inputs)? {
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

    pub async fn get_latest_block(&self) -> Option<Block> {
        self.chain.read().await.last().cloned()
    }
}
