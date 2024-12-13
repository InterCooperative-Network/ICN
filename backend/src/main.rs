mod blockchain;
mod identity;
mod reputation;
mod governance;
mod utils;
mod vm;
mod websocket;
mod consensus;
mod api;
mod claims;
mod community;
mod cooperative;
mod monitoring;
mod network;
mod relationship;

use std::sync::{Arc, Mutex};
use warp::Filter;
use crate::websocket::WebSocketHandler;
use crate::blockchain::Blockchain;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::consensus::{ProofOfCooperation, types::ConsensusConfig};
use crate::api::cooperative::CooperativeApi;
use crate::claims::ClaimsSystem;
use crate::community::CommunitySystem;
use crate::cooperative::CooperativeSystem;
use crate::monitoring::MonitoringSystem;
use crate::network::NetworkSystem;
use crate::relationship::RelationshipSystem;

#[tokio::main]
async fn main() {
    // Initialize core systems
    let identity_system = Arc::new(Mutex::new(IdentitySystem::new()));
    let reputation_system = Arc::new(Mutex::new(ReputationSystem::new()));
    let claims_system = Arc::new(Mutex::new(ClaimsSystem::new()));
    let community_system = Arc::new(Mutex::new(CommunitySystem::new()));
    let cooperative_system = Arc::new(Mutex::new(CooperativeSystem::new()));
    let monitoring_system = Arc::new(Mutex::new(MonitoringSystem::new()));
    let network_system = Arc::new(Mutex::new(NetworkSystem::new()));
    let relationship_system = Arc::new(Mutex::new(RelationshipSystem::new()));
    
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

    // Define API routes
    let api_route = warp::path("api")
        .and(warp::path("cooperative"))
        .and(warp::any().map(move || Arc::new(CooperativeApi::new(cooperative_system.clone()))))
        .and(warp::path::end())
        .map(|api: Arc<CooperativeApi>| {
            warp::reply::json(&api.get_info())
        });

    println!("Starting server on localhost:8088");
    warp::serve(ws_route.or(api_route))
        .run(([127, 0, 0, 1], 8088))
        .await;
}
