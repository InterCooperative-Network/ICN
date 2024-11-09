// examples/ws_client2.rs

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde_json::json;
use std::time::Duration;
use std::error::Error;
use tokio::time::sleep;

const WEBSOCKET_PORT: u16 = 9081; // Using same port as ws_client1

#[derive(Debug)]
struct TestScenario {
    name: String,
    messages: Vec<serde_json::Value>,
    delay: Duration,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = format!("ws://127.0.0.1:{}/ws", WEBSOCKET_PORT);
    let did = "did:icn:testclient2";

    println!("Starting WebSocket Client 2");
    println!("Connecting to {} with DID: {}", url, did);

    // Create connection with custom headers
    let request = tokio_tungstenite::tungstenite::http::Request::builder()
        .uri(url)
        .header("X-DID", did)
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Define test scenarios
    let scenarios = vec![
        TestScenario {
            name: "Validator Registration".to_string(),
            messages: vec![
                json!({
                    "type": "RegisterValidator",
                    "did": did,
                    "initial_reputation": 100
                })
            ],
            delay: Duration::from_secs(2),
        },
        TestScenario {
            name: "Event Subscription".to_string(),
            messages: vec![
                json!({
                    "type": "Subscribe",
                    "events": ["ConsensusUpdate", "BlockFinalized", "ReputationUpdate"]
                })
            ],
            delay: Duration::from_secs(1),
        },
        TestScenario {
            name: "Transaction Submission".to_string(),
            messages: vec![
                json!({
                    "type": "SubmitTransaction",
                    "transaction": {
                        "sender": did,
                        "receiver": "did:icn:testclient1",
                        "amount": 50,
                        "timestamp": chrono::Utc::now().timestamp()
                    }
                })
            ],
            delay: Duration::from_secs(2),
        },
        TestScenario {
            name: "Voting Simulation".to_string(),
            messages: vec![
                json!({
                    "type": "SubmitVote",
                    "proposal_id": "1",
                    "vote": true,
                    "timestamp": chrono::Utc::now().timestamp()
                })
            ],
            delay: Duration::from_secs(2),
        },
    ];

    // Run test scenarios
    for scenario in scenarios {
        println!("\nExecuting scenario: {}", scenario.name);
        
        for msg in scenario.messages {
            write.send(Message::Text(msg.to_string())).await?;
            println!("Sent message: {}", msg);
            
            // Wait for and process responses
            tokio::select! {
                Some(msg) = read.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            println!("Received response: {}", text);
                            
                            // Process specific response types
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                                    match msg_type {
                                        "Error" => {
                                            eprintln!("Error in scenario {}: {}", scenario.name, text);
                                        },
                                        _ => println!("Successful {} response", msg_type)
                                    }
                                }
                            }
                        },
                        Ok(Message::Close(frame)) => {
                            println!("Connection closed by server: {:?}", frame);
                            return Ok(());
                        },
                        Err(e) => {
                            eprintln!("Error receiving message: {}", e);
                            return Err(Box::new(e));
                        },
                        _ => {}
                    }
                },
                _ = sleep(Duration::from_secs(5)) => {
                    println!("No response received within timeout");
                }
            }
        }
        
        sleep(scenario.delay).await;
    }

    // Keep connection alive to observe events
    println!("\nAll scenarios completed. Listening for events...");
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received event: {}", text);
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