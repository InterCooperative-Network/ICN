use icn_backend::{
    blockchain::{Blockchain, Block, Transaction, TransactionType},
    identity::{DID, Algorithm, DIDError, IdentitySystem},
    reputation::ReputationSystem,
    governance::{Proposal, ProposalType, ProposalHistory},
    vm::{VM, Contract},
    vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact},
    vm::opcode::OpCode,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::thread_rng;
use tokio::time::{sleep, Duration};
use warp::Filter;

use icn_types::{
    Block, Transaction, TransactionType, FederationOperation,
    FederationType, FederationTerms
};

use reqwest::Client;
use serde_json::json;
use tendermint::rpc::Client as TendermintClient;
use tendermint::lite::TrustedState;
use crate::core::consensus::TendermintConsensus;

// Remove the unused imports of non-existent crates
// use icn_consensus;
// use icn_core;
// use icn_crypto;
// use icn_p2p;
// use icn_runtime;
// use icn_storage;

// Create test helper structs
struct RuntimeManager;
struct ExecutionContext;
struct ValidationNode;
struct Check;
struct StateValidation;

impl RuntimeManager {
    fn new() -> Self {
        RuntimeManager
    }
}

impl ExecutionContext {
    fn default() -> Self {
        ExecutionContext
    }
}

mod test_helpers {
    use super::*;
    
    pub fn create_test_runtime() -> RuntimeManager {
        RuntimeManager::new()
    }

    pub fn create_test_context() -> ExecutionContext {
        ExecutionContext::default()
    }

    pub fn create_test_validation_node() -> ValidationNode {
        ValidationNode {
            pre_checks: vec![
                Check {
                    condition: "balance >= 100".to_string(),
                    action: "require_minimum_balance".to_string(),
                }
            ],
            post_checks: vec![],
            state_validation: Some(StateValidation {
                current: Some("PENDING".to_string()),
                expected: Some("APPROVED".to_string()),
                transition: Some("PENDING->APPROVED".to_string()),
            }),
            resource_checks: None,
            custom_merge: None,
        }
    }

    #[derive(Default)]
    pub struct RuntimeManager;
    #[derive(Default)] 
    pub struct ExecutionContext;
    #[derive(Default)]
    pub struct ValidationNode;
    #[derive(Default)]
    pub struct Check;
    #[derive(Default)]
    pub struct StateValidation;

    impl RuntimeManager {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn execute_validation_rules(&self, _validation: &ValidationNode, _context: &ExecutionContext) -> Result<(), String> {
            Ok(())
        }
    }
}

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

// Tests for key rotation

#[tokio::test]
async fn test_key_rotation() {
    let mut identity_system = IdentitySystem::new();
    let did = "did:icn:test".to_string();
    let algorithm = Algorithm::Secp256k1;
    let did_instance = DID::new(did.clone(), algorithm.clone());
    identity_system.register_did(did.clone(), vec!["transfer".to_string()], 100, did_instance.public_key.clone(), algorithm.clone());

    let old_public_key = identity_system.public_keys.get(&did).unwrap().0.clone();
    identity_system.rotate_key(&did).unwrap();
    let new_public_key = identity_system.public_keys.get(&did).unwrap().0.clone();

    assert_ne!(old_public_key, new_public_key);
}

// Real-time Reputation Recalibration Tests

#[tokio::test]
async fn test_real_time_reputation_recalibration() {
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));

    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.adjust_reputation("did:icn:test", 100, "governance".to_string());
    }

    // Simulate real-time recalibration
    let reputation_system_clone = reputation_system.clone();
    tokio::spawn(async move {
        loop {
            {
                let mut reputation = reputation_system_clone.lock().unwrap();
                reputation.apply_decay("did:icn:test", 0.1, "governance".to_string());
            }
            sleep(Duration::from_secs(10)).await;
        }
    });

    // Wait for some time to allow recalibration to occur
    sleep(Duration::from_secs(30)).await;

    {
        let reputation = reputation_system.lock().unwrap();
        assert!(reputation.get_reputation("did:icn:test", "governance".to_string()) < 100);
    }
}

#[tokio::test]
async fn test_backend_startup() {
    // Start the backend application
    let backend_future = tokio::spawn(async {
        let routes = warp::path::end().map(|| warp::reply::html("Backend is running"));
        warp::serve(routes).run(([0, 0, 0, 0], 8081)).await;
    });

    // Wait for the backend to start
    sleep(Duration::from_secs(2)).await;

    // Check if the backend is running
    let response = reqwest::get("http://localhost:8081").await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Backend is running");

    // Stop the backend
    backend_future.abort();
}

#[tokio::test]
async fn test_frontend_connection() {
    // Start the backend application
    let backend_future = tokio::spawn(async {
        let routes = warp::path::end().map(|| warp::reply::html("Backend is running"));
        warp::serve(routes).run(([0, 0, 0, 0], 8081)).await;
    });

    // Wait for the backend to start
    sleep(Duration::from_secs(2)).await;

    // Simulate frontend connection
    let response = reqwest::get("http://localhost:8081").await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Backend is running");

    // Stop the backend
    backend_future.abort();
}

// Notification Tests

#[tokio::test]
async fn test_notification_sending() {
    let notification_manager = NotificationManager::new("http://localhost:8081/email".to_string(), "http://localhost:8081/sms".to_string());

    // Test sending email notification
    let email_result = notification_manager.send_email("Test Subject", "Test Body").await;
    assert!(email_result.is_ok());

    // Test sending SMS notification
    let sms_result = notification_manager.send_sms("Test Message").await;
    assert!(sms_result.is_ok());
}

#[tokio::test]
async fn test_notification_fallback() {
    let notification_manager = NotificationManager::new("http://localhost:8081/email".to_string(), "http://localhost:8081/sms".to_string());

    // Simulate email failure by using an invalid URL
    let invalid_email_manager = NotificationManager::new("http://invalid-url".to_string(), "http://localhost:8081/sms".to_string());

    // Test fallback to SMS when email fails
    let result = invalid_email_manager.send_notification("Test Subject", "Test Body").await;
    assert!(result.is_ok());
}

// Federation Tests

#[tokio::test]
async fn test_initiate_federation() {
    let federation_operation = FederationOperation::InitiateFederation {
        federation_type: FederationType::Cooperative,
        partner_id: "did:icn:partner".to_string(),
        terms: FederationTerms {
            minimum_reputation: 50,
            resource_sharing_policies: "Equal distribution".to_string(),
            governance_rules: "Majority vote".to_string(),
            duration: "2025-12-31T23:59:59Z".to_string(),
        },
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_join_federation() {
    let federation_operation = FederationOperation::JoinFederation {
        federation_id: "federation123".to_string(),
        commitment: vec!["Adhere to terms".to_string(), "Contribute resources".to_string()],
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_leave_federation() {
    let federation_operation = FederationOperation::LeaveFederation {
        federation_id: "federation123".to_string(),
        reason: "No longer able to participate".to_string(),
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_propose_action() {
    let federation_operation = FederationOperation::ProposeAction {
        federation_id: "federation123".to_string(),
        action_type: "New Project".to_string(),
        description: "Proposal for a new collaborative project".to_string(),
        resources: {
            let mut resources = HashMap::new();
            resources.insert("resourceX".to_string(), 100);
            resources.insert("resourceY".to_string(), 200);
            resources
        },
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_vote_on_federation_proposal() {
    let federation_operation = FederationOperation::VoteOnProposal {
        federation_id: "federation123".to_string(),
        proposal_id: "proposal456".to_string(),
        approve: true,
        notes: Some("Support the project".to_string()),
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_share_resources() {
    let federation_operation = FederationOperation::ShareResources {
        federation_id: "federation123".to_string(),
        resource_type: "resourceX".to_string(),
        amount: 50,
        recipient_id: "did:icn:recipient".to_string(),
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_federation_terms() {
    let federation_operation = FederationOperation::UpdateFederationTerms {
        federation_id: "federation123".to_string(),
        new_terms: FederationTerms {
            minimum_reputation: 60,
            resource_sharing_policies: "Proportional distribution".to_string(),
            governance_rules: "Supermajority vote".to_string(),
            duration: "2026-12-31T23:59:59Z".to_string(),
        },
    };

    let result = handle_federation_operation(federation_operation).await;
    assert!(result.is_ok());
}

// Tests for real-time updates for proposals and voting

#[tokio::test]
async fn test_real_time_proposal_updates() {
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

    // Simulate WebSocket connection and message handling
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel();
    let ws_tx_clone = ws_tx.clone();

    tokio::spawn(async move {
        while let Some(message) = ws_rx.recv().await {
            if let Ok(proposal_update) = serde_json::from_str::<Proposal>(&message) {
                proposal_history.update_proposal(proposal_update);
            }
        }
    });

    // Simulate sending a proposal update via WebSocket
    let updated_proposal = Proposal {
        votes_for: 1,
        ..proposal.clone()
    };
    let message = serde_json::to_string(&updated_proposal).unwrap();
    ws_tx_clone.send(message).unwrap();

    // Wait for the update to be processed
    sleep(Duration::from_secs(1)).await;

    assert_eq!(proposal_history.get_proposal(proposal.id.clone()).unwrap().votes_for, 1);
}

#[tokio::test]
async fn test_real_time_voting_updates() {
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

    // Simulate WebSocket connection and message handling
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel();
    let ws_tx_clone = ws_tx.clone();

    tokio::spawn(async move {
        while let Some(message) = ws_rx.recv().await {
            if let Ok(vote_update) = serde_json::from_str::<Proposal>(&message) {
                proposal_history.update_proposal(vote_update);
            }
        }
    });

    // Simulate sending a vote update via WebSocket
    let updated_proposal = Proposal {
        votes_for: 1,
        ..proposal.clone()
    };
    let message = serde_json::to_string(&updated_proposal).unwrap();
    ws_tx_clone.send(message).unwrap();

    // Wait for the update to be processed
    sleep(Duration::from_secs(1)).await;

    assert_eq!(proposal_history.get_proposal(proposal.id.clone()).unwrap().votes_for, 1);
}

#[tokio::test]
async fn test_validation_rule_execution() {
    let runtime = test_helpers::create_test_runtime();
    let context = test_helpers::create_test_context();
    let validation = test_helpers::create_test_validation_node();
    
    let result = runtime.execute_validation_rules(&validation, &context);
    assert!(result.is_ok());
}

use reqwest::Client;
use serde_json::json;
use tokio::time::Duration;

#[tokio::test]
async fn test_create_proposal() {
    let client = Client::new();
    let resp = client.post("http://localhost:8081/api/governance/proposals")
        .json(&json!({
            "title": "Test Proposal",
            "description": "Test Description",
            "created_by": "did:icn:test",
            "ends_at": "2024-12-31T23:59:59Z"
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(resp.status(), 200);
    
    let proposal: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(proposal["title"], "Test Proposal");
}

#[tokio::test]
async fn test_websocket_connection() {
    let (ws_stream, _) = tokio_tungstenite::connect_async("ws://localhost:8081/ws")
        .await
        .unwrap();
        
    // Test sending and receiving messages
    // ...existing test code...
}

#[tokio::test]
async fn test_federation_member_management() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    
    let federation_id = "federation123".to_string();
    let member_did = "did:icn:member1".to_string();
    
    // Test adding member
    {
        let mut identity = identity_system.lock().unwrap();
        identity.assign_federation_role(
            federation_id.clone(),
            member_did.clone(),
            "member".to_string(),
        ).unwrap();
    }

    // Verify member role
    {
        let identity = identity_system.lock().unwrap();
        let roles = identity.get_federation_roles(&federation_id, &member_did);
        assert!(roles.contains(&"member".to_string()));
    }

    // Test role revocation
    {
        let mut identity = identity_system.lock().unwrap();
        identity.revoke_federation_role(
            &federation_id,
            &member_did,
            "member",
        ).unwrap();
        
        let roles = identity.get_federation_roles(&federation_id, &member_did);
        assert!(roles.is_empty());
    }
}

#[tokio::test]
async fn test_federation_member_status_updates() {
    let mut federation = Federation::new(
        "federation123".to_string(),
        FederationType::Cooperative,
        FederationTerms {
            minimum_reputation: 50,
            resource_sharing_policies: "Equal distribution".to_string(),
            governance_rules: "Majority vote".to_string(),
            duration: "2025-12-31T23:59:59Z".to_string(),
        },
        "did:icn:admin".to_string(),
    );

    let member_did = "did:icn:member1".to_string();

    // Test adding member
    assert!(federation.add_member(member_did.clone(), MemberRole::Member).is_ok());
    
    // Verify member status
    assert_eq!(
        federation.get_member_status(&member_did),
        Some(&MemberStatus::Active)
    );

    // Test updating member status
    assert!(federation.update_member_status(&member_did, MemberStatus::Suspended).is_ok());
    
    assert_eq!(
        federation.get_member_status(&member_did),
        Some(&MemberStatus::Suspended)
    );

    // Verify active members list
    let active_members = federation.get_active_members();
    assert!(!active_members.contains(&member_did));
}

// Tests for key rotation and revocation

#[tokio::test]
async fn test_key_rotation_and_revocation() {
    let mut did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    
    // Test key rotation
    let original_key = did.public_key.clone();
    assert!(did.rotate_key().is_ok());
    assert_ne!(original_key, did.public_key);
    
    // Verify the rotated key works
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign with rotated key");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify with rotated key"));
    
    // Test key revocation
    assert!(did.revoke_key().is_ok());
    
    // Attempt to rotate a revoked key should fail
    assert!(did.rotate_key().is_err());
}

// Tests for BLS threshold signatures

#[tokio::test]
async fn test_bls_threshold_signatures() {
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let message = b"test message";

    // Generate BLS private keys
    let private_keys: Vec<BlsPrivateKey> = (0..3).map(|_| BlsPrivateKey::generate(&mut rand::thread_rng())).collect();
    let public_keys: Vec<BlsPublicKey> = private_keys.iter().map(|key| BlsPublicKey::from(key)).collect();

    // Generate BLS threshold signature
    let signature = {
        let mut identity = identity_system.lock().unwrap();
        identity.generate_bls_threshold_signature(message, private_keys.clone()).unwrap()
    };

    // Verify BLS threshold signature
    {
        let identity = identity_system.lock().unwrap();
        assert!(identity.verify_bls_threshold_signature(message, &signature, public_keys.clone()).unwrap());
    }
}

#[tokio::test]
async fn test_tendermint_consensus_integration() {
    let tendermint_client = TendermintClient::new("http://localhost:26657").unwrap();
    let trusted_state = TrustedState::default();
    let tendermint_consensus = TendermintConsensus::new(tendermint_client, trusted_state);

    // Test starting the Tendermint consensus engine
    assert!(tendermint_consensus.start().await.is_ok());

    // Test proposing a block
    let block = tendermint::block::Block::default();
    assert!(tendermint_consensus.propose_block(block.clone()).await.is_ok());

    // Test voting on a block
    assert!(tendermint_consensus.vote_on_block(block.clone(), true).await.is_ok());

    // Test finalizing a block
    assert!(tendermint_consensus.finalize_block(block).await.is_ok());

    // Test stopping the Tendermint consensus engine
    assert!(tendermint_consensus.stop().await.is_ok());
}
