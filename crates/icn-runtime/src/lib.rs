use async_trait::async_trait;
use icn_types::{Block, Transaction};

pub struct RuntimeManager {
    // Fields for the RuntimeManager struct
}

impl RuntimeManager {
    // Methods for the RuntimeManager struct
}

#[async_trait]
pub trait VM {
    async fn execute_transaction(&self, transaction: Transaction);
    async fn execute_block(&self, block: Block);
}

pub struct ContractExecution {
    // Fields for the ContractExecution struct
}

impl ContractExecution {
    // Methods for the ContractExecution struct
}
