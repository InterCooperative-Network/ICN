use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use rand::Rng;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Clone)]
struct CloneableSecret(StaticSecret);

impl CloneableSecret {
    fn new() -> Self {
        Self(StaticSecret::random_from_rng(rand::thread_rng()))
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalingPacket {
    sender_id: String,
    payload: Vec<u8>,
    nonce: Vec<u8>,
}

pub struct SDPManager {
    socket: Arc<UdpSocket>,
    routes: Arc<RwLock<HashMap<String, SocketAddr>>>,
    secret: CloneableSecret,
    peer_keys: Arc<RwLock<HashMap<String, PublicKey>>>,
}

impl SDPManager {
    pub fn new(bind_addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        
        Ok(Self {
            socket: Arc::new(socket),
            routes: Arc::new(RwLock::new(HashMap::new())),
            secret: CloneableSecret::new(),
            peer_keys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register(&self, id: String, addr: SocketAddr, public_key: PublicKey) {
        self.routes.write().await.insert(id.clone(), addr);
        self.peer_keys.write().await.insert(id, public_key);
    }

    pub async fn unregister(&self, id: &str) {
        self.routes.write().await.remove(id);
        self.peer_keys.write().await.remove(id);
    }

    async fn handle_packet(&self, packet: SignalingPacket, src: SocketAddr) -> Result<(), String> {
        let routes = self.routes.read().await;
        let socket = self.socket.clone();

        if let Some(route) = routes.get(&packet.sender_id) {
            if route != &src {
                // Only forward if source matches registered address
                return Ok(());
            }

            let serialized = serde_json::to_vec(&packet)
                .map_err(|e| format!("Serialization error: {}", e))?;

            socket.send_to(&serialized, *route)
                .map_err(|e| format!("Send error: {}", e))?;
        }

        Ok(())
    }

    pub async fn establish_connection(&self, peer_id: &str) -> Result<Vec<u8>, String> {
        let public = self.secret.public_key();
        let nonce: [u8; 12] = rand::thread_rng().gen();
        
        let routes = self.routes.read().await;
        if !routes.contains_key(peer_id) {
            return Err("Peer not found".to_string());
        }

        let packet = SignalingPacket {
            sender_id: "server".to_string(),
            payload: public.as_bytes().to_vec(),
            nonce: nonce.to_vec(),
        };

        let serialized = serde_json::to_vec(&packet)
            .map_err(|e| format!("Serialization error: {}", e))?;

        if let Some(route) = routes.get(peer_id) {
            self.socket.send_to(&serialized, *route)
                .map_err(|e| format!("Send error: {}", e))?;
        }

        Ok(nonce.to_vec())
    }

    pub async fn send_message(&self, peer_id: &str, data: Vec<u8>, _priority: u8) -> Result<(), String> {
        let routes = self.routes.read().await;
        
        if let Some(route) = routes.get(peer_id) {
            let packet = SignalingPacket {
                sender_id: "server".to_string(),
                payload: data,
                nonce: rand::thread_rng().gen::<[u8; 12]>().to_vec(),
            };

            let serialized = serde_json::to_vec(&packet)
                .map_err(|e| format!("Serialization error: {}", e))?;

            self.socket.send_to(&serialized, *route)
                .map_err(|e| format!("Send error: {}", e))?;
        } else {
            return Err(format!("No route found for peer {}", peer_id));
        }

        Ok(())
    }

    pub async fn start<F>(&self, handler: F) -> Result<(), String> 
    where
        F: Fn(Result<(Vec<u8>, SocketAddr, String), String>) + Send + Sync + 'static
    {
        let socket = self.socket.clone();
        let mut buf = vec![0u8; 65536];
        let handler = Arc::new(handler);

        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let data = &buf[..len];
                    let handler = handler.clone();
                    
                    match serde_json::from_slice::<SignalingPacket>(data) {
                        Ok(packet) => {
                            if let Err(e) = self.handle_packet(packet.clone(), src).await {
                                handler(Err(format!("Packet handling error: {}", e)));
                                continue;
                            }
                            handler(Ok((packet.payload, src, packet.sender_id)));
                        }
                        Err(e) => {
                            handler(Err(format!("Parse error: {}", e)));
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    return Err(format!("Recv error: {}", e));
                }
            }
        }
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.secret.public_key()
    }
}
