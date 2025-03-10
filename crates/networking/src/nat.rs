use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::UdpSocket;
use thiserror::Error;
use bytes::{Bytes, BytesMut, BufMut};
use rand::Rng;
use log::{debug, error, info};

/// Default STUN servers to use for NAT traversal
const DEFAULT_STUN_SERVERS: &[&str] = &[
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302",
    "stun2.l.google.com:19302",
    "stun3.l.google.com:19302",
];

#[derive(Error, Debug)]
pub enum NatError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("STUN protocol error: {0}")]
    StunError(String),
    
    #[error("Hole punching failed: {0}")]
    HolePunchingError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

pub type NatResult<T> = Result<T, NatError>;

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq)]
enum StunMessageType {
    BindingRequest = 0x0001,
    BindingResponse = 0x0101,
    BindingError = 0x0111,
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq)]
enum StunAttributeType {
    MappedAddress = 0x0001,
    XorMappedAddress = 0x0020,
    Software = 0x8022,
    Fingerprint = 0x8028,
}

/// Represents a NAT type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NatType {
    None,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
}

/// Configuration for NAT traversal
#[derive(Debug, Clone)]
pub struct NatConfig {
    /// List of STUN servers to use
    pub stun_servers: Vec<String>,
    
    /// Timeout for STUN requests in seconds
    pub stun_timeout: u64,
    
    /// Number of hole punching attempts
    pub hole_punching_attempts: u32,
    
    /// Timeout for hole punching in seconds
    pub hole_punching_timeout: u64,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            stun_servers: DEFAULT_STUN_SERVERS.iter().map(|s| s.to_string()).collect(),
            stun_timeout: 5,
            hole_punching_attempts: 5,
            hole_punching_timeout: 10,
        }
    }
}

/// Manages NAT traversal operations
pub struct NatManager {
    config: NatConfig,
    socket: Arc<UdpSocket>,
    external_addr: Arc<RwLock<Option<SocketAddr>>>,
    nat_type: Arc<RwLock<NatType>>,
    active_sessions: Arc<RwLock<HashMap<String, HolePunchingSession>>>,
}

/// Represents an active hole punching session
#[derive(Debug)]
struct HolePunchingSession {
    peer_id: String,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    start_time: std::time::Instant,
    attempts: u32,
}

impl NatManager {
    pub async fn new(config: NatConfig) -> NatResult<Self> {
        // Bind to random port
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| NatError::NetworkError(e.to_string()))?;
            
        Ok(Self {
            config,
            socket: Arc::new(socket),
            external_addr: Arc::new(RwLock::new(None)),
            nat_type: Arc::new(RwLock::new(NatType::None)),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Discover external IP and NAT type using STUN
    pub async fn discover_nat(&self) -> NatResult<(SocketAddr, NatType)> {
        for stun_server in &self.config.stun_servers {
            match self.stun_request(stun_server).await {
                Ok((addr, nat_type)) => {
                    // Update cached values
                    *self.external_addr.write().await = Some(addr);
                    *self.nat_type.write().await = nat_type;
                    return Ok((addr, nat_type));
                }
                Err(e) => {
                    error!("STUN request to {} failed: {}", stun_server, e);
                    continue;
                }
            }
        }
        
        Err(NatError::StunError("All STUN servers failed".to_string()))
    }
    
    /// Send STUN binding request to server
    async fn stun_request(&self, stun_server: &str) -> NatResult<(SocketAddr, NatType)> {
        // Resolve STUN server address
        let server_addr = tokio::net::lookup_host(stun_server).await
            .map_err(|e| NatError::NetworkError(e.to_string()))?
            .next()
            .ok_or_else(|| NatError::NetworkError("Failed to resolve STUN server".to_string()))?;
            
        // Create STUN binding request
        let transaction_id = rand::thread_rng().gen::<[u8; 12]>();
        let request = self.create_stun_request(transaction_id);
        
        // Send request
        self.socket.send_to(&request, server_addr).await
            .map_err(|e| NatError::NetworkError(e.to_string()))?;
            
        // Wait for response with timeout
        let mut buf = [0u8; 512];
        let timeout = tokio::time::Duration::from_secs(self.config.stun_timeout);
        
        let (size, _) = tokio::select! {
            result = self.socket.recv_from(&mut buf) => {
                result.map_err(|e| NatError::NetworkError(e.to_string()))?
            }
            _ = tokio::time::sleep(timeout) => {
                return Err(NatError::TimeoutError("STUN request timed out".to_string()));
            }
        };
        
        // Parse response
        self.parse_stun_response(&buf[..size], &transaction_id)
    }
    
    /// Create STUN binding request message
    fn create_stun_request(&self, transaction_id: [u8; 12]) -> Bytes {
        let mut buf = BytesMut::with_capacity(20);
        
        // Message Type: Binding Request
        buf.put_u16(StunMessageType::BindingRequest as u16);
        // Message Length: 0 (no attributes)
        buf.put_u16(0);
        // Magic Cookie
        buf.put_u32(0x2112A442);
        // Transaction ID
        buf.put_slice(&transaction_id);
        
        buf.freeze()
    }
    
    /// Parse STUN response message
    fn parse_stun_response(&self, data: &[u8], expected_transaction_id: &[u8; 12]) -> NatResult<(SocketAddr, NatType)> {
        if data.len() < 20 {
            return Err(NatError::StunError("Response too short".to_string()));
        }
        
        // Verify message type
        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        if msg_type != StunMessageType::BindingResponse as u16 {
            return Err(NatError::StunError("Unexpected message type".to_string()));
        }
        
        // Verify transaction ID
        if &data[8..20] != expected_transaction_id {
            return Err(NatError::StunError("Transaction ID mismatch".to_string()));
        }
        
        // Parse attributes
        let mut pos = 20;
        let mut mapped_addr = None;
        let mut xor_mapped_addr = None;
        
        while pos + 4 <= data.len() {
            let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            pos += 4;
            
            if pos + attr_len > data.len() {
                break;
            }
            
            match attr_type {
                t if t == StunAttributeType::MappedAddress as u16 => {
                    mapped_addr = self.parse_address(&data[pos..pos + attr_len]);
                }
                t if t == StunAttributeType::XorMappedAddress as u16 => {
                    xor_mapped_addr = self.parse_xor_address(&data[pos..pos + attr_len]);
                }
                _ => {}
            }
            
            pos += attr_len;
        }
        
        // Prefer XOR-MAPPED-ADDRESS over MAPPED-ADDRESS
        let addr = xor_mapped_addr.or(mapped_addr)
            .ok_or_else(|| NatError::StunError("No address attribute found".to_string()))?;
            
        // Determine NAT type (simplified)
        let nat_type = if addr.ip().is_private() {
            NatType::None
        } else {
            NatType::PortRestrictedCone // More detailed detection would require multiple tests
        };
        
        Ok((addr, nat_type))
    }
    
    /// Parse STUN address attribute
    fn parse_address(&self, data: &[u8]) -> Option<SocketAddr> {
        if data.len() < 4 {
            return None;
        }
        
        let family = data[1];
        let port = u16::from_be_bytes([data[2], data[3]]);
        
        match family {
            0x01 => { // IPv4
                if data.len() < 8 {
                    return None;
                }
                let ip = IpAddr::from([data[4], data[5], data[6], data[7]]);
                Some(SocketAddr::new(ip, port))
            }
            0x02 => { // IPv6
                if data.len() < 20 {
                    return None;
                }
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&data[4..20]);
                let ip = IpAddr::from(ip_bytes);
                Some(SocketAddr::new(ip, port))
            }
            _ => None,
        }
    }
    
    /// Parse STUN XOR-MAPPED-ADDRESS attribute
    fn parse_xor_address(&self, data: &[u8]) -> Option<SocketAddr> {
        let addr = self.parse_address(data)?;
        
        // XOR with magic cookie
        let port = addr.port() ^ (0x2112A442 >> 16) as u16;
        let ip = match addr.ip() {
            IpAddr::V4(ip) => {
                let ip_bytes = ip.octets();
                let xor_ip = u32::from_be_bytes(ip_bytes) ^ 0x2112A442;
                IpAddr::V4(xor_ip.to_be_bytes().into())
            }
            IpAddr::V6(_) => {
                // IPv6 XOR not implemented yet
                return None;
            }
        };
        
        Some(SocketAddr::new(ip, port))
    }
    
    /// Start hole punching to a peer
    pub async fn start_hole_punching(&self, peer_id: String, peer_addr: SocketAddr) -> NatResult<()> {
        let local_addr = self.socket.local_addr()
            .map_err(|e| NatError::NetworkError(e.to_string()))?;
            
        let session = HolePunchingSession {
            peer_id: peer_id.clone(),
            local_addr,
            remote_addr: peer_addr,
            start_time: std::time::Instant::now(),
            attempts: 0,
        };
        
        // Store session
        self.active_sessions.write().await.insert(peer_id.clone(), session);
        
        // Start hole punching task
        let socket = self.socket.clone();
        let config = self.config.clone();
        let sessions = self.active_sessions.clone();
        
        tokio::spawn(async move {
            let result = Self::hole_punching_task(
                socket,
                peer_id.clone(),
                peer_addr,
                config.hole_punching_attempts,
                config.hole_punching_timeout,
            ).await;
            
            // Remove session when done
            sessions.write().await.remove(&peer_id);
            
            if let Err(e) = result {
                error!("Hole punching to {} failed: {}", peer_id, e);
            }
        });
        
        Ok(())
    }
    
    /// Hole punching task
    async fn hole_punching_task(
        socket: Arc<UdpSocket>,
        peer_id: String,
        peer_addr: SocketAddr,
        max_attempts: u32,
        timeout: u64,
    ) -> NatResult<()> {
        let mut attempts = 0;
        let timeout_duration = tokio::time::Duration::from_secs(timeout);
        let start_time = std::time::Instant::now();
        
        // Send hole punching packets
        while attempts < max_attempts {
            // Send packet
            let packet = format!("HOLE_PUNCH:{}", peer_id);
            socket.send_to(packet.as_bytes(), peer_addr).await
                .map_err(|e| NatError::NetworkError(e.to_string()))?;
                
            attempts += 1;
            
            // Check for response
            let mut buf = [0u8; 1024];
            match tokio::time::timeout(timeout_duration, socket.recv_from(&mut buf)).await {
                Ok(Ok((size, addr))) => {
                    if addr == peer_addr {
                        let response = String::from_utf8_lossy(&buf[..size]);
                        if response.starts_with("HOLE_PUNCH_ACK:") {
                            info!("Hole punching to {} successful", peer_id);
                            return Ok(());
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Receive error: {}", e);
                }
                Err(_) => {
                    // Timeout, continue with next attempt
                }
            }
            
            // Check overall timeout
            if start_time.elapsed() >= timeout_duration {
                return Err(NatError::TimeoutError("Hole punching timed out".to_string()));
            }
            
            // Wait before next attempt
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        Err(NatError::HolePunchingError("Maximum attempts reached".to_string()))
    }
    
    /// Get the current external address
    pub async fn get_external_addr(&self) -> Option<SocketAddr> {
        *self.external_addr.read().await
    }
    
    /// Get the detected NAT type
    pub async fn get_nat_type(&self) -> NatType {
        *self.nat_type.read().await
    }
    
    /// Get active hole punching sessions
    pub async fn get_active_sessions(&self) -> Vec<String> {
        self.active_sessions.read().await
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_nat_discovery() {
        let config = NatConfig::default();
        let nat_manager = NatManager::new(config).await.unwrap();
        
        match nat_manager.discover_nat().await {
            Ok((addr, nat_type)) => {
                println!("External address: {}", addr);
                println!("NAT type: {:?}", nat_type);
                assert!(addr.ip().is_global());
            }
            Err(e) => {
                println!("NAT discovery failed: {}", e);
                // Don't fail test as it depends on network connectivity
            }
        }
    }
    
    #[tokio::test]
    async fn test_stun_message_parsing() {
        let config = NatConfig::default();
        let nat_manager = NatManager::new(config).await.unwrap();
        
        // Create test transaction ID
        let transaction_id = [1u8; 12];
        
        // Create test response
        let mut response = Vec::new();
        // Message type: Binding Response
        response.extend_from_slice(&[0x01, 0x01]);
        // Message length: 12 (size of XOR-MAPPED-ADDRESS attribute)
        response.extend_from_slice(&[0x00, 0x0c]);
        // Magic cookie
        response.extend_from_slice(&[0x21, 0x12, 0xa4, 0x42]);
        // Transaction ID
        response.extend_from_slice(&transaction_id);
        // XOR-MAPPED-ADDRESS attribute
        response.extend_from_slice(&[0x00, 0x20]); // Type
        response.extend_from_slice(&[0x00, 0x08]); // Length
        response.extend_from_slice(&[0x00, 0x01]); // Family (IPv4)
        response.extend_from_slice(&[0x04, 0xd2]); // Port 1234
        response.extend_from_slice(&[192, 168, 1, 1]); // IP 192.168.1.1
        
        let (addr, nat_type) = nat_manager.parse_stun_response(&response, &transaction_id).unwrap();
        assert_eq!(addr.port(), 1234 ^ 0xa442);
        assert_eq!(nat_type, NatType::PortRestrictedCone);
    }
} 