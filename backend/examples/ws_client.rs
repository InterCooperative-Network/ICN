use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{
   connect_async,
   tungstenite::{Message, handshake::client::generate_key},
   MaybeTlsStream, WebSocketStream
};
use serde_json::json;
use url::Url;
use std::error::Error;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio::io::{self, AsyncBufReadExt};
use tokio::net::TcpStream;
use http::Request;

type WsBaseStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WriteSink = futures_util::stream::SplitSink<WsBaseStream, Message>;
type ReadStream = futures_util::stream::SplitStream<WsBaseStream>;

async fn create_websocket_connection(url: &str, did: &str) 
    -> Result<(WriteSink, ReadStream), Box<dyn Error>> 
{
    let url = Url::parse(url)?;
    let ws_key = generate_key();

    let request = Request::builder()
        .uri(url.as_str())
        .header("Host", url.host_str().ok_or("Invalid host")?)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", ws_key)
        .header("X-DID", did)
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;
    Ok(ws_stream.split())
}

// Handler functions
async fn handle_create_cooperative(write_tx: &mpsc::Sender<Message>) -> Result<(), Box<dyn Error>> {
   println!("\n=== Create Cooperative ===");
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   
   println!("Enter cooperative name: ");
   let name = stdin.next_line().await?.unwrap_or_default();
   
   println!("Enter purpose: ");
   let purpose = stdin.next_line().await?.unwrap_or_default();
   
   let msg = json!({
       "type": "CreateCooperative",
       "name": name,
       "purpose": purpose,
       "initialResources": {
           "computeUnits": 100,
           "storageUnits": 100,
           "networkUnits": 100
       }
   });

   write_tx.send(Message::Text(msg.to_string())).await?;
   println!("Cooperative creation request sent!");
   Ok(())
}

async fn handle_submit_transaction(write_tx: &mpsc::Sender<Message>) -> Result<(), Box<dyn Error>> {
   println!("\n=== Submit Transaction ===");
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   
   println!("Enter recipient DID: ");
   let recipient = stdin.next_line().await?.unwrap_or_default();
   
   println!("Enter amount: ");
   let amount: u64 = stdin.next_line().await?.unwrap_or_default().parse().unwrap_or(0);
   
   let msg = json!({
       "type": "SubmitTransaction",
       "transaction": {
           "type": "Transfer",
           "receiver": recipient,
           "amount": amount
       }
   });

   write_tx.send(Message::Text(msg.to_string())).await?;
   println!("Transaction submitted!");
   Ok(())
}

async fn handle_create_proposal(write_tx: &mpsc::Sender<Message>) -> Result<(), Box<dyn Error>> {
   println!("\n=== Create Proposal ===");
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   
   println!("Select proposal type:");
   println!("1. Funding");
   println!("2. Policy Change");
   println!("3. Resource Allocation");
   
   let proposal_type = match stdin.next_line().await?.unwrap_or_default().as_str() {
       "1" => "Funding",
       "2" => "PolicyChange",
       "3" => "ResourceAllocation",
       _ => return Ok(())
   };
   
   println!("Enter proposal title: ");
   let title = stdin.next_line().await?.unwrap_or_default();
   
   println!("Enter description: ");
   let description = stdin.next_line().await?.unwrap_or_default();
   
   let msg = json!({
       "type": "CreateProposal",
       "proposalType": proposal_type,
       "title": title,
       "description": description,
       "duration": 60
   });

   write_tx.send(Message::Text(msg.to_string())).await?;
   println!("Proposal created!");
   Ok(())
}

async fn handle_vote(write_tx: &mpsc::Sender<Message>) -> Result<(), Box<dyn Error>> {
   println!("\n=== Vote on Proposal ===");
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   
   println!("Enter proposal ID: ");
   let proposal_id = stdin.next_line().await?.unwrap_or_default();
   
   println!("Vote (yes/no): ");
   let vote = stdin.next_line().await?.unwrap_or_default().to_lowercase();
   let approve = vote == "yes" || vote == "y";
   
   let msg = json!({
       "type": "CastVote",
       "proposalId": proposal_id,
       "approve": approve
   });

   write_tx.send(Message::Text(msg.to_string())).await?;
   println!("Vote cast!");
   Ok(())
}

async fn handle_resource_management(write_tx: &mpsc::Sender<Message>) -> Result<(), Box<dyn Error>> {
   println!("\n=== Resource Management ===");
   println!("1. View Resources");
   println!("2. Allocate Resources");
   println!("3. Transfer Resources");
   
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   let choice = stdin.next_line().await?.unwrap_or_default();
   
   let msg = match choice.as_str() {
       "1" => json!({
           "type": "QueryResources"
       }),
       "2" => {
           println!("Enter amount to allocate: ");
           let amount = stdin.next_line().await?.unwrap_or_default().parse::<u64>().unwrap_or(0);
           json!({
               "type": "AllocateResources",
               "amount": amount
           })
       },
       "3" => {
           println!("Enter recipient DID: ");
           let recipient = stdin.next_line().await?.unwrap_or_default();
           println!("Enter amount to transfer: ");
           let amount = stdin.next_line().await?.unwrap_or_default().parse::<u64>().unwrap_or(0);
           json!({
               "type": "TransferResources",
               "recipient": recipient,
               "amount": amount
           })
       },
       _ => return Ok(())
   };

   write_tx.send(Message::Text(msg.to_string())).await?;
   println!("Resource management request sent!");
   Ok(())
}

async fn display_menu() {
   println!("\n=== ICN Test Client Menu ===");
   println!("1. Register as Validator");
   println!("2. Create Cooperative");
   println!("3. Submit Transaction");
   println!("4. Create Proposal");
   println!("5. Vote on Proposal");
   println!("6. Check Status");
   println!("7. View Reputation");
   println!("8. Join Cooperative");
   println!("9. List Active Proposals");
   println!("10. Resource Management");
   println!("h. Help");
   println!("q. Quit");
   println!("\nEnter choice: ");
}

async fn display_help() {
   println!("\n=== ICN Client Help ===");
   println!("This client allows you to interact with the ICN network.");
   println!("\nKey Concepts:");
   println!("- DIDs: Your identity on the network");
   println!("- Cooperatives: Groups that share resources");
   println!("- Proposals: Governance decisions to be voted on");
   println!("- Resources: Computing, storage, or network capacity");
   println!("\nBasic Flow:");
   println!("1. Register as validator");
   println!("2. Create or join a cooperative");
   println!("3. Submit proposals or vote on existing ones");
   println!("4. Manage and transfer resources");
   println!("\nPress Enter to continue...");
   let mut stdin = io::BufReader::new(io::stdin()).lines();
   let _ = stdin.next_line().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   println!("Starting ICN WebSocket test client...");

   let test_did = "did:icn:test".to_string();
   let (mut write, mut read) = create_websocket_connection("ws://localhost:8088/ws", &test_did).await?;

   println!("Connected to ICN server!");

   let (write_tx, mut write_rx) = mpsc::channel::<Message>(32);

   // Message writer task
   let write_to_ws = tokio::spawn(async move {
       while let Some(msg) = write_rx.recv().await {
           if let Err(e) = write.send(msg).await {
               eprintln!("Error sending message: {}", e);
               break;
           }
       }
   });

   // Message reader task
   let read_from_ws = tokio::spawn(async move {
       while let Some(msg) = read.next().await {
           match msg {
               Ok(Message::Text(text)) => {
                   println!("\nReceived message:");
                   if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                       println!("{}", serde_json::to_string_pretty(&json).unwrap_or(text));
                   } else {
                       println!("{}", text);
                   }
               }
               Ok(Message::Close(frame)) => {
                   println!("\nServer closed connection: {:?}", frame);
                   break;
               }
               Ok(msg) => println!("\nReceived message: {:?}", msg),
               Err(e) => {
                   eprintln!("\nError receiving message: {}", e);
                   break;
               }
           }
       }
   });

   // Register as validator
   let register_msg = json!({
       "type": "RegisterValidator",
       "did": test_did.clone(),
       "initial_reputation": 100
   });
   write_tx.send(Message::Text(register_msg.to_string())).await?;
   println!("Registered as validator");
   sleep(Duration::from_millis(100)).await;

   // Subscribe to events
   let subscribe_msg = json!({
       "type": "Subscribe",
       "events": ["ConsensusUpdate", "BlockFinalized", "ReputationUpdate", "ProposalUpdate"]
   });
   write_tx.send(Message::Text(subscribe_msg.to_string())).await?;
   println!("Subscribed to events");
   sleep(Duration::from_millis(100)).await;

   // Main menu loop
   loop {
       display_menu().await;
       let mut stdin = io::BufReader::new(io::stdin()).lines();
       
       match stdin.next_line().await?.unwrap_or_default().as_str() {
           "1" => {
               let msg = json!({
                   "type": "RegisterValidator",
                   "did": test_did,
                   "initial_reputation": 100
               });
               write_tx.send(Message::Text(msg.to_string())).await?;
               println!("Registration request sent");
           },
           "2" => handle_create_cooperative(&write_tx).await?,
           "3" => handle_submit_transaction(&write_tx).await?,
           "4" => handle_create_proposal(&write_tx).await?,
           "5" => handle_vote(&write_tx).await?,
           "6" => {
               let msg = json!({"type": "QueryStatus"});
               write_tx.send(Message::Text(msg.to_string())).await?;
               println!("Status query sent");
           },
           "7" => {
               let msg = json!({"type": "QueryReputation"});
               write_tx.send(Message::Text(msg.to_string())).await?;
               println!("Reputation query sent");
           },
           "8" => {
               println!("\nEnter cooperative ID to join: ");
               let coop_id = stdin.next_line().await?.unwrap_or_default();
               let msg = json!({
                   "type": "JoinCooperative",
                   "cooperativeId": coop_id
               });
               write_tx.send(Message::Text(msg.to_string())).await?;
               println!("Join request sent");
           },
           "9" => {
               let msg = json!({"type": "ListProposals"});
               write_tx.send(Message::Text(msg.to_string())).await?;
               println!("Proposal list requested");
           },
           "10" => handle_resource_management(&write_tx).await?,
           "h" | "H" => display_help().await,
           "q" | "Q" => break,
           _ => println!("Invalid choice, please try again or enter 'h' for help")
       }

       sleep(Duration::from_millis(100)).await;
   }

   // Cleanup
   let close_msg = Message::Close(None);
   write_tx.send(close_msg).await?;
   
   sleep(Duration::from_millis(100)).await;
   write_to_ws.abort();
   read_from_ws.abort();
   
   println!("Client shutting down...");
   Ok(())
}
