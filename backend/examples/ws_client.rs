use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use http::{Request, Uri};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    RegisterValidator {
        did: String,
        initial_reputation: i64,
    },
    SubmitTransaction {
        transaction: serde_json::Value,
    },
    QueryStatus,
    Subscribe {
        events: Vec<String>,
    },
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    RecordMutualAid {
        receiver: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
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
    CommandResponse {
        command: String,
        status: String,
        message: String,
        data: Option<serde_json::Value>,
    },
    Error {
        code: String,
        message: String,
    },
}

async fn handle_user_input(
    tx: mpsc::Sender<ClientMessage>, 
    mut shutdown: mpsc::Receiver<()>
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    loop {
        println!("\nICN Client - Available Commands:");
        println!("1. Register as validator");
        println!("2. Query consensus status");
        println!("3. Subscribe to events");
        println!("4. Record contribution");
        println!("5. Record mutual aid");
        println!("6. Exit");
        println!("\nEnter command number: ");

        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        match line.trim() {
                            "1" => {
                                println!("Enter your DID: ");
                                if let Ok(Some(did)) = lines.next_line().await {
                                    let msg = ClientMessage::RegisterValidator {
                                        did: did.trim().to_string(),
                                        initial_reputation: 100,
                                    };
                                    tx.send(msg).await?;
                                }
                            },
                            "2" => {
                                let msg = ClientMessage::QueryStatus;
                                tx.send(msg).await?;
                            },
                            "3" => {
                                println!("Available events: consensus, blocks, validations");
                                println!("Enter event names (comma separated): ");
                                if let Ok(Some(events)) = lines.next_line().await {
                                    let events: Vec<String> = events
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                    let msg = ClientMessage::Subscribe { events };
                                    tx.send(msg).await?;
                                }
                            },
                            "4" => {
                                println!("Enter contribution description: ");
                                let description = lines.next_line().await?.unwrap_or_default();
                                println!("Enter impact story: ");
                                let impact_story = lines.next_line().await?.unwrap_or_default();
                                println!("Enter context: ");
                                let context = lines.next_line().await?.unwrap_or_default();
                                println!("Enter tags (comma separated): ");
                                let tags = lines.next_line().await?
                                    .unwrap_or_default()
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();

                                let msg = ClientMessage::RecordContribution {
                                    description,
                                    impact_story,
                                    context,
                                    tags,
                                };
                                tx.send(msg).await?;
                            },
                            "5" => {
                                println!("Enter receiver DID: ");
                                let receiver = lines.next_line().await?.unwrap_or_default();
                                println!("Enter description: ");
                                let description = lines.next_line().await?.unwrap_or_default();
                                println!("Enter impact story (optional, press enter to skip): ");
                                let impact_story = match lines.next_line().await?.unwrap_or_default() {
                                    s if s.is_empty() => None,
                                    s => Some(s),
                                };
                                println!("Enter reciprocity notes (optional, press enter to skip): ");
                                let reciprocity_notes = match lines.next_line().await?.unwrap_or_default() {
                                    s if s.is_empty() => None,
                                    s => Some(s),
                                };
                                println!("Enter tags (comma separated): ");
                                let tags = lines.next_line().await?
                                    .unwrap_or_default()
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();

                                let msg = ClientMessage::RecordMutualAid {
                                    receiver,
                                    description,
                                    impact_story,
                                    reciprocity_notes,
                                    tags,
                                };
                                tx.send(msg).await?;
                            },
                            "6" => {
                                println!("Exiting...");
                                return Ok(());
                            },
                            _ => println!("Invalid command"),
                        }
                    }
                    Ok(None) => break,
                    Err(e) => eprintln!("Error reading input: {}", e),
                }
            }
            _ = shutdown.recv() => {
                println!("Shutting down user input handler...");
                break;
            }
        }
    }
    Ok(())
}

async fn handle_server_messages(
    read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
) {
    tokio::pin!(read);

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
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
                                println!("\n=== Consensus Update ===");
                                println!("Round: {}", round_number);
                                println!("Status: {}", status);
                                println!("Coordinator: {}", coordinator);
                                println!("Votes: {}", votes_count);
                                println!("Participation: {:.1}%", participation_rate * 100.0);
                                println!("Time Remaining: {}ms", remaining_time_ms);
                                println!("=====================");
                            },
                            ServerMessage::BlockFinalized {
                                block_number,
                                transactions_count,
                                timestamp,
                                proposer,
                            } => {
                                println!("\n=== Block Finalized ===");
                                println!("Block #: {}", block_number);
                                println!("Transactions: {}", transactions_count);
                                println!("Proposer: {}", proposer);
                                println!("Time: {}", chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap());
                                println!("=====================");
                            },
                            ServerMessage::CommandResponse { command, status, message, data } => {
                                println!("\n=== Command Response ===");
                                println!("Command: {}", command);
                                println!("Status: {}", status);
                                println!("Message: {}", message);
                                if let Some(data) = data {
                                    println!("Data: {}", data);
                                }
                                println!("=====================");
                            },
                            ServerMessage::Error { code, message } => {
                                eprintln!("\n=== Error ===");
                                eprintln!("Code: {}", code);
                                eprintln!("Message: {}", message);
                                eprintln!("=============");
                            }
                        }
                    },
                    Err(e) => eprintln!("Failed to parse server message: {}", e),
                }
            },
            Ok(Message::Close(..)) => {
                println!("Server closed connection");
                break;
            },
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            },
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Create request with WebSocket headers
    let request = Request::builder()
        .uri("ws://localhost:8088/ws")
        .header("Host", "localhost:8088")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
        .header("X-DID", "did:icn:client1")
        .body(())?;

    // Connect to WebSocket server
    let (ws_stream, _) = connect_async(request).await?;
    println!("Connected to ICN WebSocket server");

    let (write, read) = ws_stream.split();
    let (tx, rx) = mpsc::channel(32);
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

    // Spawn writer task
    let writer_task = tokio::spawn(async move {
        let mut write = write;
        let mut rx = rx;
        
        while let Some(msg) = rx.recv().await {
            let msg_json = serde_json::to_string(&msg)?;
            write.send(Message::Text(msg_json)).await?;
        }
        
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    // Spawn reader task
    let reader_task = tokio::spawn(handle_server_messages(read));

    // Handle user input
    handle_user_input(tx, shutdown_rx).await?;

    // Cleanup
    let _ = shutdown_tx.send(()).await;
    let _ = writer_task.await?;
    let _ = reader_task.await;


