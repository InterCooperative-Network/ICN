use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use serde::{Serialize, Deserialize};
use std::net::{SocketAddr, ToSocketAddrs};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use icn_p2p::{SDPManager, PublicKey};  // Use re-exported PublicKey

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
    pub async fn new(bind_addr: Option<&str>) -> Result<Self, String> {
        let (tx, _rx) = mpsc::channel(100);
        let mut layer = Self {
            gossip_peers: Vec::new(),
            event_tx: tx,
            sdp_manager: None,
            peers: HashMap::new(),
        };

        if let Some(addr) = bind_addr {
            layer.initialize(addr).await?;
        }

        Ok(layer)
    }

    pub async fn initialize(&mut self, bind_addr: &str) -> Result<(), String> {
        let addr = bind_addr.to_socket_addrs()
            .map_err(|e| format!("Invalid address: {}", e))?
            .next()
            .ok_or_else(|| "Failed to resolve address".to_string())?;

        match SDPManager::new(addr) {
            Ok(manager) => {
                self.sdp_manager = Some(Arc::new(Mutex::new(manager)));
                
                let manager_clone = self.sdp_manager.as_ref().unwrap().clone();
                tokio::spawn(async move {
                    let handler = |result: Result<(Vec<u8>, SocketAddr, String), String>| {
                        match result {
                            Ok((payload, _addr, sender)) => {  // Prefix unused addr with _
                                println!("Received message from {}: {:?}", sender, payload);
                            }
                            Err(e) => {
                                println!("Error handling message: {}", e);
                            }
                        }
                    };
                    
                    if let Err(e) = manager_clone.lock().await.start(handler).await {
                        eprintln!("SDP manager error: {}", e);
                    }
                });
                
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize SDP manager: {}", e))
        }
    }

    pub async fn register_peer(&mut self, peer_id: String, public_key: PublicKey, addrs: Vec<SocketAddr>) -> Result<(), String> {
        if let Some(manager) = &self.sdp_manager {
            let manager = manager.lock().await;
            for addr in addrs.clone() {
                manager.register(peer_id.clone(), addr, public_key).await;
            }
            self.peers.insert(peer_id.clone(), PeerInfo {
                id: peer_id,
                addresses: addrs,
                public_key: Some(public_key),
                transport: NetworkTransport::SDP,
            });
            Ok(())
        } else {
            Err("SDP manager not initialized".to_string())
        }
    }

    pub async fn send_message(&self, peer_id: &str, data: Vec<u8>, priority: u8) -> Result<(), String> {
        if let Some(manager) = &self.sdp_manager {
            manager.lock().await.send_message(peer_id, data, priority).await?;
        }
        Ok(())
    }

    pub fn get_public_key(&self) -> Option<PublicKey> {
        self.sdp_manager.as_ref().map(|sdp| {
            let guard = futures::executor::block_on(sdp.lock());
            guard.get_public_key()
        })
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
                    
                    if let Err(e) = manager.send_message(peer_id, bytes.to_vec(), priority).await {
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
}
