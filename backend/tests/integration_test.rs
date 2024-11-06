use icn_backend::{
    blockchain::{Blockchain, Transaction, TransactionType},
    identity::{DID, IdentitySystem},
    reputation::ReputationSystem,
    governance::{Proposal, ProposalType, ProposalHistory},
    vm::{VM, Contract},
    vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact},
    vm::opcode::OpCode,
};

use secp256k1::SecretKey; // Import SecretKey for DID creation
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::thread_rng;

#[tokio::test]
async fn test_integration() {
    // Setup base systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    // Initialize blockchain with proper dependencies
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone()
    );

    // Test basic transaction creation and processing
    let transaction = Transaction::new(
        "did:icn:test".to_string(),
        TransactionType::Transfer {
            receiver: "did:icn:receiver".to_string(),
            amount: 100,
        },
    );

    // Register test identity with a generated SecretKey
    {
        let mut identity = identity_system.lock().unwrap();
        identity.register_did(
            DID::new("did:icn:test".to_string(), &SecretKey::new(&mut thread_rng())),
            vec!["transfer".to_string()],
        );
    }

    // Set initial reputation
    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.increase_reputation("did:icn:test", 100);
    }

    // Verify that the transaction was added and processed correctly
    assert!(blockchain.add_transaction(transaction).await.is_ok());
    assert_eq!(blockchain.pending_transactions.len(), 1);
}
