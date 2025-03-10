use std::env;
use std::sync::Arc;
use std::convert::Infallible;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tokio::sync::RwLock;
use tokio::time;
use clap::{Parser, ValueEnum};
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use warp::{Filter, http::StatusCode};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import crates
use icn_consensus::*;
use icn_types::*;
use icn_p2p::*;
use icn_core::core::Core;
use icn_core::identity::IdentityManager;
use icn_core::models::ResourceAllocationSystem;
use icn_core::models::FederationManager;
use icn_core::storage::StorageInterface;
use icn_core::network::NetworkInterface;
use icn_core::telemetry::TelemetryManager;
use icn_core::reputation::ReputationInterface;
use icn_core::vm::RuntimeInterface;

// Define node types
#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
enum NodeType {
    Bootstrap,
    Validator,
    Observer,
}

// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Type of node to run
    #[arg(short, long, env = "NODE_TYPE", default_value = "validator")]
    node_type: NodeType,

    /// Port for P2P networking
    #[arg(short, long, env = "NODE_PORT", default_value_t = 9000)]
    node_port: u16,

    /// Port for API server
    #[arg(short, long, env = "API_PORT", default_value_t = 8082)]
    api_port: u16,

    /// Bootstrap nodes to connect to (comma-separated WebSocket URLs)
    #[arg(short, long, env = "BOOTSTRAP_NODES", use_value_delimiter = true, value_delimiter = ',')]
    bootstrap_nodes: Vec<String>,

    /// Cooperative ID
    #[arg(long, env = "COOPERATIVE_ID", default_value = "icn-primary")]
    cooperative_id: String,
}

// Node state structure
#[derive(Clone)]
struct NodeState {
    id: String,
    node_type: NodeType,
    status: String,
    cooperative_id: String,
    start_time: DateTime<Utc>,
    peers: Vec<String>,
    validators: Vec<ValidatorInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ValidatorInfo {
    id: String,
    address: String,
    status: String,
    last_seen: DateTime<Utc>,
}

// API response types
#[derive(Serialize)]
struct StatusResponse {
    status: String,
    node_id: String,
    node_type: String,
    uptime_seconds: u64,
    peers_connected: usize,
    cooperative_id: String,
    version: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    env_logger::init_from_env(env_logger::Env::default().default_filter_or(&rust_log));

    // Parse command line arguments
    let args = Args::parse();

    info!("Starting ICN node (type: {:?})...", args.node_type);
    
    // Generate a random node ID
    let node_id = format!("node-{}", Uuid::new_v4());
    
    // Create initial node state
    let node_state = Arc::new(RwLock::new(NodeState {
        id: node_id,
        node_type: args.node_type,
        status: "online".to_string(),
        cooperative_id: args.cooperative_id.clone(),
        start_time: Utc::now(),
        peers: vec![],
        validators: vec![],
    }));
    
    // Create mock connect to bootstrap nodes
    if !args.bootstrap_nodes.is_empty() {
        info!("Connecting to bootstrap nodes: {:?}", args.bootstrap_nodes);
        
        // Update peers in node state
        let mut state = node_state.write().await;
        for node in &args.bootstrap_nodes {
            state.peers.push(node.clone());
        }
        
        // Add mock validators
        state.validators.push(ValidatorInfo {
            id: format!("validator-{}", Uuid::new_v4()),
            address: "ws://localhost:9001".to_string(),
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
        
        state.validators.push(ValidatorInfo {
            id: format!("validator-{}", Uuid::new_v4()),
            address: "ws://localhost:9002".to_string(),
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
    }
    
    // Start P2P mock service
    let p2p_port = args.node_port;
    info!("Starting P2P service on port {}", p2p_port);
    
    // Start API server
    let api_port = args.api_port;
    info!("Starting API server on port {}", api_port);
    
    // Routes
    let node_state_filter = warp::any().map(move || node_state.clone());
    
    // GET /api/v1/status
    let status_route = warp::path!("api" / "v1" / "status")
        .and(node_state_filter.clone())
        .and_then(handle_status);
    
    // GET /api/v1/validators
    let validators_route = warp::path!("api" / "v1" / "validators")
        .and(node_state_filter.clone())
        .and_then(handle_validators);
    
    // GET /api/v1/health
    let health_route = warp::path!("api" / "v1" / "health")
        .map(|| "OK");
    
    // Combine all routes
    let routes = status_route
        .or(validators_route)
        .or(health_route)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);
    
    // Start background maintenance task
    let node_state_bg = node_state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut state = node_state_bg.write().await;
            
            // Update last seen time for validators
            for validator in &mut state.validators {
                validator.last_seen = Utc::now();
            }
        }
    });
    
    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    
    // Signal handler task
    tokio::spawn(async move {
        let ctrl_c = tokio::signal::ctrl_c();
        
        tokio::select! {
            _ = ctrl_c => {
                info!("Received shutdown signal, initiating graceful shutdown...");
                let _ = shutdown_tx.send(());
            }
        }
    });
    
    // Start the server in a new task
    let server = warp::serve(routes).bind(([0, 0, 0, 0], api_port));
    let (_, server_future) = warp::serve(routes).bind_with_graceful_shutdown(
        ([0, 0, 0, 0], api_port),
        async move {
            let _ = shutdown_rx.await;
        }
    );
    
    info!("ICN node startup complete - running...");
    info!("API available at http://localhost:{}/api/v1/status", api_port);
    
    // Run the server
    server_future.await;
    
    info!("Node shutdown completed");
    Ok(())
}

async fn handle_status(
    state: Arc<RwLock<NodeState>>
) -> Result<impl warp::Reply, Infallible> {
    let state = state.read().await;
    
    let uptime = Utc::now().signed_duration_since(state.start_time).num_seconds() as u64;
    
    let node_type_str = match state.node_type {
        NodeType::Bootstrap => "bootstrap",
        NodeType::Validator => "validator",
        NodeType::Observer => "observer",
    };
    
    let response = StatusResponse {
        status: state.status.clone(),
        node_id: state.id.clone(),
        node_type: node_type_str.to_string(),
        uptime_seconds: uptime,
        peers_connected: state.peers.len(),
        cooperative_id: state.cooperative_id.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    Ok(warp::reply::json(&response))
}

async fn handle_validators(
    state: Arc<RwLock<NodeState>>
) -> Result<impl warp::Reply, Infallible> {
    let state = state.read().await;
    Ok(warp::reply::json(&state.validators))
}

async fn handle_rejection(rejection: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let error_message = if rejection.is_not_found() {
        "Not Found".to_string()
    } else {
        "Internal Server Error".to_string()
    };
    
    let json = warp::reply::json(&ErrorResponse {
        error: error_message,
    });
    
    let status = if rejection.is_not_found() {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    
    Ok(warp::reply::with_status(json, status))
}

// Mock implementations for demonstration
struct MockStorage;
impl StorageInterface for MockStorage {}

struct MockNetwork;
impl NetworkInterface for MockNetwork {}

struct MockReputation;
impl ReputationInterface for MockReputation {}

struct MockRuntime;
impl RuntimeInterface for MockRuntime {}

struct MockConsensus; 