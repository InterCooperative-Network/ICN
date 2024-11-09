// examples/blockchain_test.rs

use std::sync::{Arc, Mutex};
use tokio;
use icn_backend::{
    blockchain::{Block, Blockchain, Transaction, TransactionType},
    consensus::{ProofOfCooperation, types::ConsensusConfig},
    identity::IdentitySystem,
    reputation::ReputationSystem,
    websocket::WebSocketHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting blockchain test client...");

    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let ws_handler = Arc::new(WebSocketHandler::new());

    // Initialize consensus
    let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
        ConsensusConfig::default(),
        ws_handler.clone(),
    )));

    // Create test DIDs
    {
        let mut identity = identity_system.lock().unwrap();
        identity.register_did(
            "did:icn:test1".to_string(),
            vec!["transaction.create".to_string()]
        );
        identity.register_did(
            "did:icn:test2".to_string(),
            vec!["transaction.create".to_string()]
        );
        println!("✓ Created test DIDs");
    }

    // Set initial reputation
    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.increase_reputation("did:icn:test1", 100);
        reputation.increase_reputation("did:icn:test2", 100);
        println!("✓ Set initial reputation scores");
    }

    // Initialize blockchain
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone(),
        consensus.clone(),
    );
    println!("✓ Initialized blockchain");

    // Create and add test transactions
    let transactions = vec![
        Transaction::new(
            "did:icn:test1".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:test2".to_string(),
                amount: 50,
            },
        ),
        Transaction::new(
            "did:icn:test2".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:test1".to_string(),
                amount: 30,
            },
        ),
    ];

    // Process transactions
    println!("\nProcessing test transactions...");
    for (i, tx) in transactions.iter().enumerate() {
        match blockchain.add_transaction(tx.clone()).await {
            Ok(_) => println!("✓ Transaction {} added successfully", i + 1),
            Err(e) => println!("✗ Transaction {} failed: {}", i + 1, e),
        }
    }

    // Force block creation
    println!("\nFinalizing block...");
    match blockchain.finalize_block().await {
        Ok(_) => println!("✓ Block finalized successfully"),
        Err(e) => println!("✗ Block finalization failed: {}", e),
    }

    // Print blockchain state
    println!("\nFinal blockchain state:");
    println!("Chain length: {}", blockchain.get_block_count());
    println!("Total transactions: {}", blockchain.get_transaction_count());

    if let Some(latest) = blockchain.chain.last() {
        println!("Latest block: #{}", latest.index);
        println!("Block hash: {}", latest.hash);
        println!("Transactions in block: {}", latest.transactions.len());
    }

    // Check consensus state
    if let Some(round) = blockchain.get_current_round() {
        println!("\nConsensus state:");
        println!("Current round: {}", round.round_number);
        println!("Round status: {:?}", round.status);
        println!("Participation rate: {:.1}%", round.stats.participation_rate * 100.0);
    }

    // Check final reputation scores
    {
        let reputation = reputation_system.lock().unwrap();
        println!("\nFinal reputation scores:");
        println!("test1: {}", reputation.get_reputation("did:icn:test1"));
        println!("test2: {}", reputation.get_reputation("did:icn:test2"));
    }

    println!("\nTest completed successfully!");
    Ok(())
}