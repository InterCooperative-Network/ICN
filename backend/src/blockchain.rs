use icn_types::{Block, Transaction};
use sha2::{Sha256, Digest};
use tendermint::block::Block as TendermintBlock;
use tendermint::lite::TrustedState;
use tendermint::rpc::Client;
use tokio::sync::Mutex;
use std::sync::Arc;
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use std::collections::HashMap;
use log::{info, error};

pub trait BlockchainOperations {
    fn add_block(&mut self, block: Block);
    fn get_latest_block(&self) -> Option<&Block>;
    fn get_block_by_index(&self, index: usize) -> Option<&Block>;
    fn add_transaction(&mut self, transaction: Transaction);
    fn process_pending_transactions(&mut self) -> Result<(), String>;
    fn validate_transaction(&self, transaction: &Transaction) -> Result<bool, String>;
    fn calculate_hash(block: &Block) -> String;
    async fn start_consensus_round(&mut self, block: &mut Block) -> Result<(), String>;
    async fn vote_on_block(&mut self, block: &mut Block, validator_did: String, vote: bool) -> Result<(), String>;
    async fn finalize_block(&mut self, block: &mut Block) -> Result<(), String>;
    fn validate_contribution(&self, contribution: &Contribution) -> Result<bool, String>;
    async fn propose_block(&self, block: TendermintBlock) -> Result<(), String>;
    async fn vote_on_tendermint_block(&self, block: TendermintBlock, vote: bool) -> Result<(), String>;
    async fn finalize_tendermint_block(&self, block: TendermintBlock) -> Result<(), String>;
}

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub tendermint_client: Client,
    pub trusted_state: Arc<Mutex<TrustedState>>,
    pub cache: HashMap<String, Block>,
}

impl Blockchain {
    pub fn new(tendermint_client: Client, trusted_state: TrustedState) -> Self {
        Self {
            blocks: vec![],
            pending_transactions: vec![],
            tendermint_client,
            trusted_state: Arc::new(Mutex::new(trusted_state)),
            cache: HashMap::new(),
        }
    }
}

impl BlockchainOperations for Blockchain {
    fn add_block(&mut self, block: Block) {
        info!("Adding block with index: {}", block.index);
        self.cache.insert(block.index.to_string(), block.clone());
        self.blocks.push(block);
    }

    fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    fn get_block_by_index(&self, index: usize) -> Option<&Block> {
        if let Some(block) = self.cache.get(&index.to_string()) {
            return Some(block);
        }
        self.blocks.get(index)
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        info!("Adding transaction with ID: {}", transaction.id);
        self.pending_transactions.push(transaction);
    }

    fn process_pending_transactions(&mut self) -> Result<(), String> {
        info!("Processing pending transactions");
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

    fn validate_transaction(&self, transaction: &Transaction) -> Result<bool, String> {
        info!("Validating transaction with ID: {}", transaction.id);
        if let Some(proof) = &transaction.zk_snark_proof {
            if !verify_proof(proof) {
                return Err("Invalid zk-SNARK proof".to_string());
            }
        }
        Ok(true)
    }

    fn calculate_hash(block: &Block) -> String {
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

    async fn start_consensus_round(&mut self, block: &mut Block) -> Result<(), String> {
        info!("Starting consensus round for block with index: {}", block.index);
        block.start_consensus_round().await.map_err(|e| e.to_string())
    }

    async fn vote_on_block(&mut self, block: &mut Block, validator_did: String, vote: bool) -> Result<(), String> {
        info!("Voting on block with index: {} by validator: {}", block.index, validator_did);
        block.vote_on_block(validator_did, vote).await.map_err(|e| e.to_string())
    }

    async fn finalize_block(&mut self, block: &mut Block) -> Result<(), String> {
        info!("Finalizing block with index: {}", block.index);
        if block.metadata.validator_count >= 3 { // Assuming 3 is the required number of validators for consensus
            block.finalize().await.map_err(|e| e.to_string())?;
            self.add_block(block.clone());
            Ok(())
        } else {
            Err("Consensus not reached".to_string())
        }
    }

    fn validate_contribution(&self, contribution: &Contribution) -> Result<bool, String> {
        info!("Validating contribution");
        Ok(true)
    }

    async fn propose_block(&self, block: TendermintBlock) -> Result<(), String> {
        info!("Proposing block with height: {}", block.header.height);
        if let Some(proof) = &block.zk_snark_proof {
            if !verify_proof(proof) {
                return Err("Invalid zk-SNARK proof".to_string());
            }
        }
        Ok(())
    }

    async fn vote_on_tendermint_block(&self, block: TendermintBlock, vote: bool) -> Result<(), String> {
        info!("Voting on Tendermint block with height: {}", block.header.height);
        if let Some(proof) = &block.zk_snark_proof {
            if !verify_proof(proof) {
                return Err("Invalid zk-SNARK proof".to_string());
            }
        }
        Ok(())
    }

    async fn finalize_tendermint_block(&self, block: TendermintBlock) -> Result<(), String> {
        info!("Finalizing Tendermint block with height: {}", block.header.height);
        Ok(())
    }
}
