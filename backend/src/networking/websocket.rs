use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use dashmap::DashMap;

pub type WebSocketClients = Arc<DashMap<String, warp::ws::Sender>>;

pub async fn handle_websocket(ws: WebSocket, clients: WebSocketClients) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let client_id = uuid::Uuid::new_v4().to_string();
    clients.insert(client_id.clone(), ws_tx.clone());

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Err(e) = process_message(msg, &clients).await {
                    eprintln!("Error processing message: {}", e);
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    clients.remove(&client_id);
}

async fn process_message(msg: Message, clients: &WebSocketClients) -> Result<(), String> {
    if let Ok(text) = msg.to_str() {
        broadcast_message(&Message::text(text), clients).await;
    }
    Ok(())
}

pub async fn broadcast_message(msg: &Message, clients: &WebSocketClients) {
    for client in clients.iter() {
        if let Err(e) = client.value().send(msg.clone()).await {
            eprintln!("Error sending message to client: {}", e);
        }
    }
}
