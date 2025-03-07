use chacha20poly1305::{
    aead::{Aead, NewAead, generic_array::GenericArray},
    XChaCha20Poly1305, Key
};
use blake3::Hasher;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::Mutex;
use x25519_dalek::{PublicKey, SharedSecret};

#[derive(Clone)]
struct CloneableSecret(x25519_dalek::StaticSecret);

impl CloneableSecret {
    fn new() -> Self {
        Self(x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng()))
    }
    
    fn diffie_hellman(&self, peer_public: &PublicKey) -> [u8; 32] {
        self.0.diffie_hellman(peer_public).as_bytes()
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SDPPacket {
    pub header: SDPHeader,
    pub payload: Vec<u8>,
    pub hmac: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SDPHeader {
    pub version: u8,
    pub packet_type: PacketType,
    pub encryption: EncryptionType,
    pub routing: RoutingType,
    pub priority: u8,
    pub nonce: [u8; 24],
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    Handshake,
    Data,
    Ack,
    Control,
}

/// Types of encryption supported by SDP
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionType {
    None,
    XChaCha20Poly1305,
    // Can be extended with other algorithms in the future
}

/// Types of routing supported by SDP
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingType {
    Direct,
    Multipath,
    OnionRouted,
}

/// Manages SDP communication for a node in the ICN network
pub struct SDPManager {
    socket: Arc<Mutex<UdpSocket>>,
    keypair: (CloneableSecret, PublicKey),
    peer_keys: std::collections::HashMap<String, PublicKey>,
    routes: std::collections::HashMap<String, Vec<SocketAddr>>,
}

impl SDPManager {
    /// Create a new SDP Manager with a bound socket
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        
        let secret = CloneableSecret::new();
        let public = PublicKey::from(&secret.0);
        
        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            keypair: (secret, public),
            peer_keys: std::collections::HashMap::new(),
            routes: std::collections::HashMap::new(),
        })
    }
    
    /// Register a peer's public key for secure communication
    pub fn register_peer(&mut self, peer_id: String, public_key: PublicKey, routes: Vec<SocketAddr>) {
        self.peer_keys.insert(peer_id.clone(), public_key);
        self.routes.insert(peer_id, routes);
    }
    
    /// Generate an XChaCha20 key from a shared secret
    fn derive_symmetric_key(shared_secret: &[u8]) -> Key {
        let mut hasher = Hasher::new();
        hasher.update(shared_secret);
        let result = hasher.finalize();
        let key_bytes = result.as_bytes();
        Key::from_slice(key_bytes).clone()
    }
    
    /// Derive shared secret with a peer using X25519
    pub fn derive_shared_secret(&self, peer_public_key: &PublicKey) -> Key {
        let shared_secret = self.keypair.0.diffie_hellman(peer_public_key);
        Self::derive_symmetric_key(&shared_secret)
    }
    
    /// Encrypt a message using XChaCha20-Poly1305
    pub fn encrypt_message(message: &[u8], key: &Key, nonce_bytes: &[u8; 24]) -> Result<Vec<u8>, String> {
        let cipher = XChaCha20Poly1305::new(key);
        let nonce = GenericArray::from_slice(nonce_bytes);
        cipher.encrypt(nonce, message)
            .map_err(|e| format!("Encryption failed: {}", e))
    }
    
    /// Decrypt a message using XChaCha20-Poly1305
    pub fn decrypt_message(encrypted: &[u8], key: &Key, nonce_bytes: &[u8; 24]) -> Result<Vec<u8>, String> {
        let cipher = XChaCha20Poly1305::new(key);
        let nonce = GenericArray::from_slice(nonce_bytes);
        cipher.decrypt(nonce, encrypted)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
    
    /// Send an SDP message to a peer
    pub async fn send_message(&self, peer_id: &str, message: &[u8], priority: u8) -> Result<(), String> {
        // Get peer public key
        let peer_key = self.peer_keys.get(peer_id)
            .ok_or_else(|| format!("Unknown peer: {}", peer_id))?;
            
        // Get routes for this peer
        let routes = self.routes.get(peer_id)
            .ok_or_else(|| format!("No routes for peer: {}", peer_id))?;
            
        if routes.is_empty() {
            return Err("No routes available for peer".to_string());
        }
        
        // Generate nonce
        let mut nonce = [0u8; 24];
        rand::thread_rng().fill(&mut nonce[..]);
        
        // Derive shared key and encrypt message
        let shared_key = self.derive_shared_secret(peer_key);
        let encrypted = Self::encrypt_message(message, &shared_key, &nonce)?;
        
        // Create header
        let header = SDPHeader {
            version: 1,
            packet_type: PacketType::Data,
            encryption: EncryptionType::XChaCha20Poly1305,
            routing: if routes.len() > 1 { RoutingType::Multipath } else { RoutingType::Direct },
            priority,
            nonce,
        };
        
        // Calculate HMAC
        let hmac = blake3::hash(&encrypted).as_bytes().to_vec();
        
        // Create packet
        let packet = SDPPacket {
            header,
            payload: encrypted,
            hmac,
        };
        
        // Serialize packet
        let serialized = bincode::serialize(&packet)
            .map_err(|e| format!("Serialization failed: {}", e))?;
            
        // Send to appropriate routes based on routing strategy
        let socket = self.socket.lock().await;
        
        match packet.header.routing {
            RoutingType::Direct => {
                // Send to first route
                socket.send_to(&serialized, routes[0])
                    .map_err(|e| format!("Failed to send: {}", e))?;
            },
            RoutingType::Multipath => {
                // Send to all routes for resilience
                for route in routes {
                    socket.send_to(&serialized, *route)
                        .map_err(|e| format!("Failed to send to {}: {}", route, e))?;
                }
            },
            RoutingType::OnionRouted => {
                // Implement onion routing for enhanced privacy
                // This would require additional onion wrapping logic
                return Err("Onion routing not implemented yet".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Get the public key associated with this SDP manager
    pub async fn get_public_key(&self) -> PublicKey {
        self.keypair.1
    }

    /// Start listening for incoming SDP messages
    pub async fn start_receiver(&self, handler: impl Fn(Vec<u8>, SocketAddr) + Send + Sync + 'static) -> Result<(), String> {
        let socket_clone = self.socket.clone();
        
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096]; // Buffer for incoming packets
            
            loop {
                match socket_clone.lock().await.recv_from(&mut buffer) {
                    Ok((size, src)) => {
                        // Deserialize packet
                        if let Ok(packet) = bincode::deserialize::<SDPPacket>(&buffer[..size]) {
                            // Verify HMAC
                            let hash = blake3::hash(&packet.payload);
                            let hash_bytes = hash.as_bytes();
                            
                            if hash_bytes == packet.hmac.as_slice() {
                                // Process packet based on type
                                handler(packet.payload.clone(), src);
                            }
                        }
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Non-blocking operation, just continue
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    },
                    Err(e) => {
                        eprintln!("Error receiving SDP packet: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
}