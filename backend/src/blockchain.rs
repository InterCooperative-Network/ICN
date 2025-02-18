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
    
    // Implementation using icn-types structures
}
