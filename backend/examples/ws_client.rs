// File: ./backend/examples/ws_client.rs

use futures_util::{Sink, SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::http::header::HeaderValue;
use tokio_tungstenite::{connect_async, tungstenite::http::Request, tungstenite::protocol::Message};
use url::Url;
use std::error::Error;
use std::time::Instant;

#[derive(Debug, Serialize)]
enum ClientMessage {
    RegisterValidator { did: String, initial_reputation: i64 },
    Subscribe { events: Vec<String> },
    SubmitTransaction { transaction: TransactionData },
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
    CommandResponse { command: String, status: String, message: String },
    Error { code: String, message: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting WebSocket test client...");

    // Add "X-DID" header to the WebSocket request
    let url = Url::parse("ws://localhost:8081/ws")?;
    let request = Request::builder()
        .uri(url.as_str())
        .header("X-DID", HeaderValue::from_str("did:icn:test1")?)
        .body(())
        .expect("Failed to build request");

    let (ws_stream, _) = connect_async(request).await?;
    println!("✓ WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Send registration message after connection
    let register_msg = ClientMessage::RegisterValidator {
        did: "did:icn:test1".to_string(),
        initial_reputation: 100,
    };
    send_message(&mut write, &register_msg).await?;
    println!("✓ Sent registration message");

    run_tests(&mut write, &mut read).await?;

    println!("\nTest client completed!");
    Ok(())
}

async fn run_tests<S, R>(
    write: &mut S,
    read: &mut R,
) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + 'static,
    R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    subscribe_events(write).await?;
    submit_transaction(write).await?;
    query_status(write).await?;
    listen_for_responses(read).await;
    Ok(())
}

async fn subscribe_events<S>(write: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + 'static,
{
    println!("\nSubscribing to events...");
    let subscribe_msg = ClientMessage::Subscribe {
        events: vec!["consensus".to_string(), "blocks".to_string(), "reputation".to_string()],
    };
    send_message(write, &subscribe_msg).await?;
    println!("✓ Subscribed to events");
    Ok(())
}

async fn submit_transaction<S>(write: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + 'static,
{
    println!("\nSubmitting test transaction...");
    let transaction = TransactionData {
        sender: "did:icn:test1".to_string(),
        receiver: "did:icn:test2".to_string(),
        amount: 50,
        timestamp: current_timestamp(),
    };
    let tx_msg = ClientMessage::SubmitTransaction { transaction };
    send_message(write, &tx_msg).await?;
    println!("✓ Sent test transaction");
    Ok(())
}

async fn query_status<S>(write: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + 'static,
{
    println!("\nQuerying node status...");
    let status_msg = ClientMessage::QueryStatus;
    send_message(write, &status_msg).await?;
    println!("✓ Sent status query");
    Ok(())
}

async fn listen_for_responses<R>(read: &mut R)
where
    R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    println!("\nListening for events (30 second timeout)...");
    let timeout = Duration::from_secs(30);
    let start = Instant::now();

    while let Some(msg) = read.next().await {
        if start.elapsed() > timeout {
            println!("Reached timeout period");
            break;
        }

        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<ServerMessage>(&text) {
                Ok(server_msg) => handle_server_message(server_msg),
                Err(e) => eprintln!("❌ Error parsing message: {}", e),
            },
            Ok(Message::Close(..)) => {
                println!("Server closed connection");
                break;
            }
            Err(e) => eprintln!("❌ Error: {}", e),
            _ => {}
        }

        sleep(Duration::from_millis(100)).await;
    }
}

async fn send_message<S>(write: &mut S, msg: &ClientMessage) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + 'static,
{
    let msg = serde_json::to_string(msg)?;
    write.send(Message::Text(msg)).await?;
    Ok(())
}

fn handle_server_message(message: ServerMessage) {
    match message {
        ServerMessage::ConsensusUpdate {
            round_number,
            status,
            coordinator,
            votes_count,
            participation_rate,
            remaining_time_ms,
        } => {
            println!("\n📢 Consensus Update:");
            println!("  Round: {}", round_number);
            println!("  Status: {}", status);
            println!("  Coordinator: {}", coordinator);
            println!("  Votes: {}", votes_count);
            println!("  Participation: {:.1}%", participation_rate * 100.0);
            println!("  Time remaining: {}ms", remaining_time_ms);
        }
        ServerMessage::BlockFinalized {
            block_number,
            transactions_count,
            timestamp,
            proposer,
        } => {
            println!("\n📦 Block Finalized:");
            println!("  Block: #{}", block_number);
            println!("  Transactions: {}", transactions_count);
            println!("  Proposer: {}", proposer);
            println!("  Timestamp: {}", timestamp);
        }
        ServerMessage::ReputationUpdate {
            did,
            change,
            new_total,
            reason,
        } => {
            println!("\n⭐ Reputation Update:");
            println!("  DID: {}", did);
            println!("  Change: {}", change);
            println!("  New Total: {}", new_total);
            println!("  Reason: {}", reason);
        }
        ServerMessage::CommandResponse {
            command,
            status,
            message,
        } => {
            println!("\n✉️ Command Response:");
            println!("  Command: {}", command);
            println!("  Status: {}", status);
            println!("  Message: {}", message);
        }
        ServerMessage::Error { code, message } => {
            println!("\n❌ Error:");
            println!("  Code: {}", code);
            println!("  Message: {}", message);
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
