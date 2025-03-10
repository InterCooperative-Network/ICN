use std::collections::HashMap;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::time::Instant;
use log::info;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub enum PeerStatus {
    Connected,
    Disconnected,
    Syncing,
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub status: PeerStatus,
    pub latency: u64,
    pub connected_since: SystemTime,
}

#[derive(Debug)]
pub enum Message {
    Block { hash: String, data: Vec<u8> },
    Transaction { hash: String, data: Vec<u8> },
    Proposal { id: String, data: Vec<u8> },
    Vote { proposal_id: String, voter: String, approve: bool },
    Identity { did: String, data: Vec<u8> },
    Reputation { did: String, score: i64 },
    Ping,
    Pong,
}

pub trait NetworkingOperations {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn connect(&mut self, address: &str) -> Result<(), String>;
    fn disconnect(&mut self, address: &str) -> Result<(), String>;
    fn send_message(&mut self, address: &str, message: &[u8]) -> Result<(), String>;
    fn receive_message(&self, address: &str) -> Result<Vec<u8>, String>;
}

pub struct NetworkManager {
    peers: HashMap<String, Peer>,
    message_sender: Option<Sender<Message>>,
    max_peers: usize,
    network_key: Vec<u8>,
    bandwidth_usage: f32,
    last_bandwidth_update: Instant,
    bytes_transferred: u64,
    cache: HashMap<String, Vec<u8>>,
}

impl NetworkManager {
    pub fn new(max_peers: usize) -> Self {
        let network_key = vec![0u8; 32]; // In a real application, this would be a proper crypto key
        
        Self {
            peers: HashMap::new(),
            message_sender: None,
            max_peers,
            network_key,
            bandwidth_usage: 0.0,
            last_bandwidth_update: Instant::now(),
            bytes_transferred: 0,
            cache: HashMap::new(),
        }
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel(100);
        self.message_sender = Some(sender);
        
        // Start background task for processing messages
        let receiver_handle = tokio::spawn(async move {
            Self::process_messages(receiver).await;
        });
        
        Ok(())
    }
    
    async fn process_messages(mut receiver: Receiver<Message>) {
        while let Some(message) = receiver.recv().await {
            match message {
                Message::Block { hash, data: _ } => {
                    println!("Received block with hash: {}", hash);
                },
                Message::Transaction { hash, data: _ } => {
                    println!("Received transaction with hash: {}", hash);
                },
                Message::Proposal { id, data: _ } => {
                    println!("Received proposal with id: {}", id);
                },
                Message::Vote { proposal_id, voter, approve } => {
                    println!("Received vote on proposal {} from {}: {}", proposal_id, voter, approve);
                },
                Message::Identity { did, data: _ } => {
                    println!("Received identity for DID: {}", did);
                },
                Message::Reputation { did, score } => {
                    println!("Received reputation update for DID: {}, new score: {}", did, score);
                },
                Message::Ping => {
                    println!("Received ping");
                },
                Message::Pong => {
                    println!("Received pong");
                },
            }
        }
    }
    
    pub fn add_peer(&mut self, id: String, address: String) -> Result<(), String> {
        if self.peers.len() >= self.max_peers {
            return Err("Maximum number of peers reached".to_string());
        }
        
        let peer = Peer {
            id: id.clone(),
            address,
            status: PeerStatus::Connected,
            latency: 0,
            connected_since: SystemTime::now(),
        };
        
        self.peers.insert(id, peer);
        Ok(())
    }
    
    pub fn remove_peer(&mut self, id: &str) -> Result<(), String> {
        if self.peers.remove(id).is_none() {
            return Err("Peer not found".to_string());
        }
        Ok(())
    }
    
    pub fn get_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    pub fn get_connected_peer_count(&self) -> u32 {
        self.peers.values()
            .filter(|p| matches!(p.status, PeerStatus::Connected))
            .count() as u32
    }

    pub fn get_average_latency(&self) -> u32 {
        let connected_peers: Vec<_> = self.peers.values()
            .filter(|p| matches!(p.status, PeerStatus::Connected))
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
            // Calculate MB/s
            self.bandwidth_usage = (self.bytes_transferred as f32) / (1024.0 * 1024.0) / elapsed.as_secs_f32();
            self.bytes_transferred = 0;
            self.last_bandwidth_update = Instant::now();
        }
    }
    
    pub fn get_bandwidth_usage(&self) -> f32 {
        self.bandwidth_usage
    }
    
    pub fn send_message(&self, peer_id: &str, message: Message) -> Result<(), String> {
        if !self.peers.contains_key(peer_id) {
            return Err("Peer not found".to_string());
        }
        
        let sender = self.message_sender.as_ref().ok_or("Network not started")?;
        
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            sender_clone.send(message).await.unwrap();
        });
        
        Ok(())
    }
    
    pub async fn broadcast_message(&self, message: Message) -> Result<(), String> {
        let sender = self.message_sender.as_ref().ok_or("Network not started")?;
        
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            sender_clone.send(message).await.unwrap();
        });
        
        Ok(())
    }
    
    pub async fn ping_all_peers(&mut self) -> Result<(), String> {
        for peer in self.peers.values_mut() {
            // In a real implementation, this would actually ping each peer
            // For testing, we just update latency with a random value
            peer.latency = rand::random::<u64>() % 100;
            peer.status = if peer.latency < 50 { PeerStatus::Connected } else { PeerStatus::Disconnected };
        }
        
        Ok(())
    }
}

impl NetworkingOperations for NetworkManager {
    fn start(&mut self) -> Result<(), String> {
        info!("Starting network connections");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        info!("Stopping network connections");
        Ok(())
    }

    fn connect(&mut self, address: &str) -> Result<(), String> {
        info!("Connecting to network address: {}", address);
        Ok(())
    }

    fn disconnect(&mut self, address: &str) -> Result<(), String> {
        info!("Disconnecting from network address: {}", address);
        Ok(())
    }

    fn send_message(&mut self, address: &str, message: &[u8]) -> Result<(), String> {
        info!("Sending message to network address: {}", address);
        self.cache.insert(address.to_string(), message.to_vec());
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
}
