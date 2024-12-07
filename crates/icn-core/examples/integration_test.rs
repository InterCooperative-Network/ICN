// File: ./backend/examples/integration_test.rs

use std::sync::{Arc, Mutex};
use tokio;
use std::time::Duration;
use chrono::Utc;

use icn_backend::{
    blockchain::{Block, Blockchain, Transaction, TransactionType},
    consensus::{ProofOfCooperation, types::ConsensusConfig},
    identity::{DID, IdentitySystem},
    reputation::ReputationSystem,
    websocket::WebSocketHandler,
    governance::{Proposal, ProposalType, ProposalStatus},
    vm::{VM, Contract, ExecutionContext, cooperative_metadata::CooperativeMetadata},
};

/// Tests DID creation and verification
async fn test_identity_system(identity_system: &Arc<Mutex<IdentitySystem>>) -> Result<(), String> {
    println!("\n=== Testing Identity System ===");
    
    // Generate test DIDs
    let mut identity = identity_system.lock().unwrap();
    
    // Create DIDs with different permission sets
    identity.register_did(
        "did:icn:validator1".to_string(), 
        vec!["transaction.create".to_string(), "proposal.vote".to_string()]
    );
    println!("✓ Registered validator DID");

    identity.register_did(
        "did:icn:member1".to_string(),
        vec!["transaction.create".to_string()]
    );
    println!("✓ Registered member DID");

    // Verify permissions
    let validator_perms = identity.get_permissions("did:icn:validator1");
    assert!(validator_perms.contains(&"proposal.vote".to_string()));
    println!("✓ Validator permissions verified");

    let member_perms = identity.get_permissions("did:icn:member1");
    assert!(!member_perms.contains(&"proposal.vote".to_string()));
    println!("✓ Member permissions verified");

    Ok(())
}

/// Tests reputation score updates and verification
async fn test_reputation_system(reputation_system: &Arc<Mutex<ReputationSystem>>) -> Result<(), String> {
    println!("\n=== Testing Reputation System ===");

    let mut reputation = reputation_system.lock().unwrap();

    // Test reputation increases
    reputation.increase_reputation("did:icn:validator1", 100);
    assert_eq!(reputation.get_reputation("did:icn:validator1"), 100);
    println!("✓ Reputation increase verified");

    // Test reputation decreases
    reputation.decrease_reputation("did:icn:validator1", 20);
    assert_eq!(reputation.get_reputation("did:icn:validator1"), 80);
    println!("✓ Reputation decrease verified");

    // Test reputation floor (shouldn't go below 0)
    reputation.decrease_reputation("did:icn:member1", 50);
    assert_eq!(reputation.get_reputation("did:icn:member1"), 0);
    println!("✓ Reputation floor verified");

    Ok(())
}

/// Tests blockchain transaction processing and consensus
async fn test_blockchain(
    blockchain: &mut Blockchain,
    identity_system: &Arc<Mutex<IdentitySystem>>,
    reputation_system: &Arc<Mutex<ReputationSystem>>
) -> Result<(), String> {
    println!("\n=== Testing Blockchain System ===");

    // Create test transaction
    let transaction = Transaction::new(
        "did:icn:validator1".to_string(),
        TransactionType::Transfer {
            receiver: "did:icn:member1".to_string(),
            amount: 50,
        },
    );

    // Add transaction to blockchain
    blockchain.add_transaction(transaction).await?;
    println!("✓ Transaction added to pending pool");

    // Verify transaction in pending pool
    assert!(!blockchain.pending_transactions.is_empty());
    println!("✓ Transaction found in pending pool");

    // Force block creation
    blockchain.finalize_block().await?;
    println!("✓ Block finalized");

    // Verify chain state
    assert_eq!(blockchain.get_block_count(), 2); // Genesis + 1 new block
    println!("✓ Chain state verified");

    Ok(())
}

/// Tests VM contract execution
async fn test_vm(
    identity_system: &Arc<Mutex<IdentitySystem>>,
    reputation_system: &Arc<Mutex<ReputationSystem>>
) -> Result<(), String> {
    println!("\n=== Testing Virtual Machine ===");

    // Get reputation context
    let reputation_context = {
        let reputation = reputation_system.lock().unwrap();
        reputation.get_reputation_context()
    };

    // Create VM instance
    let mut vm = VM::new(1000, reputation_context);
    println!("✓ VM initialized");

    // Create test contract
    let cooperative_metadata = CooperativeMetadata {
        creator_did: "did:icn:validator1".to_string(),
        cooperative_id: "coop1".to_string(),
        purpose: "Test cooperative".to_string(),
        resource_impact: Default::default(),
        federation_id: None,
        creation_timestamp: Utc::now().timestamp() as u64,
        last_updated: Utc::now().timestamp() as u64,
        member_count: 1,
        resource_allocation: Default::default(),
    };

    let contract = Contract {
        id: "contract1".to_string(),
        code: vec![],  // Add some test VM opcodes here
        state: Default::default(),
        required_reputation: 50,
        cooperative_metadata,
        version: "1.0.0".to_string(),
        dependencies: vec![],
        permissions: vec!["cooperative.create".to_string()],
    };

    // Create execution context
    let context = ExecutionContext {
        caller_did: "did:icn:validator1".to_string(),
        cooperative_id: "coop1".to_string(),
        timestamp: Utc::now().timestamp() as u64,
        block_number: 1,
        reputation_score: 80,
        permissions: vec!["cooperative.create".to_string()],
    };

    // Execute contract
    vm.set_execution_context(context);
    vm.execute_contract(&contract)?;
    println!("✓ Contract executed successfully");

    Ok(())
}

/// Tests governance proposal creation and voting
async fn test_governance(
    blockchain: &mut Blockchain,
    identity_system: &Arc<Mutex<IdentitySystem>>,
    reputation_system: &Arc<Mutex<ReputationSystem>>
) -> Result<(), String> {
    println!("\n=== Testing Governance System ===");

    // Create test proposal
    let proposal = Proposal::new(
        1,
        ProposalType::ResourceAllocation,
        "Test resource allocation".to_string(),
        50,  // Required reputation
        60,  // Duration in minutes
    );

    // Submit proposal as transaction
    let transaction = Transaction::new(
        "did:icn:validator1".to_string(),
        TransactionType::ContractExecution {
            contract_id: "governance".to_string(),
            input_data: Default::default(),
        },
    );

    blockchain.add_transaction(transaction).await?;
    println!("✓ Proposal submitted");

    // Finalize block with proposal
    blockchain.finalize_block().await?;
    println!("✓ Proposal finalized in block");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("Starting integration tests...");

    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let ws_handler = Arc::new(WebSocketHandler::new());

    // Initialize blockchain
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone(),
    );

    // Run test suites
    test_identity_system(&identity_system).await?;
    test_reputation_system(&reputation_system).await?;
    test_blockchain(&mut blockchain, &identity_system, &reputation_system).await?;
    test_vm(&identity_system, &reputation_system).await?;
    test_governance(&mut blockchain, &identity_system, &reputation_system).await?;

    println!("\nAll integration tests completed successfully!");
    Ok(())
}