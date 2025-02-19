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
}
