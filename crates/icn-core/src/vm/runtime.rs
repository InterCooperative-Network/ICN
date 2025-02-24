use async_trait::async_trait;
use icn_types::{Transaction, RuntimeError};

#[async_trait]
pub trait RuntimeInterface: Send + Sync {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn execute_transaction(&self, tx: Transaction) -> Result<(), RuntimeError>;
    async fn load_bytecode(&self, bytecode: &[u8]) -> Result<(), RuntimeError>;
}

pub struct RuntimeManager {
    vm_instance: super::VirtualMachine,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            vm_instance: super::VirtualMachine::new(1_000_000, 1024 * 1024),
        }
    }
}

#[async_trait]
impl RuntimeInterface for RuntimeManager {
    async fn start(&self) -> Result<(), String> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    async fn execute_transaction(&self, _tx: Transaction) -> Result<(), RuntimeError> {
        Ok(())
    }

    async fn load_bytecode(&self, _bytecode: &[u8]) -> Result<(), RuntimeError> {
        Ok(())
    }
}
