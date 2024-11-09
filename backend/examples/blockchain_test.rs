// examples/blockchain_test.rs

use std::sync::{Arc, Mutex};
use tokio;

use icn_backend::{
    blockchain::{Blockchain, Transaction, TransactionType},
    identity::{DID, IdentitySystem},
    reputation::ReputationSystem,
    consensus::{ProofOfCooperation, types::ConsensusConfig},
    websocket::WebSocketHandler,
};

use secp256k1::Secp256k1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting blockchain integration test...");

    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let ws_handler = Arc::new(WebSocketHandler::new());

    let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
        ConsensusConfig::default(),
        ws_handler.clone(),
    )));

    // Register test DIDs
    {
        let mut identity = identity_system.lock().unwrap();
        let secp = Secp256k1::new();

        // Create validator DID
        let (validator_did, _) = DID::generate_random("did:icn:validator1".to_string());
        identity.register_did(validator_did, vec![
            "validator.propose".to_string(),
            "validator.vote".to_string()
        ]);

        // Create test user DID
        let (user_did, _) = DID::generate_random("did:icn:user1".to_string());
        identity.register_did(user_did, vec![
            "transaction.create".to_string()
        ]);
    }

    // Initialize reputation scores
    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.increase_reputation("did:icn:validator1", 100);
        reputation.increase_reputation("did:icn:user1", 50);
    }

    // Create and initialize blockchain
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone(),
        consensus.clone(),
    );

    // Test transaction processing
    println!("\nTesting transaction processing...");
    
    let tx = Transaction::new(
        "did:icn:user1".to_string(),
        TransactionType::Transfer {
            receiver: "did:icn:recipient".to_string(),
            amount: 100,
        },
    );

    match blockchain.process_transaction(&tx).await {
        Ok(_) => println!("Transaction processed successfully"),
        Err(e) => eprintln!("Transaction processing failed: {}", e),
    }

    // Test block finalization
    println!("\nTesting block finalization...");
    match blockchain.finalize_block().await {
        Ok(_) => println!("Block finalized successfully"),
        Err(e) => eprintln!("Block finalization failed: {}", e),
    }

    // Print blockchain state
    println!("\nCurrent blockchain state:");
    println!("Block count: {}", blockchain.get_block_count());
    println!("Latest block height: {}", blockchain.current_block_number);
    if let Some(latest) = blockchain.chain.last() {
        println!("Latest block hash: {}", latest.hash);
        println!("Transaction count: {}", latest.transactions.len());
    }

    // Print reputation scores
    {
        let reputation = reputation_system.lock().unwrap();
        println!("\nFinal reputation scores:");
        println!("Validator1: {}", reputation.get_reputation("did:icn:validator1"));
        println!("User1: {}", reputation.get_reputation("did:icn:user1"));
    }

    Ok(())
}