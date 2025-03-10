use warp::Filter;
use serde::{Deserialize, Serialize};
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::networking::{NetworkManager, PeerStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub status: String,
    pub peer_count: u32,
    pub avg_latency: u32,
    pub bandwidth_usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeer {
    pub id: String,
    pub address: String,
    pub latency: u32,
    pub connected_since: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeersResponse {
    pub peers: Vec<NetworkPeer>,
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

    let connect_route = warp::path!("api" / "v1" / "network" / "connect")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_network(network.clone()))
        .and_then(handle_connect);

    let disconnect_route = warp::path!("api" / "v1" / "network" / "disconnect")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_network(network.clone()))
        .and_then(handle_disconnect);

    status_route
        .or(peers_route)
        .or(connect_route)
        .or(disconnect_route)
}

async fn handle_status(
    params: std::collections::HashMap<String, String>,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network status requested");
    let detailed = params.get("detail").map(|v| v == "true").unwrap_or(false);
    
    let network = network.lock().await;
    
    Ok(warp::reply::json(&NetworkStatus {
        status: "running".to_string(),
        peer_count: network.get_connected_peer_count(),
        avg_latency: if detailed { network.get_average_latency() } else { 0 },
        bandwidth_usage: if detailed { network.get_bandwidth_usage() } else { 0.0 },
    }))
}

async fn handle_peers(
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network peers list requested");
    let network = network.lock().await;
    let peers: Vec<NetworkPeer> = network.get_peers()
        .into_iter()
        .filter(|p| matches!(p.status, PeerStatus::Connected))
        .map(|p| NetworkPeer {
            id: p.id.clone(),
            address: p.address.clone(),
            latency: p.latency as u32,
            connected_since: p.connected_since
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        })
        .collect();
    
    Ok(warp::reply::json(&NetworkPeersResponse { peers }))
}

async fn handle_connect(
    addr: String,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network connect requested to: {}", addr);
    let mut network = network.lock().await;
    
    // Generate a random peer ID for this example
    let peer_id = format!("peer_{}", rand::random::<u32>());
    match network.add_peer(peer_id.clone(), addr.clone()) {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "peer_id": peer_id,
            "message": format!("Connected to {}", addr)
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": e
        }))),
    }
}

async fn handle_disconnect(
    peer_id: String,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Network disconnect requested for peer: {}", peer_id);
    let mut network = network.lock().await;
    
    match network.remove_peer(&peer_id) {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": format!("Disconnected from peer {}", peer_id)
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": e
        }))),
    }
}

fn with_network(network: Arc<Mutex<NetworkManager>>) -> impl Filter<Extract = (Arc<Mutex<NetworkManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network.clone())
}