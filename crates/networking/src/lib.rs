//! Networking module for the Internet of Cooperative Networks (ICN)
//!
//! This module provides networking capabilities for ICN nodes, including:
//! - Secure communication between nodes
//! - Federation-to-federation communication
//! - Service discovery
//! - NAT traversal
//! - Peer-to-peer messaging

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use thiserror::Error;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use icn_types::FederationId;
use icn_crypto::{KeyPair, Algorithm};

pub mod sdp;
pub mod nat;

pub use sdp::{SDPEndpoint, SDPConfig, SDPMessage, ReliabilityMode};
pub use nat::{NatManager, NatConfig, NatType, NatError};

/// Error types for networking operations
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("SDP error: {0}")]
    SDPError(#[from] sdp::SDPError),
    
    #[error("NAT error: {0}")]
    NatError(#[from] nat::NatError),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),
    
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for networking operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Network protocol types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Protocol {
    /// QUIC protocol for secure, multiplexed connections
    Quic,
    
    /// WebRTC for peer-to-peer connections
    WebRtc,
    
    /// WebSocket for web-based connections
    WebSocket,
    
    /// TCP for basic connections
    Tcp,
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MessageType {
    /// Handshake message for establishing connections
    Handshake,
    
    /// Data message for regular communication
    Data,
    
    /// Control message for network management
    Control,
    
    /// Discovery message for finding peers
    Discovery,
    
    /// Heartbeat message for connection maintenance
    Heartbeat,
}

/// Network message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Unique message ID
    pub id: String,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Source node ID
    pub source: String,
    
    /// Destination node ID
    pub destination: String,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Message timestamp
    pub timestamp: u64,
    
    /// Digital signature
    pub signature: Option<Vec<u8>>,
}

/// Network node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID (usually a public key hash)
    pub id: String,
    
    /// Node addresses
    pub addresses: Vec<SocketAddr>,
    
    /// Node federation ID
    pub federation_id: Option<FederationId>,
    
    /// Node supported protocols
    pub protocols: Vec<Protocol>,
    
    /// Node public key
    pub public_key: Vec<u8>,
    
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

/// Manages network connections and NAT traversal
pub struct NetworkManager {
    sdp: Arc<SDPEndpoint>,
    nat: Arc<NatManager>,
    connections: Arc<RwLock<Vec<SocketAddr>>>,
}

/// Connection interface for different protocols
pub struct Connection {
    /// Remote node info
    pub remote_node: NodeInfo,
    
    /// Connection protocol
    pub protocol: Protocol,
    
    /// Connection state
    pub state: Mutex<ConnectionState>,
}

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    
    /// Connection is established and ready
    Connected,
    
    /// Connection is being closed
    Closing,
    
    /// Connection is closed
    Closed,
    
    /// Connection has failed
    Failed(String),
}

/// Message handler trait
pub trait MessageHandler: Send + Sync {
    /// Handle incoming message
    fn handle_message(&self, message: &NetworkMessage) -> NetworkResult<()>;
    
    /// Get message types this handler can process
    fn message_types(&self) -> Vec<MessageType>;
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(keypair: KeyPair) -> NetworkResult<Self> {
        // Initialize NAT manager first
        let nat_config = NatConfig::default();
        let nat = Arc::new(NatManager::new(nat_config).await?);
        
        // Get external address and NAT type
        let (external_addr, nat_type) = nat.discover_nat().await?;
        
        // Initialize SDP with NAT info
        let sdp_config = SDPConfig {
            bind_address: external_addr,
            max_concurrent_streams: 100,
            keep_alive_interval: std::time::Duration::from_secs(10),
            idle_timeout: std::time::Duration::from_secs(30),
            enable_0rtt: false,
            connection_timeout: 10,
        };
        
        let sdp = Arc::new(SDPEndpoint::new(sdp_config, keypair).await?);
        
        Ok(Self {
            sdp,
            nat,
            connections: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Start the network manager
    pub async fn start(&self) -> NetworkResult<()> {
        info!("Starting network manager");
        
        // TODO: Implement actual network startup logic
        
        Ok(())
    }
    
    /// Connect to a remote peer
    pub async fn connect(&self, addr: SocketAddr) -> NetworkResult<()> {
        // Start NAT traversal if needed
        if self.nat.get_nat_type().await != NatType::None {
            self.nat.start_hole_punching(
                addr.to_string(),
                addr,
            ).await?;
        }
        
        // Establish SDP connection
        self.sdp.connect(addr).await?;
        
        // Store connection
        self.connections.write().await.push(addr);
        
        Ok(())
    }
    
    /// Send message to a peer
    pub async fn send_message(
        &self,
        dest: SocketAddr,
        payload: Vec<u8>,
        reliability: ReliabilityMode,
    ) -> NetworkResult<()> {
        let message = SDPMessage {
            destination: dest,
            payload: bytes::Bytes::from(payload),
            priority: 0,
            reliability,
        };
        
        self.sdp.send(message).await?;
        Ok(())
    }
    
    /// Get our external address
    pub async fn get_external_addr(&self) -> SocketAddr {
        self.sdp.get_external_addr().await
    }
    
    /// Get detected NAT type
    pub async fn get_nat_type(&self) -> NatType {
        self.nat.get_nat_type().await
    }
    
    /// Get active connections
    pub async fn get_connections(&self) -> Vec<SocketAddr> {
        self.connections.read().await.clone()
    }
    
    /// Get active NAT traversal sessions
    pub async fn get_nat_sessions(&self) -> Vec<String> {
        self.nat.get_active_sessions().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_manager() {
        // Generate test keypair
        let keypair = KeyPair::generate(icn_crypto::Algorithm::Ed25519).unwrap();
        
        // Create network manager
        let manager = NetworkManager::new(keypair).await.unwrap();
        
        // Get external address and NAT type
        let addr = manager.get_external_addr().await;
        let nat_type = manager.get_nat_type().await;
        
        println!("External address: {}", addr);
        println!("NAT type: {:?}", nat_type);
        
        // Try connecting to a peer
        let peer_addr = "1.2.3.4:1234".parse().unwrap();
        match manager.connect(peer_addr).await {
            Ok(()) => println!("Connected to peer"),
            Err(e) => println!("Failed to connect: {}", e),
        }
    }
} 