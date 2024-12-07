use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

use icn_types::*;
use icn_consensus::{ConsensusConfig, ProofOfCooperation};
use icn_p2p::websocket::WebSocketHandler;
use icn_storage::state::StateManager;

#[tokio::main]
async fn main() {
    // Initialize core systems
    let storage = Arc::new(StateManager::new().await);
    let ws_handler = Arc::new(WebSocketHandler::new());
    
    // Initialize consensus system
    let consensus = Arc::new(RwLock::new(ProofOfCooperation::new(
        ConsensusConfig::default(),
        ws_handler.clone(),
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
