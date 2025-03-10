// api/network.rs
use warp::Filter;
use serde::{Deserialize, Serialize};
use log::{info, debug, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::networking::{NetworkManager, PeerStatus};
use std::convert::Infallible;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub status: String,
    pub peer_count: u32,
    pub avg_latency: u32,
    pub bandwidth_usage: f32,
    pub uptime: u64,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeer {
    pub id: String,
    pub address: String,
    pub latency: u32,
    pub connected_since: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeersResponse {
    pub peers: Vec<NetworkPeer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRequest {
    pub count: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    pub peer_id: String,
    pub latency: u32,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    pub status: String,
    pub details: String,
    pub recommendations: Vec<String>,
}

/// Network routes handler
pub fn network_routes(network: Arc<Mutex<NetworkManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let status_route = warp::path!("api" / "v1" / "network" / "status")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_network(network.clone()))
        .and_then(handle_status);

    let peers_route = warp::path!("api" / "v1" / "network" / "peers")
        .and(warp::get())
        .and(with_network(network.clone()))
        .and_then(handle_peers);

    let connect_route = warp::path!("api" / "v1" / "network" / "peers" / "connect")
        .and(warp::post())
        .and(warp::body::json::<ConnectRequest>())
        .and(with_network(network.clone()))
        .and_then(handle_connect);

    let disconnect_route = warp::path!("api" / "v1" / "network" / "peers" / String / "disconnect")
        .and(warp::post())
        .and(with_network(network.clone()))
        .and_then(handle_disconnect);
        
    let ping_route = warp::path!("api" / "v1" / "network" / "peers" / String / "ping")
        .and(warp::post())
        .and(warp::body::json::<PingRequest>())
        .and(with_network(network.clone()))
        .and_then(handle_ping);
        
    let diagnostics_route = warp::path!("api" / "v1" / "network" / "diagnostics")
        .and(warp::get())
        .and(with_network(network.clone()))
        .and_then(handle_diagnostics);

    // Combine all routes
    status_route
        .or(peers_route)
        .or(connect_route)
        .or(disconnect_route)
        .or(ping_route)
        .or(diagnostics_route)
}

async fn handle_status(
    params: std::collections::HashMap<String, String>,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network status requested");
    let detailed = params.get("detail").map(|v| v == "true").unwrap_or(false);
    
    let network = network.lock().await;
    
    let status = NetworkStatus {
        status: "running".to_string(),
        peer_count: network.get_connected_peer_count(),
        avg_latency: if detailed { network.get_average_latency() } else { 0 },
        bandwidth_usage: if detailed { network.get_bandwidth_usage() } else { 0.0 },
        uptime: network.get_uptime_seconds(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    debug!("Responding with network status: {:?}", status);
    Ok(warp::reply::json(&status))
}

async fn handle_peers(
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network peers list requested");
    let network = network.lock().await;
    let peers: Vec<NetworkPeer> = network.get_peers()
        .into_iter()
        .map(|p| NetworkPeer {
            id: p.id.clone(),
            address: p.address.clone(),
            latency: p.latency as u32,
            connected_since: p.connected_since
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            status: format!("{:?}", p.status).to_lowercase(),
        })
        .collect();
    
    debug!("Responding with {} peers", peers.len());
    Ok(warp::reply::json(&NetworkPeersResponse { peers }))
}

async fn handle_connect(
    connect_req: ConnectRequest,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let addr = connect_req.address;
    info!("Network connect requested to: {}", addr);
    let mut network = network.lock().await;
    
    // Generate a random peer ID for demonstration
    use rand::Rng;
    let peer_id = format!("peer_{}", rand::thread_rng().gen::<u32>());
    
    match network.add_peer(peer_id.clone(), addr.clone()) {
        Ok(_) => {
            let peer = NetworkPeer {
                id: peer_id,
                address: addr,
                latency: 0,
                connected_since: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                status: "connected".to_string(),
            };
            debug!("Successfully connected to peer: {:?}", peer);
            Ok(warp::reply::json(&peer))
        },
        Err(e) => {
            error!("Failed to connect to peer: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": e
            })))
        },
    }
}

async fn handle_disconnect(
    peer_id: String,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network disconnect requested for peer: {}", peer_id);
    let mut network = network.lock().await;
    
    match network.remove_peer(&peer_id) {
        Ok(_) => {
            debug!("Successfully disconnected from peer: {}", peer_id);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "message": format!("Disconnected from peer {}", peer_id)
            })))
        },
        Err(e) => {
            error!("Failed to disconnect from peer: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": e
            })))
        },
    }
}

async fn handle_ping(
    peer_id: String,
    ping_req: PingRequest,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let count = ping_req.count.min(10); // Limit to maximum 10 pings
    info!("Ping requested for peer {} ({} times)", peer_id, count);
    
    let network = network.lock().await;
    
    // Check if peer exists
    if !network.has_peer(&peer_id) {
        error!("Peer not found: {}", peer_id);
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": format!("Peer not found: {}", peer_id)
        })));
    }
    
    // Simulate ping results
    let mut results = Vec::with_capacity(count as usize);
    for i in 0..count {
        // Simulate some failures and varying latencies
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let success = rng.gen_bool(0.9); // 90% success rate
        let latency = if success { 
            rng.gen_range(5..100) 
        } else { 
            0 
        };
        
        results.push(PingResult {
            peer_id: peer_id.clone(),
            latency,
            success,
        });
        
        // Simulate slight delay between pings
        if i < count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
    
    debug!("Ping results: {:?}", results);
    Ok(warp::reply::json(&results))
}

async fn handle_diagnostics(
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network diagnostics requested");
    let network = network.lock().await;
    
    // Get peer statistics
    let peer_count = network.get_connected_peer_count();
    let avg_latency = network.get_average_latency();
    let bandwidth = network.get_bandwidth_usage();
    
    // Generate diagnostic report
    let mut details = String::new();
    details.push_str(&format!("Connected peers: {}\n", peer_count));
    details.push_str(&format!("Average latency: {}ms\n", avg_latency));
    details.push_str(&format!("Bandwidth usage: {:.2}%\n", bandwidth));
    details.push_str(&format!("Node uptime: {} seconds\n", network.get_uptime_seconds()));
    
    // Generate recommendations based on diagnostics
    let mut recommendations = Vec::new();
    if peer_count < 3 {
        recommendations.push("Consider connecting to more peers for better network resilience".to_string());
    }
    if avg_latency > 100 {
        recommendations.push("High latency detected. Consider connecting to closer nodes".to_string());
    }
    if bandwidth > 80.0 {
        recommendations.push("High bandwidth usage. Consider optimizing data transfer".to_string());
    }
    
    let response = DiagnosticsResponse {
        status: "completed".to_string(),
        details,
        recommendations,
    };
    
    debug!("Diagnostics completed with {} recommendations", recommendations.len());
    Ok(warp::reply::json(&response))
}

fn with_network(network: Arc<Mutex<NetworkManager>>) -> impl Filter<Extract = (Arc<Mutex<NetworkManager>>,), Error = Infallible> + Clone {
    warp::any().map(move || network.clone())
}