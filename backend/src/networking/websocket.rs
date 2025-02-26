use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use dashmap::DashMap;
use log::{info, error};

pub type WebSocketClients = Arc<DashMap<String, warp::ws::Sender>>;

pub trait WebSocketOperations {
    fn handle_websocket(&self, ws: WebSocket, clients: WebSocketClients);
    fn process_message(&self, msg: Message, clients: &WebSocketClients) -> Result<(), String>;
    fn broadcast_message(&self, msg: &Message, clients: &WebSocketClients);
}

pub struct WebSocketManager {
    cache: DashMap<String, String>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        WebSocketManager {
            cache: DashMap::new(),
        }
    }
}

impl WebSocketOperations for WebSocketManager {
    fn handle_websocket(&self, ws: WebSocket, clients: WebSocketClients) {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let client_id = uuid::Uuid::new_v4().to_string();
        clients.insert(client_id.clone(), ws_tx.clone());

        tokio::spawn(async move {
            while let Some(result) = ws_rx.next().await {
                match result {
                    Ok(msg) => {
                        if let Err(e) = self.process_message(msg, &clients).await {
                            error!("Error processing message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }

            clients.remove(&client_id);
        });
    }

    fn process_message(&self, msg: Message, clients: &WebSocketClients) -> Result<(), String> {
        if let Ok(text) = msg.to_str() {
            self.cache.insert(text.to_string(), text.to_string());
            self.broadcast_message(&Message::text(text), clients);
        }
        Ok(())
    }

    fn broadcast_message(&self, msg: &Message, clients: &WebSocketClients) {
        for client in clients.iter() {
            if let Err(e) = client.value().send(msg.clone()).await {
                error!("Error sending message to client: {}", e);
            }
        }
    }
}
