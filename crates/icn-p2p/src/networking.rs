// networking.rs
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::sync::mpsc;
use log::{info, debug, error, trace};
use serde::{Serialize, Deserialize};
use rand::Rng;
use icn_crypto::PublicKey;

#[derive(Debug, Clone)]
pub enum NetworkTransport {
    TCP,
    UDP,
    WebSocket,
    SDP,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PeerStatus {
    Connected,
    Disconnected,
    Syncing,
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub addresses: Vec<SocketAddr>,
    pub public_key: Option<PublicKey>,
    pub transport: NetworkTransport,
    pub status: PeerStatus,
    pub bandwidth_usage: u64,
    pub last_seen: SystemTime,
}

pub struct NetworkLayer {
    peers: RwLock<HashMap<String, PeerInfo>>,
    total_bandwidth: Mutex<u64>,
    rng: Mutex<rand::rngs::ThreadRng>,
}

impl NetworkLayer {
    pub fn new() -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            total_bandwidth: Mutex::new(0),
            rng: Mutex::new(rand::thread_rng()),
        }
    }

    pub async fn measure_latency(&self, peer_id: &str) -> Result<f64, String> {
        let peers = self.peers.read().await;
        let peer = peers.get(peer_id).ok_or(format!("Peer {} not found", peer_id))?;

        // Clone needed values before dropping peers read lock
        let status = peer.status.clone();
        let addresses = peer.addresses.clone();
        drop(peers); // Release read lock

        let latency = match status {
            PeerStatus::Connected => {
                let start = Instant::now();
                // Simulate network round trip
                let mut rng = self.rng.lock().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(1..50))).await;
                let duration = start.elapsed();
                duration.as_secs_f64() * 1000.0 // Convert to milliseconds
            },
            PeerStatus::Disconnected => {
                return Err(format!("Peer {} is disconnected", peer_id));
            },
            PeerStatus::Syncing | PeerStatus::Unreachable => {
                return Err(format!("Peer {} is not reachable", peer_id));
            }
        };

        self.update_bandwidth_usage(64).await; // 64 bytes for ping packet
        Ok(latency)
    }

    async fn update_bandwidth_usage(&self, bytes: u64) {
        let mut bandwidth = self.total_bandwidth.lock().await;
        *bandwidth += bytes;
    }

    pub async fn simulate_traffic(&self, _peer_id: &str) -> Result<u64, String> {
        let mut rng = self.rng.lock().await;
        let traffic = rng.gen_range(1024..10240); // 1KB to 10KB
        drop(rng);

        self.update_bandwidth_usage(traffic).await;
        Ok(traffic)
    }
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub status: PeerStatus,
    pub latency: u64,
    pub connected_since: SystemTime,
    pub last_seen: SystemTime,
    pub version: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Block { hash: String, data: Vec<u8> },
    Transaction { hash: String, data: Vec<u8> },
    Proposal { id: String, data: Vec<u8> },
    Vote { proposal_id: String, voter: String, approve: bool },
    Identity { did: String, data: Vec<u8> },
    Reputation { did: String, score: i64 },
    Ping { id: u64 },
    Pong { id: u64, received_at: u64 },
}

pub trait NetworkingOperations {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn connect(&mut self, address: &str) -> Result<(), String>;
    fn disconnect(&mut self, address: &str) -> Result<(), String>;
    fn send_message(&mut self, address: &str, message: &[u8]) -> Result<(), String>;
    fn receive_message(&self, address: &str) -> Result<Vec<u8>, String>;
    fn get_peer_latency(&self, peer_id: &str) -> Result<u64, String>;
}

pub struct NetworkManager {
    peers: HashMap<String, Peer>,
    message_sender: Option<mpsc::Sender<Message>>,
    max_peers: usize,
    network_key: Vec<u8>,
    bandwidth_usage: f32,
    last_bandwidth_update: Instant,
    bytes_transferred: u64,
    cache: HashMap<String, Vec<u8>>,
    start_time: Instant,
    ping_stats: HashMap<String, Vec<u64>>,
}

impl NetworkManager {
    pub fn new(max_peers: usize) -> Self {
        // In a real implementation, this would be a proper crypto key
        let network_key = vec![0u8; 32];
        
        Self {
            peers: HashMap::new(),
            message_sender: None,
            max_peers,
            network_key,
            bandwidth_usage: 0.0,
            last_bandwidth_update: Instant::now(),
            bytes_transferred: 0,
            cache: HashMap::new(),
            start_time: Instant::now(),
            ping_stats: HashMap::new(),
        }
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        info!("Starting NetworkManager");
        let (sender, mut receiver) = mpsc::channel(100);
        self.message_sender = Some(sender);
        
        // Start background task for processing messages
        tokio::spawn(async move {
            Self::process_messages(receiver).await;
        });
        
        debug!("NetworkManager started successfully");
        Ok(())
    }
    
    async fn process_messages(mut receiver: mpsc::Receiver<Message>) {
        while let Some(message) = receiver.recv().await {
            match message {
                Message::Block { hash, data: _ } => {
                    debug!("Received block with hash: {}", hash);
                },
                Message::Transaction { hash, data: _ } => {
                    debug!("Received transaction with hash: {}", hash);
                },
                Message::Proposal { id, data: _ } => {
                    debug!("Received proposal with id: {}", id);
                },
                Message::Vote { proposal_id, voter, approve } => {
                    debug!("Received vote on proposal {} from {}: {}", proposal_id, voter, approve);
                },
                Message::Identity { did, data: _ } => {
                    debug!("Received identity for DID: {}", did);
                },
                Message::Reputation { did, score } => {
                    debug!("Received reputation update for DID: {}, new score: {}", did, score);
                },
                Message::Ping { id } => {
                    trace!("Received ping with id: {}", id);
                },
                Message::Pong { id, received_at } => {
                    trace!("Received pong for ping {}, round-trip: {}ms", id, received_at);
                },
            }
        }
    }
    
    pub fn add_peer(&mut self, id: String, address: String) -> Result<(), String> {
        if self.peers.contains_key(&id) {
            return Err(format!("Peer {} already exists", id));
        }
        
        if self.peers.len() >= self.max_peers {
            return Err("Maximum number of peers reached".to_string());
        }
        
        let peer = Peer {
            id: id.clone(),
            address,
            status: PeerStatus::Connected,
            latency: 0,
            connected_since: SystemTime::now(),
            last_seen: SystemTime::now(),
            version: None,
            capabilities: Vec::new(),
        };
        
        info!("Adding peer {}: {:?}", id, peer);
        self.peers.insert(id, peer);
        Ok(())
    }
    
    pub fn remove_peer(&mut self, id: &str) -> Result<(), String> {
        if let Some(peer) = self.peers.remove(id) {
            info!("Removed peer {}: {:?}", id, peer);
            Ok(())
        } else {
            Err(format!("Peer {} not found", id))
        }
    }
    
    pub fn has_peer(&self, id: &str) -> bool {
        self.peers.contains_key(id)
    }
    
    pub fn get_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    pub fn get_connected_peer_count(&self) -> u32 {
        self.peers.values()
            .filter(|p| p.status == PeerStatus::Connected)
            .count() as u32
    }

    pub fn get_average_latency(&self) -> u32 {
        let connected_peers: Vec<_> = self.peers.values()
            .filter(|p| p.status == PeerStatus::Connected)
            .collect();
        
        if connected_peers.is_empty() {
            return 0;
        }

        let total_latency: u64 = connected_peers.iter()
            .map(|p| p.latency)
            .sum();

        (total_latency / connected_peers.len() as u64) as u32
    }

    pub fn update_bandwidth_usage(&mut self, bytes: u64) {
        self.bytes_transferred += bytes;
        let elapsed = self.last_bandwidth_update.elapsed();
        
        if elapsed.as_secs() >= 1 {
            // Calculate MB/s and convert to percentage of a theoretical max of 100MB/s
            let mbps = (self.bytes_transferred as f32) / (1024.0 * 1024.0) / elapsed.as_secs_f32();
            self.bandwidth_usage = (mbps / 100.0) * 100.0; // Assuming 100MB/s as max
            self.bytes_transferred = 0;
            self.last_bandwidth_update = Instant::now();
            
            debug!("Updated bandwidth usage: {:.2}% ({:.2} MB/s)", self.bandwidth_usage, mbps);
        }
    }
    
    pub fn get_bandwidth_usage(&self) -> f32 {
        self.bandwidth_usage
    }
    
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    pub fn send_message(&self, peer_id: &str, message: Message) -> Result<(), String> {
        if !self.peers.contains_key(peer_id) {
            return Err(format!("Peer {} not found", peer_id));
        }
        
        let sender = self.message_sender.as_ref().ok_or("Network not started")?;
        
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender_clone.send(message).await {
                error!("Failed to send message: {}", e);
            }
        });
        
        trace!("Message queued for sending to peer {}", peer_id);
        Ok(())
    }
    
    pub async fn broadcast_message(&self, message: Message) -> Result<(), String> {
        let sender = self.message_sender.as_ref().ok_or("Network not started")?;
        
        let message_str = format!("{:?}", message);
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender_clone.send(message).await {
                error!("Failed to broadcast message: {}", e);
            }
        });
        
        debug!("Message broadcast to all peers: {}", message_str);
        Ok(())
    }
    
    pub async fn ping_all_peers(&mut self) -> Result<(), String> {
        debug!("Pinging all peers");
        let mut statuses_updated = 0;
        
        for peer in self.peers.values_mut() {
            // Simulate pinging the peer
            let mut rng = rand::thread_rng();
            
            // Randomly decide if peer is reachable (90% chance)
            let is_reachable = rng.gen_bool(0.9);
            
            if is_reachable {
                // Update peer latency (between 5 and 150ms)
                peer.latency = rng.gen_range(5..150);
                peer.last_seen = SystemTime::now();
                
                // Update peer status based on latency
                if peer.latency < 50 {
                    peer.status = PeerStatus::Connected;
                } else if peer.latency < 100 {
                    peer.status = PeerStatus::Syncing;
                } else {
                    peer.status = PeerStatus::Connected; // Still connected but high latency
                }
                
                // Record ping stats for this peer
                if !self.ping_stats.contains_key(&peer.id) {
                    self.ping_stats.insert(peer.id.clone(), Vec::new());
                }
                
                let stats = self.ping_stats.get_mut(&peer.id).unwrap();
                stats.push(peer.latency);
                
                // Keep only the last 10 pings
                if stats.len() > 10 {
                    stats.remove(0);
                }
            } else {
                // Peer is unreachable
                peer.status = PeerStatus::Unreachable;
                peer.latency = 0;
            }
            
            statuses_updated += 1;
            trace!("Updated peer {}: status={:?}, latency={}ms", peer.id, peer.status, peer.latency);
        }
        
        // Simulate some network traffic to update bandwidth usage
        let traffic = rng.gen_range(1024..10240); // 1KB to 10KB
        self.update_bandwidth_usage(traffic);
        
        debug!("Finished pinging {} peers", statuses_updated);
        Ok(())
    }
    
    pub async fn ping_peer(&mut self, peer_id: &str, count: u8) -> Result<Vec<u64>, String> {
        let peer = self.peers.get(peer_id).ok_or(format!("Peer {} not found", peer_id))?;
        
        debug!("Pinging peer {} {} times", peer_id, count);
        let mut results = Vec::with_capacity(count as usize);
        let mut rng = rand::thread_rng();
        
        for i in 0..count {
            // Simulate ping result based on peer's current status
            let latency = match peer.status {
                PeerStatus::Connected => rng.gen_range(5..50),
                PeerStatus::Syncing => rng.gen_range(50..100),
                PeerStatus::Disconnected | PeerStatus::Unreachable => 0,
            };
            
            if latency > 0 {
                results.push(latency);
            }
            
            // Simulate network traffic
            self.update_bandwidth_usage(64); // 64 bytes for ping packet
            
            trace!("Ping {}/{} to peer {}: {}ms", i+1, count, peer_id, latency);
        }
        
        debug!("Completed pinging peer {}: {} successful pings", peer_id, results.len());
        Ok(results)
    }
    
    pub fn get_peer_stats(&self, peer_id: &str) -> Result<(f64, f64, u64), String> {
        let stats = self.ping_stats.get(peer_id).ok_or(format!("No stats for peer {}", peer_id))?;
        
        if stats.is_empty() {
            return Err("No ping statistics available".to_string());
        }
        
        // Calculate mean latency
        let sum: u64 = stats.iter().sum();
        let mean = sum as f64 / stats.len() as f64;
        
        // Calculate standard deviation
        let variance = stats.iter()
            .map(|&value| {
                let diff = mean - value as f64;
                diff * diff
            })
            .sum::<f64>() / stats.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // Get max latency
        let max = *stats.iter().max().unwrap_or(&0);
        
        Ok((mean, std_dev, max))
    }
}

impl NetworkingOperations for NetworkManager {
    fn start(&mut self) -> Result<(), String> {
        self.start()
    }

    fn stop(&mut self) -> Result<(), String> {
        info!("Stopping network connections");
        // Reset message sender to effectively stop processing
        self.message_sender = None;
        Ok(())
    }

    fn connect(&mut self, address: &str) -> Result<(), String> {
        info!("Connecting to network address: {}", address);
        
        // Generate a random peer ID
        let peer_id = format!("auto_{}", rand::thread_rng().gen::<u32>());
        self.add_peer(peer_id, address.to_string())
    }

    fn disconnect(&mut self, address: &str) -> Result<(), String> {
        info!("Disconnecting from network address: {}", address);
        
        // Find peer by address
        let peer_id = self.peers.iter()
            .find(|(_, peer)| peer.address == address)
            .map(|(id, _)| id.clone())
            .ok_or(format!("No peer found with address {}", address))?;
        
        self.remove_peer(&peer_id)
    }

    fn send_message(&mut self, address: &str, message: &[u8]) -> Result<(), String> {
        info!("Sending message to network address: {}", address);
        
        // Store message in cache
        self.cache.insert(address.to_string(), message.to_vec());
        
        // Update bandwidth usage stats
        self.update_bandwidth_usage(message.len() as u64);
        
        Ok(())
    }

    fn receive_message(&self, address: &str) -> Result<Vec<u8>, String> {
        info!("Receiving message from network address: {}", address);
        
        if let Some(message) = self.cache.get(address) {
            Ok(message.clone())
        } else {
            Ok(vec![])
        }
    }
    
    fn get_peer_latency(&self, peer_id: &str) -> Result<u64, String> {
        match self.peers.get(peer_id) {
            Some(peer) => Ok(peer.latency),
            None => Err(format!("Peer {} not found", peer_id)),
        }
    }
}