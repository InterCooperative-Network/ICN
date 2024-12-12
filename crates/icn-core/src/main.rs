use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use tracing::{info, error, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use config::{Config, File};
use anyhow::Result;
use serde::Deserialize;

use icn_types::*;
use icn_consensus::{ConsensusConfig, ProofOfCooperation};
use icn_p2p::websocket::WebSocketHandler;
use icn_storage::state::StateManager;
use icn_core::{Core, ConfigBuilder, SystemEvent, NodeStatus};

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    consensus: ConsensusConfig,
    storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct StorageConfig {
    url: String,
    max_connections: u32,
}

async fn setup_config() -> Result<AppConfig> {
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name("config/local").required(false))
        .build()?;

    Ok(config.try_deserialize()?)
}

async fn setup_logging() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into())
            .add_directive("icn_core=debug".parse()?))
        .with_thread_ids(true)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .try_init()?;

    Ok(())
}

async fn setup_metrics() -> Result<()> {
    // TODO: Implement metrics setup with prometheus
    Ok(())
}

async fn setup_storage(config: &StorageConfig) -> Result<Arc<StateManager>> {
    let storage = StateManager::with_config(config.url.clone(), config.max_connections).await?;
    Ok(Arc::new(storage))
}

async fn setup_consensus(
    config: ConsensusConfig,
    ws_handler: Arc<WebSocketHandler>,
) -> Result<Arc<RwLock<ProofOfCooperation>>> {
    let consensus = ProofOfCooperation::new(config, ws_handler);
    Ok(Arc::new(RwLock::new(consensus)))
}

fn setup_routes(
    ws_handler: Arc<WebSocketHandler>,
    storage: Arc<StateManager>,
    consensus: Arc<RwLock<ProofOfCooperation>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::header::<String>("X-DID"))
        .and(warp::any().map(move || ws_handler.clone()))
        .map(|ws: warp::ws::Ws, did: String, handler: Arc<WebSocketHandler>| {
            ws.on_upgrade(move |socket| async move {
                if let Err(e) = handler.handle_connection(socket, did).await {
                    error!("WebSocket connection error: {}", e);
                }
            })
        });

    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| "OK");

    ws_route.or(health_route)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup phase
    setup_logging().await?;
    info!("Starting ICN node...");

    let config = setup_config().await?;
    info!("Configuration loaded");

    setup_metrics().await?;
    info!("Metrics initialized");

    // Initialize core systems
    let storage = setup_storage(&config.storage).await?;
    info!("Storage system initialized");

    let ws_handler = Arc::new(WebSocketHandler::new());
    info!("WebSocket handler initialized");
    
    let consensus = setup_consensus(config.consensus, ws_handler.clone()).await?;
    info!("Consensus system initialized");

    // Setup routes
    let routes = setup_routes(ws_handler, storage, consensus);
    info!("Routes configured");

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");

    info!("Starting server on {}", addr);
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}
