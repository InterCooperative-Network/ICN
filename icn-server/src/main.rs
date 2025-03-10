use log::info;
use warp::Filter;
use serde::{Deserialize, Serialize};
use std::env;

mod config;
use config::ServerConfig;

// Define data structures for API responses
#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Resource {
    id: String,
    resource_type: String,
    capacity: String,
    owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Identity {
    did: String,
    public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cooperative {
    id: String,
    name: String,
    member_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Health check route
    let health_route = warp::path!("api" / "v1" / "health")
        .and(warp::get())
        .map(|| {
            info!("Health check requested");
            warp::reply::json(&HealthResponse {
                status: "ok".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })
        });

    // Resource routes
    let resources = vec![
        Resource {
            id: "res_1".to_string(),
            resource_type: "compute".to_string(),
            capacity: "10 CPU cores".to_string(),
            owner: "coop_1".to_string(),
        },
        Resource {
            id: "res_2".to_string(),
            resource_type: "storage".to_string(),
            capacity: "100 GB".to_string(),
            owner: "coop_2".to_string(),
        },
    ];

    let resource_list = warp::path!("api" / "v1" / "resources")
        .and(warp::get())
        .map(move || {
            info!("Resource list requested");
            warp::reply::json(&serde_json::json!({ "resources": resources.clone() }))
        });

    // Identity routes
    let identities = vec![
        Identity {
            did: "did:icn:abc123".to_string(),
            public_key: "0x123456".to_string(),
        },
        Identity {
            did: "did:icn:def456".to_string(),
            public_key: "0x789012".to_string(),
        },
    ];

    let identity_list = warp::path!("api" / "v1" / "identities")
        .and(warp::get())
        .map(move || {
            info!("Identity list requested");
            warp::reply::json(&serde_json::json!({ "identities": identities.clone() }))
        });

    // Cooperative routes
    let cooperatives = vec![
        Cooperative {
            id: "coop_1".to_string(),
            name: "Example Cooperative 1".to_string(),
            member_count: 5,
        },
        Cooperative {
            id: "coop_2".to_string(),
            name: "Example Cooperative 2".to_string(),
            member_count: 10,
        },
    ];

    let cooperative_list = warp::path!("api" / "v1" / "cooperatives")
        .and(warp::get())
        .map(move || {
            info!("Cooperative list requested");
            warp::reply::json(&serde_json::json!({ "cooperatives": cooperatives.clone() }))
        });

    // Combine routes and add CORS
    let config = ServerConfig::from_env();
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);
    
    let routes = health_route
        .or(resource_list)
        .or(identity_list)
        .or(cooperative_list)
        .with(cors)
        .with(warp::log("icn_backend"));

    // Configure server address and start
    let addr = config.socket_addr();
    info!("Server starting on http://{}", addr);
    
    // Bind to the socket and start the server
    warp::serve(routes)
        .run(addr)
        .await;
    
    Ok(())
}
