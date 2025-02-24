use super::vm::IcnVM;
use crate::consensus::ConsensusMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct GovernanceExecutor {
    vm: Arc<IcnVM>,
    consensus_tx: mpsc::Sender<ConsensusMessage>,
}

impl GovernanceExecutor {
    pub fn new(vm: Arc<IcnVM>, consensus_tx: mpsc::Sender<ConsensusMessage>) -> Self {
        Self { vm, consensus_tx }
    }

    pub async fn execute_proposal(&self, proposal_id: &str, rules: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ast = super::parser::parse(rules)?;
        
        // Execute in VM and get state changes
        let state_changes = self.vm.execute_with_state_diff(ast)?;
        
        // Prepare consensus message
        let consensus_msg = ConsensusMessage::ProposalExecution {
            id: proposal_id.to_string(),
            changes: state_changes,
            timestamp: chrono::Utc::now(),
        };

        // Submit to consensus layer
        self.consensus_tx.send(consensus_msg).await?;
        Ok(())
    }

    pub async fn validate_transaction(&self, tx: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Execute validation rules
        let validation_result = self.vm.execute_validation(tx)?;
        Ok(validation_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_governance_execution() {
        // Test governance rule execution
        let rules = r#"
            governance: {
                voting_method: "quadratic",
                threshold: 75,
                min_participants: 10
            }
        "#;

        let (consensus_tx, _consensus_rx) = mpsc::channel(32);
        let vm = Arc::new(IcnVM::new());
        let executor = GovernanceExecutor::new(vm, consensus_tx);

        let result = executor.execute_proposal("test-proposal", rules).await;
        assert!(result.is_ok());
    }
}
