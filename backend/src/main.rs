use warp::Filter;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

mod api;
mod networking;

use networking::NetworkManager;

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Initialize NetworkManager with max 100 peers
    let network_manager = Arc::new(Mutex::new(NetworkManager::new(100)));
    let mut network = network_manager.lock().await;
    network.start().expect("Failed to start network manager");
    drop(network);

    // Combine routes and add CORS
    let routes = api::routes(network_manager.clone())
        .with(warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]))
        .with(warp::log("icn_backend"));

    // Start background network monitoring
    let network_manager_clone = network_manager.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            if let Ok(mut network) = network_manager_clone.try_lock() {
                if let Err(e) = network.ping_all_peers().await {
                    eprintln!("Error pinging peers: {}", e);
                }
            }
        }
    });

    // Start the server
    info!("Server starting on http://127.0.0.1:8082");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8082))
        .await;
}
