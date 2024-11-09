// examples/blockchain_test.rs

use std::sync::{Arc, Mutex};
use tokio;
use icn_backend::{
    blockchain::{Block, Blockchain, Transaction, TransactionType},
    identity::IdentitySystem,
    reputation::ReputationSystem,
    consensus::{ProofOfCooperation, types::ConsensusConfig},
    websocket::WebSocketHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting blockchain test...");

    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let ws_handler = Arc::new(WebSocketHandler::new());
    
    // Initialize consensus
    let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
        ConsensusConfig::default(),
        ws_handler.clone(),
    )));

    // Initialize blockchain
    let mut blockchain = Blockchain::new(
        identity_system.clone(),
        reputation_system.clone(),
    );

    // Register test identities
    {
        let mut identity = identity_system.lock().unwrap();
        identity.register_did(
            "did:icn:alice".to_string(), 
            vec!["transaction.create".to_string()]
        );
        identity.register_did(
            "did:icn:bob".to_string(),
            vec!["transaction.create".to_string()]
        );
    }

    // Set initial reputation
    {
        let mut reputation = reputation_system.lock().unwrap();
        reputation.increase_reputation("did:icn:alice", 100);
        reputation.increase_reputation("did:icn:bob", 100);
    }

    // Create and process test transactions
    let transactions = vec![
        Transaction::new(
            "did:icn:alice".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:bob".to_string(),
                amount: 50,
            },
        ),
        Transaction::new(
            "did:icn:bob".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:alice".to_string(),
                amount: 30,
            },
        ),
    ];

    // Process transactions
    for tx in transactions {
        println!("Processing transaction: {:?}", tx);
        blockchain.add_transaction(tx).await?;
    }

    // Force block finalization
    println!("Finalizing block...");
    blockchain.finalize_block().await?;

    // Print blockchain state
    println!("\nFinal blockchain state:");
    println!("Total blocks: {}", blockchain.get_block_count());
    println!("Total transactions: {}", blockchain.get_transaction_count());
    
    if let Some(latest_block) = blockchain.chain.last() {
        println!("Latest block hash: {}", latest_block.hash);
        println!("Latest block transactions: {}", latest_block.transactions.len());
    }

    // Test reputation effects
    {
        let reputation = reputation_system.lock().unwrap();
        println!("\nFinal reputation scores:");
        println!("Alice: {}", reputation.get_reputation("did:icn:alice"));
        println!("Bob: {}", reputation.get_reputation("did:icn:bob"));
    }

    Ok(())
}