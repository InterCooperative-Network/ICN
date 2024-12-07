use crate::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use icn_types::DID;
use tokio::sync::Mutex;
use warp::ws::{Message as WsMessage, WebSocket};
use std::{collections::HashMap, sync::Arc};

pub struct ConnectionInfo {
    socket: WebSocket,
    connected_at: chrono::DateTime<chrono::Utc>,
    last_active: chrono::DateTime<chrono::Utc>,
}

pub struct WebSocketHandler {
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_connection(&self, ws: WebSocket, did: String) {
        let connections = self.connections.clone();
        
        // Create connection info
        let connection = ConnectionInfo {
            socket: ws,
            connected_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };
        
        // Store connection
        connections.lock().await.insert(did.clone(), connection);
    }
}
