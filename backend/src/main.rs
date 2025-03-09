use std::net::SocketAddr;
use warp::Filter;
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};

mod api;
mod middleware;
mod services;
mod core;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Initialize database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    let db_pool = Arc::new(db_pool);

    // Set up routes
    let health_routes = api::health::health_routes(Arc::clone(&db_pool));
    
    // Combine routes with middleware
    let routes = health_routes
        .with(warp::cors().allow_any_origin())
        .with(warp::log("icn_backend"));

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    
    // Set up graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel();
    let server = warp::serve(routes);
    let (addr, server) = server.bind_with_graceful_shutdown(addr, async {
        rx.await.ok();
        info!("Shutdown signal received, stopping server...");
    });

    info!("Server running on http://{}", addr);

    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("Ctrl+C received, initiating graceful shutdown...");
        let _ = tx.send(());
    });

    // Run the server
    server.await;
    info!("Server shutdown complete");
    Ok(())
}
