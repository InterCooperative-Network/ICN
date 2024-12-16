use icn_backend::{
    blockchain::{Blockchain, Block, Transaction, TransactionType},
    identity::{DID, Algorithm, DIDError, IdentitySystem},
    reputation::ReputationSystem,
    governance::{Proposal, ProposalType, ProposalHistory},
    vm::{VM, Contract},
    vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact},
    vm::opcode::OpCode,
};

use icn_consensus;
use icn_core;
use icn_crypto;
use icn_p2p;
use icn_runtime;
use icn_storage;

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
            DID::new("did:icn:test".to_string(), Algorithm::Secp256k1),
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
async fn test_block_finalization() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone()
    );

    let block = Block::new(
        1,
        "previous_hash".to_string(),
        vec![],
        "proposer".to_string()
    );
    
    let mut finalized_block = block.clone();
    assert!(finalized_block.finalize().await.is_ok());
    assert!(finalized_block.metadata.size > 0);
}

// Reputation System Tests

#[tokio::test]
async fn test_reputation_decay() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    
    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 100, "governance".to_string());
        reputation.apply_decay("did:icn:test", 0.1, "governance".to_string()); // 10% decay
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert_eq!(reputation.get_reputation("did:icn:test", "governance".to_string()), 90);
    }
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

// Cooperative VM Tests

#[tokio::test]
async fn test_cooperative_contract_execution() {
    let vm = VM::new();
    let metadata = CooperativeMetadata {
        resource_impact: ResourceImpact {
            cpu: 10,
            memory: 1024,
            bandwidth: 100,
        },
        // Add other metadata fields as needed
    };

    let contract = Contract::new(
        vec![OpCode::Push(1), OpCode::Push(2), OpCode::Add],
        metadata
    );

    let result = vm.execute(&contract);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

// DID Tests

#[tokio::test]
async fn test_did_creation_and_serialization() {
    let did = DID::new("did:icn:test".to_string(), Algorithm::Secp256k1);
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
            DID::new("did:icn:test".to_string(), Algorithm::Secp256k1),
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
    let did = DID::new("did:icn:test".to_string(), Algorithm::Secp256k1);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify"));
}

#[tokio::test]
async fn test_did_operations() {
    let did = DID::new(
        "did:icn:test".to_string(), 
        Algorithm::Secp256k1
    );
    
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify"));

    // Test serialization
    let serialized = serde_json::to_string(&did).unwrap();
    let deserialized: DID = serde_json::from_str(&serialized).unwrap();
    assert_eq!(did.id, deserialized.id);
}
