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

// Additional Test Cases

#[tokio::test]
async fn test_blockchain_consistency() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone()
    );

    let block1 = Block::new(
        1,
        "previous_hash".to_string(),
        vec![],
        "proposer".to_string()
    );

    let block2 = Block::new(
        2,
        block1.hash.clone(),
        vec![],
        "proposer".to_string()
    );

    blockchain.add_block(block1).await.unwrap();
    blockchain.add_block(block2).await.unwrap();

    assert!(blockchain.verify_chain().await.is_ok());
}

#[tokio::test]
async fn test_vm_execution_with_complex_contract() {
    let vm = VM::new();
    let metadata = CooperativeMetadata {
        resource_impact: ResourceImpact {
            cpu: 20,
            memory: 2048,
            bandwidth: 200,
        },
        // Add other metadata fields as needed
    };

    let contract = Contract::new(
        vec![
            OpCode::Push(10),
            OpCode::Push(20),
            OpCode::Add,
            OpCode::Push(5),
            OpCode::Mul,
        ],
        metadata
    );

    let result = vm.execute(&contract);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 150);
}

#[tokio::test]
async fn test_governance_proposal_creation_and_voting() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let mut proposal_history = ProposalHistory::new();

    let proposal = Proposal::new(
        "did:icn:proposer".to_string(),
        ProposalType::ResourceAllocation {
            resource: "cpu".to_string(),
            amount: 100,
        },
    );

    proposal_history.add_proposal(proposal.clone());

    {
        let mut identity = identity_system.lock().unwrap();
        identity.register_did(
            DID::new("did:icn:voter".to_string(), Algorithm::Secp256k1),
            vec!["vote".to_string()],
        );
    }

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.increase_reputation("did:icn:voter", 50);
    }

    proposal_history.vote("did:icn:voter".to_string(), proposal.id.clone(), true);

    assert_eq!(proposal_history.get_proposal(proposal.id.clone()).unwrap().votes_for, 1);
}

// Tests for enhanced reputation management

#[tokio::test]
async fn test_dynamic_reputation_adjustment() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.dynamic_adjustment("did:icn:test", 50);
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert_eq!(reputation.get_reputation("did:icn:test", "consensus".to_string()), 50);
    }
}

#[tokio::test]
async fn test_reputation_decay_mechanism() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 100, "consensus".to_string());
        reputation.apply_decay("did:icn:test", 0.1);
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert_eq!(reputation.get_reputation("did:icn:test", "consensus".to_string()), 90);
    }
}

#[tokio::test]
async fn test_reputation_based_access_control() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 50, "consensus".to_string());
    }

    {
        let reputation = reputation_system.lock().unwrap();
        assert!(reputation.reputation_based_access("did:icn:test", 30));
        assert!(!reputation.reputation_based_access("did:icn:test", 60));
    }
}

// Tests for post-quantum algorithms integration

#[tokio::test]
async fn test_post_quantum_algorithms_integration() {
    let did_kyber = DID::new("did:icn:kyber".to_string(), Algorithm::Kyber);
    let did_dilithium = DID::new("did:icn:dilithium".to_string(), Algorithm::Dilithium);
    let did_falcon = DID::new("did:icn:falcon".to_string(), Algorithm::Falcon);

    let message = b"test message";

    // Test Kyber
    let signature_kyber = did_kyber.sign_message(message).expect("Failed to sign message with Kyber");
    assert!(did_kyber.verify_signature(message, &signature_kyber).expect("Failed to verify Kyber signature"));

    // Test Dilithium
    let signature_dilithium = did_dilithium.sign_message(message).expect("Failed to sign message with Dilithium");
    assert!(did_dilithium.verify_signature(message, &signature_dilithium).expect("Failed to verify Dilithium signature"));

    // Test Falcon
    let signature_falcon = did_falcon.sign_message(message).expect("Failed to sign message with Falcon");
    assert!(did_falcon.verify_signature(message, &signature_falcon).expect("Failed to verify Falcon signature"));
}
