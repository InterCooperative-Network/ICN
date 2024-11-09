// src/main.rs

mod blockchain;
mod identity;
mod reputation;
mod governance;
mod utils;
mod vm;
mod websocket;
mod consensus;

use std::sync::{Arc, Mutex};
use warp::Filter;
use crate::websocket::WebSocketHandler;
use crate::blockchain::Blockchain;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::consensus::{ProofOfCooperation, types::ConsensusConfig};

#[tokio::main]
async fn main() {
    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    
    // Create WebSocket handler for real-time updates
    let ws_handler = Arc::new(WebSocketHandler::new());
    
    // Initialize consensus system
    let consensus = Arc::new(Mutex::new(ProofOfCooperation::new(
        ConsensusConfig::default(),
        ws_handler.clone(),
    )));

    // Initialize the blockchain with all required systems
    let blockchain = Arc::new(Mutex::new(Blockchain::new(
        identity_system.clone(),
        reputation_system.clone(),
        consensus.clone(),
    )));

    // Define WebSocket route with DID header for user identification
    let ws_handler = ws_handler.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::header::<String>("X-DID"))
        .and(warp::any().map(move || ws_handler.clone()))
        .map(|ws: warp::ws::Ws, did: String, handler: Arc<WebSocketHandler>| {
            ws.on_upgrade(move |socket| async move {
                handler.handle_connection(socket, did).await;
            })
        });

    println!("Starting WebSocket server on localhost:8081");
    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 8081))
        .await;
}