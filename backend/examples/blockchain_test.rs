// examples/blockchain_test.rs

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use std::error::Error;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to WebSocket
    let url = Url::parse("ws://localhost:8081/ws")?;
    let mut request = IntoClientRequest::into_client_request(url)?;
    request.headers_mut().insert(
        "X-DID",
        "did:icn:validator1".parse().unwrap()
    );

    println!("Connecting to blockchain network...");
    let (ws_stream, _) = connect_async(request).await?;
    println!("Connected!");

    let (mut write, mut read) = ws_stream.split();

    // Spawn message listener
    let read_task = tokio::spawn(async move {
        println!("Listening for blockchain events...");
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    println!("\nReceived blockchain event: {}", msg);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(msg.to_string().as_str()) {
                        println!("Parsed event: {}", serde_json::to_string_pretty(&json).unwrap());
                    }
                }
                Err(e) => eprintln!("Error reading message: {}", e),
            }
        }
    });

    // Interactive test menu
    loop {
        println!("\nBlockchain Test Menu:");
        println!("1. Register as validator");
        println!("2. Submit transaction");
        println!("3. Query blockchain status");
        println!("4. Propose block");
        println!("5. Submit vote");
        println!("6. Check reputation");
        println!("q. Quit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => {
                let msg = json!({
                    "type": "register_validator",
                    "did": "did:icn:validator1",
                    "initial_reputation": 100
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "2" => {
                let msg = json!({
                    "type": "submit_transaction",
                    "transaction": {
                        "sender": "did:icn:validator1",
                        "receiver": "did:icn:user1",
                        "amount": 100,
                        "timestamp": chrono::Utc::now().timestamp()
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "3" => {
                let msg = json!({
                    "type": "query_status"
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "4" => {
                let msg = json!({
                    "type": "propose_block",
                    "block": {
                        "transactions": [],
                        "timestamp": chrono::Utc::now().timestamp()
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "5" => {
                let msg = json!({
                    "type": "submit_vote",
                    "vote": {
                        "block_hash": "sample_hash",
                        "approve": true
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "6" => {
                let msg = json!({
                    "type": "query_reputation",
                    "did": "did:icn:validator1"
                });
                write.send(Message::Text(msg.to_string())).await?;
            }
            "q" => {
                break;
            }
            _ => println!("Invalid option"),
        }

        // Small delay to allow for response processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Wait for the read task to complete
    if let Err(e) = read_task.await {
        eprintln!("Error in read task: {}", e);
    }

    Ok(())
}