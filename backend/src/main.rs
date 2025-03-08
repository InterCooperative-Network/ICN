mod middleware;

use log::info;
use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Basic health check route
    let health_route = warp::path!("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    // Combine all routes with CORS
    let routes = health_route
        .with(middleware::cors())
        .with(warp::log("icn_backend"));

    // Start the server
    let addr: SocketAddr = ([0, 0, 0, 0], 8081).into();
    info!("Server starting on {}", addr);
    warp::serve(routes)
        .run(addr)
        .await;
}
