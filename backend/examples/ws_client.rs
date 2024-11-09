// File: ./backend/examples/ws_client.rs

use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
enum ClientMessage {
    RegisterValidator {
        did: String,
        initial_reputation: i64,
    },
    Subscribe {
        events: Vec<String>,
    },
    SubmitTransaction {
        transaction: TransactionData,
    },
    QueryStatus,
}

#[derive(Debug, Serialize)]
struct TransactionData {
    sender: String,
    receiver: String,
    amount: u64,
    timestamp: u64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    ConsensusUpdate {
        round_number: u64,
        status: String,
        coordinator: String,
        votes_count: usize,
        participation_rate: f64,
        remaining_time_ms: i64,
    },
    BlockFinalized {
        block_number: u64,
        transactions_count: usize,
        timestamp: u64,
        proposer: String,
    },
    ReputationUpdate {
        did: String,
        change: i64,
        new_total: i64,
        reason: String,
    },
    CommandResponse {
        command: String,
        status: String,
        message: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting WebSocket test client...");

    // Connect to WebSocket server
    let url = Url::parse("ws://localhost:8081/ws")?;
    println!("Connecting to {}", url);

    let (ws_stream, _) = connect_async(url).await?;
    println!("✓ WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Test 1: Register as a validator
    println!("\nTest 1: Registering as validator...");
    let register_msg = ClientMessage::RegisterValidator {
        did: "did:icn:test1".to_string(),
        initial_reputation: 100,
    };
    
    let msg = serde_json::to_string(&register_msg)?;
    write.send(Message::Text(msg)).await?;
    println!("✓ Sent validator registration");

    // Test 2: Subscribe to events
    println!("\nTest 2: Subscribing to events...");
    let subscribe_msg = ClientMessage::Subscribe {
        events: vec![
            "consensus".to_string(),
            "blocks".to_string(),
            "reputation".to_string(),
        ],
    };
    
    let msg = serde_json::to_string(&subscribe_msg)?;
    write.send(Message::Text(msg)).await?;
    println!("✓ Subscribed to events");

    // Test 3: Submit a test transaction
    println!("\nTest 3: Submitting test transaction...");
    let transaction = TransactionData {
        sender: "did:icn:test1".to_string(),
        receiver: "did:icn:test2".to_string(),
        amount: 50,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };

    let tx_msg = ClientMessage::SubmitTransaction {
        transaction,
    };
    
    let msg = serde_json::to_string(&tx_msg)?;
    write.send(Message::Text(msg)).await?;
    println!("✓ Sent test transaction");

    // Test 4: Query status
    println!("\nTest 4: Querying node status...");
    let status_msg = ClientMessage::QueryStatus;
    let msg = serde_json::to_string(&status_msg)?;
    write.send(Message::Text(msg)).await?;
    println!("✓ Sent status query");

    // Listen for responses with timeout
    println!("\nListening for events (30 second timeout)...");
    let mut messages_received = 0;
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);

    while let Some(msg) = read.next().await {
        if start.elapsed() > timeout {
            println!("Reached timeout period");
            break;
        }

        match msg {
            Ok(Message::Text(text)) => {
                messages_received += 1;
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(server_msg) => {
                        match server_msg {
                            ServerMessage::ConsensusUpdate { 
                                round_number,
                                status,
                                coordinator,
                                votes_count,
                                participation_rate,
                                remaining_time_ms 
                            } => {
                                println!("\n📢 Consensus Update:");
                                println!("  Round: {}", round_number);
                                println!("  Status: {}", status);
                                println!("  Coordinator: {}", coordinator);
                                println!("  Votes: {}", votes_count);
                                println!("  Participation: {:.1}%", participation_rate * 100.0);
                                println!("  Time remaining: {}ms", remaining_time_ms);
                            },
                            ServerMessage::BlockFinalized {
                                block_number,
                                transactions_count,
                                timestamp,
                                proposer
                            } => {
                                println!("\n📦 Block Finalized:");
                                println!("  Block: #{}", block_number);
                                println!("  Transactions: {}", transactions_count);
                                println!("  Proposer: {}", proposer);
                                println!("  Timestamp: {}", timestamp);
                            },
                            ServerMessage::ReputationUpdate {
                                did,
                                change,
                                new_total,
                                reason
                            } => {
                                println!("\n⭐ Reputation Update:");
                                println!("  DID: {}", did);
                                println!("  Change: {}", change);
                                println!("  New Total: {}", new_total);
                                println!("  Reason: {}", reason);
                            },
                            ServerMessage::CommandResponse {
                                command,
                                status,
                                message
                            } => {
                                println!("\n✉️ Command Response:");
                                println!("  Command: {}", command);
                                println!("  Status: {}", status);
                                println!("  Message: {}", message);
                            },
                            ServerMessage::Error {
                                code,
                                message
                            } => {
                                println!("\n❌ Error:");
                                println!("  Code: {}", code);
                                println!("  Message: {}", message);
                            }
                        }
                    },
                    Err(e) => println!("❌ Error parsing message: {}", e),
                }
            },
            Ok(Message::Close(..)) => {
                println!("Server closed connection");
                break;
            },
            Err(e) => println!("❌ Error: {}", e),
            _ => {}
        }

        // Small delay to prevent flooding
        sleep(Duration::from_millis(100)).await;
    }

    println!("\nTest summary:");
    println!("Messages received: {}", messages_received);
    println!("Time elapsed: {:.1}s", start.elapsed().as_secs_f32());
    println!("\nTest client completed!");

    Ok(())
}