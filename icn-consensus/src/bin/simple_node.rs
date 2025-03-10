use std::env;
use std::time::Duration;
use anyhow::Result;
use clap::{Parser, ValueEnum};
use log::{info, error};
use serde::{Deserialize, Serialize};
use tokio::time;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;

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
    #[arg(short = 't', long, default_value = "validator")]
    node_type: NodeType,

    /// Port for P2P networking
    #[arg(short = 'p', long, default_value_t = 9000)]
    node_port: u16,

    /// Port for API server
    #[arg(short = 'a', long, default_value_t = 8082)]
    api_port: u16,

    /// Bootstrap nodes to connect to (comma-separated WebSocket URLs)
    #[arg(short = 'b', long, value_delimiter = ',')]
    bootstrap_nodes: Vec<String>,

    /// Cooperative ID
    #[arg(long, default_value = "icn-primary")]
    cooperative_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    env_logger::init_from_env(env_logger::Env::default().default_filter_or(&rust_log));

    // Parse command line arguments
    let args = Args::parse();

    info!("Starting ICN node (type: {:?})...", args.node_type);
    
    // Start P2P mock service
    let p2p_port = args.node_port;
    info!("Starting P2P service on port {}", p2p_port);
    
    // Start API server
    let api_port = args.api_port;
    info!("Starting API server on port {}", api_port);
    
    // Create a simple HTTP server
    let addr = ([0, 0, 0, 0], api_port).into();
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });
    
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("ICN node startup complete - running...");
    info!("API available at http://localhost:{}/api/v1/health", api_port);
    
    // Run the server
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
    
    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let path = req.uri().path();
    
    if path == "/api/v1/health" {
        Ok(Response::new(Body::from("OK")))
    } else {
        let mut not_found = Response::new(Body::from("Not Found"));
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
} 