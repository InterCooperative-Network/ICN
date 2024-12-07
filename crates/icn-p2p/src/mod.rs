// src/network/mod.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::blockchain::Block;
use crate::consensus::ConsensusRound;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    // Peer discovery and management
    PeerAnnouncement {
        node_id: String,
        address: String,
    },
    PeerList {
        peers: Vec<(String, String)>,
    },
    
    // Consensus messages
    ConsensusProposal {
        round: ConsensusRound,
        block: Block,
    },
    ConsensusVote {
        round_number: u64,
        voter: String,
        approved: bool,
        signature: String,
    },
    
    // Block and transaction propagation
    NewBlock(Block),
    TransactionAnnouncement {
        tx_hash: String,
        from: String,
    },
    
    // Federation protocol messages
    FederationJoinRequest {
        cooperative_id: String,
        federation_id: String,
    },
    FederationResponse {
        approved: bool,
        federation_id: String,
        metadata: HashMap<String, String>,
    },
    
    // Cooperative synchronization
    ResourceStateSync {
        cooperative_id: String,
        resource_updates: HashMap<String, i64>,
    },
    
    // Status and health checks
    Ping(u64),
    Pong(u64),
}

pub struct NetworkHandler {
    node_id: String,
    peers: Arc<Mutex<HashMap<String, PeerConnection>>>,
    message_tx: mpsc::Sender<NetworkMessage>,
    message_rx: mpsc::Receiver<NetworkMessage>,
    listener_address: String,
}

struct PeerConnection {
    address: String,
    tx: mpsc::Sender<Message>,
    last_seen: std::time::Instant,
    reputation: i64,
}

impl NetworkHandler {
    pub fn new(node_id: String, listener_address: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        NetworkHandler {
            node_id,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_tx: tx,
            message_rx: rx,
            listener_address,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let listener = TcpListener::bind(&self.listener_address)
            .await
            .map_err(|e| format!("Failed to bind listener: {}", e))?;
            
        println!("Network handler listening on: {}", self.listener_address);

        let peers = self.peers.clone();
        let node_id = self.node_id.clone();
        
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                println!("New connection from: {}", addr);
                
                let peer_handler = PeerHandler::new(
                    node_id.clone(),
                    peers.clone(),
                );
                
                tokio::spawn(async move {
                    if let Err(e) = peer_handler.handle_connection(stream).await {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
        });

        self.process_messages().await?;

        Ok(())
    }

    async fn process_messages(&mut self) -> Result<(), String> {
        while let Some(message) = self.message_rx.recv().await {
            match message {
                NetworkMessage::PeerAnnouncement { node_id, address } => {
                    self.handle_peer_announcement(node_id, address).await?;
                }
                NetworkMessage::NewBlock(block) => {
                    self.broadcast_block(block).await?;
                }
                NetworkMessage::ConsensusProposal { round, block } => {
                    self.broadcast_consensus_proposal(round, block).await?;
                }
                NetworkMessage::ConsensusVote { round_number, voter, approved, signature } => {
                    self.broadcast_consensus_vote(round_number, voter, approved, signature).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_peer_announcement(&mut self, peer_id: String, address: String) -> Result<(), String> {
        let mut peers = self.peers.lock().unwrap();
        
        if !peers.contains_key(&peer_id) {
            match self.connect_to_peer(&address).await {
                Ok(connection) => {
                    peers.insert(peer_id.clone(), connection);
                    println!("Connected to peer: {}", peer_id);
                }
                Err(e) => {
                    eprintln!("Failed to connect to peer {}: {}", peer_id, e);
                }
            }
        }
        
        Ok(())
    }

    async fn connect_to_peer(&self, address: &str) -> Result<PeerConnection, String> {
        let url = format!("ws://{}", address);
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("Failed to connect to peer: {}", e))?;
            
        let (sink, stream) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut sink = sink;
            while let Some(message) = rx.recv().await {
                if let Err(e) = sink.send(message).await {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        tokio::spawn(async move {
            let mut stream = stream;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_text() {
                            println!("Received message from peer: {}", text);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading message: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(PeerConnection {
            address: address.to_string(),
            tx,
            last_seen: std::time::Instant::now(),
            reputation: 0,
        })
    }

    async fn broadcast_block(&self, block: Block) -> Result<(), String> {
        let message = NetworkMessage::NewBlock(block);
        self.broadcast_message(&message).await
    }

    async fn broadcast_consensus_proposal(&self, round: ConsensusRound, block: Block) -> Result<(), String> {
        let message = NetworkMessage::ConsensusProposal { round, block };
        self.broadcast_message(&message).await
    }

    async fn broadcast_consensus_vote(
        &self,
        round_number: u64,
        voter: String,
        approved: bool,
        signature: String,
    ) -> Result<(), String> {
        let message = NetworkMessage::ConsensusVote {
            round_number,
            voter,
            approved,
            signature,
        };
        self.broadcast_message(&message).await
    }

    async fn broadcast_message(&self, message: &NetworkMessage) -> Result<(), String> {
        let message_json = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
        let peers = self.peers.lock().unwrap();
        
        for (peer_id, connection) in peers.iter() {
            if let Err(e) = connection.tx.send(Message::Text(message_json.clone())).await {
                eprintln!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }
        
        Ok(())
    }
}

struct PeerHandler {
    node_id: String,
    peers: Arc<Mutex<HashMap<String, PeerConnection>>>,
}

impl PeerHandler {
    fn new(
        node_id: String,
        peers: Arc<Mutex<HashMap<String, PeerConnection>>>,
    ) -> Self {
        PeerHandler {
            node_id,
            peers,
        }
    }

    async fn handle_connection(&self, stream: TcpStream) -> Result<(), String> {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .map_err(|e| format!("Failed to accept WebSocket connection: {}", e))?;
            
        let (_sink, mut stream) = ws_stream.split();
        
        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Ok(text) = msg.to_text() {
                        if let Ok(network_msg) = serde_json::from_str::<NetworkMessage>(text) {
                            self.handle_network_message(network_msg).await?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from WebSocket: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_network_message(&self, message: NetworkMessage) -> Result<(), String> {
        match message {
            NetworkMessage::PeerAnnouncement { node_id, address } => {
                println!("Received peer announcement from {} at {}", node_id, address);
            }
            NetworkMessage::NewBlock(block) => {
                println!("Received new block: {}", block.index);
            }
            NetworkMessage::ConsensusProposal { round, block: _ } => {
                println!("Received consensus proposal for round {}", round.round_number);
            }
            NetworkMessage::ConsensusVote { round_number, voter, approved: _, signature: _ } => {
                println!("Received consensus vote from {} for round {}", voter, round_number);
            }
            _ => {
                println!("Received other network message type");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_handler() {
        let handler = NetworkHandler::new(
            "test_node".to_string(),
            "127.0.0.1:0".to_string(),
        );
        assert_eq!(handler.node_id, "test_node");
    }
}