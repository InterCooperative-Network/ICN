#[cfg(test)]
mod tests {
    use super::*;
    use icn_result::blockchain::{Blockchain, Transaction, TransactionType};
    use icn_result::identity::IdentitySystem;
    use icn_result::reputation::ReputationSystem;

    fn setup_test_blockchain() -> Blockchain {
        let identity_system = IdentitySystem::new();
        let reputation_system = ReputationSystem::new();
        let mut blockchain = Blockchain::new(identity_system, reputation_system);
        
        // Register test validators
        blockchain.consensus.register_validator("did:icn:1".to_string(), 100).unwrap();
        blockchain.consensus.register_validator("did:icn:2".to_string(), 100).unwrap();
        blockchain.consensus.register_validator("did:icn:3".to_string(), 100).unwrap();
        
        blockchain
    }

    #[test]
    fn test_consensus_integration() {
        let mut blockchain = setup_test_blockchain();
        
        // Add some test transactions
        let transaction = Transaction::new(
            "did:icn:1".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:2".to_string(),
                amount: 100,
            },
        );
        blockchain.add_transaction(transaction);

        // Try to finalize block through consensus
        assert!(blockchain.finalize_block().is_ok());
        
        // Verify block was added
        assert_eq!(blockchain.chain.len(), 2);
        
        // Verify reputation updates were applied
        let reputation_updates = blockchain.consensus.get_reputation_updates();
        assert!(!reputation_updates.is_empty());
    }

    #[test]
    fn test_consensus_failure_recovery() {
        let mut blockchain = setup_test_blockchain();
        
        // Simulate a failed consensus round
        blockchain.consensus.start_round().unwrap();
        blockchain.consensus.check_timeout();
        
        // Verify we can start a new round after failure
        assert!(blockchain.finalize_block().is_ok());
    }
}
