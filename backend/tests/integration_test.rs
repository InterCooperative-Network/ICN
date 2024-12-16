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

#[tokio::test]
async fn test_multi_dimensional_reputation_tracking() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 50, "governance".to_string());
        reputation.adjust_reputation("did:icn:test", 30, "resource_sharing".to_string());
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert_eq!(reputation.get_reputation("did:icn:test", "governance".to_string()), 50);
        assert_eq!(reputation.get_reputation("did:icn:test", "resource_sharing".to_string()), 30);
    }
}

#[tokio::test]
async fn test_category_specific_adjustments() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 20, "technical".to_string());
        reputation.adjust_reputation("did:icn:test", -10, "governance".to_string());
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert_eq!(reputation.get_reputation("did:icn:test", "technical".to_string()), 20);
        assert_eq!(reputation.get_reputation("did:icn:test", "governance".to_string()), -10);
    }
}

#[tokio::test]
async fn test_category_specific_eligibility_checks() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 40, "governance".to_string());
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert!(reputation.is_eligible("did:icn:test", 30, "governance".to_string()));
        assert!(!reputation.is_eligible("did:icn:test", 50, "governance".to_string()));
    }
}

#[tokio::test]
async fn test_did_creation_and_serialization() {
    let did = DID::new("did:icn:test".to_string(), &SecretKey::new(&mut thread_rng()));
    let serialized_did = serde_json::to_string(&did).unwrap();
    let deserialized_did: DID = serde_json::from_str(&serialized_did).unwrap();
    assert_eq!(did, deserialized_did);
}

#[tokio::test]
async fn test_permission_handling_in_identity_system() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));

    {
        let mut identity = identity_system.lock().unwrap();
        identity.register_did(
            DID::new("did:icn:test".to_string(), &SecretKey::new(&mut thread_rng())),
            vec!["transfer".to_string()],
        );
    }

    {
        let identity = identity_system.lock().unwrap();
        assert!(identity.has_permission("did:icn:test", "transfer"));
        assert!(!identity.has_permission("did:icn:test", "invalid_permission"));
    }
}

#[tokio::test]
async fn test_did_validation() {
    let did = DID::new("did:icn:test".to_string(), &SecretKey::new(&mut thread_rng()));
    let message = b"test message";
    let signature = did.sign_message(message);
    assert!(did.verify_signature(message, &signature));
}
