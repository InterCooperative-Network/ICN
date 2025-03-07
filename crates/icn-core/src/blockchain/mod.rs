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
        if let Some(proof) = &transaction.zk_snark_proof {
            let inputs = transaction.get_zk_snark_inputs();
            let public_inputs = vec![inputs]; // Wrap in Vec<Vec<u8>> as required
            
            // Convert String to Vec<u8> for proof_data
            let proof_data = proof.as_bytes().to_vec();
            
            let zk_proof = ZkProof {
                proof_data,
                public_inputs: public_inputs.clone(), // Clone to avoid moving
            };
            
            // Handle the error conversion explicitly
            match crate::zk_snarks::verify_proof(&zk_proof, &public_inputs) {
                Ok(valid) => {
                    if !valid {
                        return Err("Invalid zk-SNARK proof".to_string());
                    }
                },
                Err(e) => {
                    return Err(format!("ZK proof verification error: {}", e));
                }
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
