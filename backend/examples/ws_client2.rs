use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use url::Url;

}

#[derive(Debug, Serialize, Deserialize)]
struct MutualAidInteraction {
    receiver: String,
    description: String,
    impact_story: Option<String>,
    reciprocity_notes: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    RecordContribution {
        contribution: Contribution,
    },
    RecordMutualAid {
        interaction: MutualAidInteraction,
    },
    Subscribe {
        events: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    ContributionRecorded {
        contribution: Contribution,
    },
    MutualAidProvided {
        interaction: MutualAidInteraction,
    },
    RelationshipUpdated {
        member_one: String,
        member_two: String,
        update_type: String,
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
        println!("\nICN Relationship Client - Commands:");
        println!("1. Record a contribution");
        println!("2. Record mutual aid");
        println!("3. Subscribe to events");
        println!("4. Exit");
        println!("\nEnter command number: ");

        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        match line.trim() {
                            "1" => {
                                println!("Enter contribution description: ");
                                let description = match lines.next_line().await? {
                                    Some(text) => text,
                                    None => continue,
                                };

                                println!("Enter impact story: ");
                                let impact_story = match lines.next_line().await? {
                                    Some(text) => text,
                                    None => continue,
                                };

                                println!("Enter context (e.g., technical, social, organizational): ");
                                let context = match lines.next_line().await? {
                                    Some(text) => text,
                                    None => continue,
                                };

                                println!("Enter tags (comma separated): ");
                                let tags = match lines.next_line().await? {
                                    Some(text) => text.split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect(),
                                    None => continue,
                                };

                                let contribution = Contribution {
                                    description,
                                    impact_story,
                                    context,
                                    tags,
                                };

                                let msg = ClientMessage::RecordContribution { contribution };
                                tx.send(msg).await?;
                            },
                            "2" => {
                                println!("Enter receiver DID: ");
                                let receiver = match lines.next_line().await? {
                                    Some(text) => text,
                                    None => continue,
                                };

                                println!("Enter description of mutual aid: ");
                                let description = match lines.next_line().await? {
                                    Some(text) => text,
                                    None => continue,
                                };

                                println!("Enter impact story (optional - press enter to skip): ");
                                let impact_story = match lines.next_line().await? {
                                    Some(text) if !text.trim().is_empty() => Some(text),
                                    _ => None,
                                };

                                println!("Enter reciprocity notes (optional - press enter to skip): ");
                                let reciprocity_notes = match lines.next_line().await? {
                                    Some(text) if !text.trim().is_empty() => Some(text),
                                    _ => None,
                                };

                                println!("Enter tags (comma separated): ");
                                let tags = match lines.next_line().await? {
                                    Some(text) => text.split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect(),
                                    None => continue,
                                };

                                let interaction = MutualAidInteraction {
                                    receiver,
                                    description,
                                    impact_story,
                                    reciprocity_notes,
                                    tags,
                                };

                                let msg = ClientMessage::RecordMutualAid { interaction };
                                tx.send(msg).await?;
                            },
                            "3" => {
                                println!("Available events: contributions, mutual_aid, relationships");
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
                            ServerMessage::ContributionRecorded { contribution } => {
                                println!("\n=== New Contribution ===");
                                println!("Description: {}", contribution.description);
                                println!("Impact: {}", contribution.impact_story);
                                println!("Context: {}", contribution.context);
                                println!("Tags: {}", contribution.tags.join(", "));
                                println!("=======================");
                            },
                            ServerMessage::MutualAidProvided { interaction } => {
                                println!("\n=== Mutual Aid Interaction ===");
                                println!("With: {}", interaction.receiver);
                                println!("Description: {}", interaction.description);
                                if let Some(impact) = interaction.impact_story {
                                    println!("Impact: {}", impact);
                                }
                                if let Some(notes) = interaction.reciprocity_notes {
                                    println!("Reciprocity Notes: {}", notes);
                                }
                                println!("Tags: {}", interaction.tags.join(", "));
                                println!("===========================");
                            },
                            ServerMessage::RelationshipUpdated { 
                                member_one, 
                                member_two, 
                                update_type 
                            } => {
                                println!("\n=== Relationship Update ===");
                                println!("Between: {} and {}", member_one, member_two);
                                println!("Update Type: {}", update_type);
                                println!("=========================");
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
    // Setup DID header for WebSocket connection
    let mut request = url::Url::parse("ws://localhost:8088/ws")?;
    request.set_query(Some("X-DID=did:icn:client1"));

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

    Ok(())
}
