// examples/blockchain_test.rs

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use std::error::Error;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;

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
    let (tx, mut rx) = mpsc::channel(32);
    let tx = Arc::new(tx);

    // Spawn message listener
    let read_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            println!("Listening for blockchain events...");
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg.to_string().as_str()) {
                            println!("\n🔔 Received event:");
                            println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
                            println!("\nPress Enter to show menu...");
                            let _ = tx.send(()).await;
                        }
                    }
                    Err(e) => eprintln!("Error reading message: {}", e),
                }
            }
        })
    };

    // Print initial menu
    print_menu();

    // Interactive test menu
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => {
                let msg = json!({
                    "type": "RegisterValidator",
                    "did": "did:icn:validator1",
                    "initial_reputation": 100
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent validator registration request");
            }
            "2" => {
                let msg = json!({
                    "type": "SubmitTransaction",
                    "transaction": {
                        "sender": "did:icn:validator1",
                        "receiver": "did:icn:user1",
                        "amount": 100,
                        "timestamp": chrono::Utc::now().timestamp()
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent transaction");
            }
            "3" => {
                let msg = json!({
                    "type": "QueryStatus"
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent status query");
            }
            "4" => {
                let msg = json!({
                    "type": "ProposeBlock",
                    "block": {
                        "transactions": [],
                        "timestamp": chrono::Utc::now().timestamp()
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent block proposal");
            }
            "5" => {
                let msg = json!({
                    "type": "SubmitVote",
                    "vote": {
                        "block_hash": "sample_hash",
                        "approve": true
                    }
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent vote");
            }
            "6" => {
                let msg = json!({
                    "type": "QueryReputation",
                    "did": "did:icn:validator1"
                });
                write.send(Message::Text(msg.to_string())).await?;
                println!("📤 Sent reputation query");
            }
            "q" => {
                break;
            }
            "h" => {
                print_menu();
            }
            _ => {
                println!("Invalid option. Press 'h' for menu");
            }
        }

        // Wait for a response or timeout
        tokio::select! {
            Some(_) = rx.recv() => {
                // Response was handled by the read task
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                println!("Waiting for response...");
            }
        }
    }

    // Cancel the read task
    read_task.abort();
    Ok(())
}

fn print_menu() {
    println!("\n🔷 Blockchain Test Menu:");
    println!("1. Register as validator");
    println!("2. Submit transaction");
    println!("3. Query blockchain status");
    println!("4. Propose block");
    println!("5. Submit vote");
    println!("6. Check reputation");
    println!("h. Show this menu");
    println!("q. Quit");
    println!("\nEnter your choice:");
}