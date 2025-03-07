use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use icn_p2p::{SDPManager, PublicKey};

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkEvent {
    VoteUpdate { proposal_id: String, votes: u32 },
    ReputationChange { member_id: String, score: i32 },
    MembershipUpdate { members: Vec<String> },
    StateChange { key: String, value: String },
}

#[derive(Debug, Clone)]
pub enum NetworkTransport {
    WebSocket,
    SDP,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub addresses: Vec<SocketAddr>,
    pub public_key: Option<PublicKey>,
    pub transport: NetworkTransport,
}

pub struct NetworkLayer {
    gossip_peers: Vec<String>,
    event_tx: mpsc::Sender<NetworkEvent>,
    sdp_manager: Option<Arc<Mutex<SDPManager>>>,
    peers: HashMap<String, PeerInfo>,
}

impl NetworkLayer {
    pub async fn new() -> Self {
        let (tx, _rx) = mpsc::channel(100);
        Self {
            gossip_peers: Vec::new(),
            event_tx: tx,
            sdp_manager: None,
            peers: HashMap::new(),
        }
    }

    pub async fn init_sdp(&mut self, bind_addr: &str) -> Result<(), String> {
        match SDPManager::new(bind_addr) {
            Ok(manager) => {
                let manager = Arc::new(Mutex::new(manager));
                let manager_clone = manager.clone();
                let event_tx = self.event_tx.clone();
                
                let handler = move |data: Vec<u8>, _src: SocketAddr| {
                    if let Ok(event) = serde_json::from_slice::<NetworkEvent>(&data) {
                        let _ = event_tx.clone().try_send(event);
                    }
                };
                
                manager_clone.lock().await.start_receiver(handler).await?;
                self.sdp_manager = Some(manager);
                Ok(())
            },
            Err(e) => Err(format!("Failed to initialize SDP: {}", e)),
        }
    }

    pub async fn add_sdp_peer(&mut self, peer_id: String, public_key: PublicKey, addresses: Vec<SocketAddr>) {
        let peer = PeerInfo {
            id: peer_id.clone(),
            addresses: addresses.clone(),
            public_key: Some(public_key),
            transport: NetworkTransport::SDP,
        };
        
        self.peers.insert(peer_id.clone(), peer);
        
        if let Some(sdp) = &self.sdp_manager {
            let mut manager = sdp.lock().await;
            manager.register_peer(peer_id, public_key, addresses);
        }
    }

    pub async fn broadcast_event(&self, event: NetworkEvent) -> Result<(), String> {
        let json = serde_json::to_string(&event)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let bytes = json.as_bytes();
        
        if let Some(sdp) = &self.sdp_manager {
            let manager = sdp.lock().await;
            
            for (peer_id, peer_info) in &self.peers {
                if let NetworkTransport::SDP = peer_info.transport {
                    let priority = match &event {
                        NetworkEvent::VoteUpdate { .. } => 8,
                        NetworkEvent::StateChange { .. } => 6,
                        _ => 4,
                    };
                    
                    if let Err(e) = manager.send_message(peer_id, bytes, priority).await {
                        eprintln!("Failed to send SDP message to {}: {}", peer_id, e);
                    }
                }
            }
        }

        // Legacy WebSocket broadcast
        for peer in &self.gossip_peers {
            if let Some(peer_info) = self.peers.get(peer) {
                if let NetworkTransport::WebSocket = peer_info.transport {
                    // WebSocket send logic here
                }
            }
        }

        Ok(())
    }

    pub async fn handle_websocket_connection(&self, mut stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) {
        while let Some(msg) = stream.next().await {
            if let Ok(msg) = msg {
                if let Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str(&text) {
                        let _ = self.event_tx.send(event).await;
                    }
                }
            }
        }
    }
    
    pub async fn get_public_key(&self) -> Option<PublicKey> {
        if let Some(sdp) = &self.sdp_manager {
            Some(sdp.lock().await.get_public_key().await)
        } else {
            None
        }
    }
}
