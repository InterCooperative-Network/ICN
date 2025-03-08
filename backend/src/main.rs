mod middleware;
mod api;

use log::info;
use std::net::SocketAddr;
use warp::Filter;
use std::sync::Arc;
use crate::services::{BlockchainService, IdentityService, GovernanceService};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Initialize services
    let blockchain_service = Arc::new(BlockchainService::new());
    let identity_service = Arc::new(IdentityService::new());
    let governance_service = Arc::new(GovernanceService::new());

    // Initialize API server
    let api_server = api::ApiServer::new(8081, blockchain_service, identity_service, governance_service);

    // Run the API server
    if let Err(e) = api_server.run().await {
        eprintln!("Failed to start API server: {}", e);
    }
}
