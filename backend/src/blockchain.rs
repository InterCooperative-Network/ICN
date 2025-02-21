use icn_types::{Block, Transaction};
use sha2::{Sha256, Digest};
use tendermint::block::Block as TendermintBlock;
use tendermint::lite::TrustedState;
use tendermint::rpc::Client;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub tendermint_client: Client,
    pub trusted_state: Arc<Mutex<TrustedState>>,
}

impl Blockchain {
    pub fn new(tendermint_client: Client, trusted_state: TrustedState) -> Self {
        Self {
            blocks: vec![],
            pending_transactions: vec![],
            tendermint_client,
            trusted_state: Arc::new(Mutex::new(trusted_state)),
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn get_block_by_index(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn process_pending_transactions(&mut self) -> Result<(), String> {
        let mut new_block = Block::new();
        for transaction in &self.pending_transactions {
            if self.validate_transaction(transaction)? {
                new_block.add_transaction(transaction.clone());
            } else {
                return Err("Invalid transaction".to_string());
            }
        }
        self.add_block(new_block);
        self.pending_transactions.clear();
        Ok(())
    }

    pub fn validate_transaction(&self, transaction: &Transaction) -> Result<bool, String> {
        // Placeholder logic for transaction validation
        Ok(true)
    }

    pub fn calculate_hash(block: &Block) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(&block.previous_hash);
        hasher.update(block.timestamp.to_string());
        for tx in &block.transactions {
            hasher.update(serde_json::to_string(tx).unwrap());
        }
        hasher.update(&block.proposer);
        format!("{:x}", hasher.finalize())
    }

    pub async fn start_consensus_round(&mut self, block: &mut Block) -> Result<(), String> {
        block.start_consensus_round().await.map_err(|e| e.to_string())
    }

    pub async fn vote_on_block(&mut self, block: &mut Block, validator_did: String, vote: bool) -> Result<(), String> {
        block.vote_on_block(validator_did, vote).await.map_err(|e| e.to_string())
    }

    pub async fn finalize_block(&mut self, block: &mut Block) -> Result<(), String> {
        if block.metadata.validator_count >= 3 { // Assuming 3 is the required number of validators for consensus
            block.finalize().await.map_err(|e| e.to_string())?;
            self.add_block(block.clone());
            Ok(())
        } else {
            Err("Consensus not reached".to_string())
        }
    }

    pub fn validate_contribution(&self, contribution: &Contribution) -> Result<bool, String> {
        // Placeholder logic for contribution validation
        Ok(true)
    }

    pub async fn propose_block(&self, block: TendermintBlock) -> Result<(), String> {
        // Placeholder logic for proposing a block using Tendermint
        Ok(())
    }

    pub async fn vote_on_tendermint_block(&self, block: TendermintBlock, vote: bool) -> Result<(), String> {
        // Placeholder logic for voting on a block using Tendermint
        Ok(())
    }

    pub async fn finalize_tendermint_block(&self, block: TendermintBlock) -> Result<(), String> {
        // Placeholder logic for finalizing a block using Tendermint
        Ok(())
    }
}
