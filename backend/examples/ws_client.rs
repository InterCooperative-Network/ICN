// examples/ws_client.rs

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde_json::json;
use std::time::Duration;
use std::error::Error;

const WEBSOCKET_PORT: u16 = 9081; // Changed from 8081

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = format!("ws://127.0.0.1:{}/ws", WEBSOCKET_PORT);
    let did = "did:icn:testclient1";

    println!("Connecting to {} with DID: {}", url, did);

    // Create connection with custom headers
    let request = tokio_tungstenite::tungstenite::http::Request::builder()
        .uri(url)
        .header("X-DID", did)
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Send registration message
    let register_msg = json!({
        "type": "RegisterValidator",
        "did": did,
        "initial_reputation": 100
    });
    write.send(Message::Text(register_msg.to_string())).await?;
    println!("Sent validator registration");

    // Subscribe to events
    let subscribe_msg = json!({
        "type": "Subscribe",
        "events": [
            "ConsensusUpdate",
            "BlockFinalized",
            "ReputationUpdate",
            "ValidatorUpdate"
        ]
    });
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    println!("Subscribed to events");

    // Test proposal creation
    let proposal_msg = json!({
        "type": "CreateProposal",
        "title": "Test Proposal",
        "description": "This is a test proposal",
        "resource_amount": 1000,
        "duration": 86400 // 24 hours in seconds
    });
    write.send(Message::Text(proposal_msg.to_string())).await?;
    println!("Created test proposal");

    // Main message loop
    println!("\nListening for messages...");
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                
                // Parse and handle different message types
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                        match msg_type {
                            "ConsensusUpdate" => {
                                println!("Consensus update received");
                                // Handle consensus update
                            },
                            "BlockFinalized" => {
                                println!("Block finalized");
                                // Handle block finalization
                            },
                            "ReputationUpdate" => {
                                println!("Reputation updated");
                                // Handle reputation update
                            },
                            "Error" => {
                                eprintln!("Error received: {}", text);
                                // Handle error
                            },
                            _ => println!("Unknown message type: {}", msg_type)
                        }
                    }
                }
            },
            Ok(Message::Close(frame)) => {
                println!("Connection closed by server: {:?}", frame);
                break;
            },
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            },
            _ => {}
        }
    }

    Ok(())
}