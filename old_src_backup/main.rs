// src/main.rs

use std::sync::{Arc, Mutex};
use warp::Filter;

// Import from the icn_backend library
use icn_backend::{
    blockchain::Blockchain,
    consensus::{ProofOfCooperation, types::ConsensusConfig},
    identity::IdentitySystem,
    relationship::RelationshipSystem, // Import the RelationshipSystem
    reputation::ReputationSystem,
    websocket::WebSocketHandler,
};

#[tokio::main]
async fn main() {
    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let relationship_system = Arc::new(Mutex::new(RelationshipSystem::new())); // Initialize the RelationshipSystem
    
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
        relationship_system.clone(), // Pass the relationship_system to the Blockchain constructor
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

    // Health check route
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| "OK");

    let routes = ws_route.or(health_route);

    println!("Starting WebSocket server on localhost:8088");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8088))
        .await;
}