// examples/ws_client.rs

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use url::Url;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse the WebSocket URL
    let url = Url::parse("ws://localhost:8081/ws")?;
    
    // Add the DID header
    let mut request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url)?;
    // In ws_client2.rs
    request.headers_mut().insert(
    "X-DID",
    "did:icn:test456".parse().unwrap()  // Different DID
);
    // Connect to the WebSocket server
    println!("Connecting to WebSocket server...");
    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connected!");

    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to read messages
    tokio::spawn(async move {
        println!("Listening for messages...");
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => println!("Received: {}", msg),
                Err(e) => eprintln!("Error reading message: {}", e),
            }
        }
    });

    // Keep the connection alive and allow for user input
    let mut input = String::new();
    loop {
        println!("\nPress Enter to continue or type 'quit' to exit");
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim() == "quit" {
            break;
        }
        
        // Send a test message
        let test_msg = Message::Text("Hello from client!".into());
        if let Err(e) = write.send(test_msg).await {
            eprintln!("Error sending message: {}", e);
            break;
        }
        
        input.clear();
    }

    Ok(())
}