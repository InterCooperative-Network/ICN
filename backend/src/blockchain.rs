use icn_types::{Block, Transaction};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            pending_transactions: vec![],
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
}
