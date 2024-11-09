// src/websocket/handler.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use tokio::sync::mpsc::Sender;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::consensus::types::{ValidatorInfo, ConsensusRound, RoundStatus};
use crate::blockchain::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    ConsensusUpdate {
        round_number: u64,
        status: RoundStatus,
        coordinator: String,
        votes_count: usize,
        participation_rate: f64,
        remaining_time_ms: i64,
    },
    BlockFinalized {
        block_number: u64,
        transactions_count: usize,
        timestamp: u64,
        proposer: String,
    },
    ReputationUpdate {
        did: String,
        change: i64,
        new_total: i64,
        reason: String,
        context: String,
    },
    ValidatorUpdate {
        did: String,
        round: u64,
        status: String,
        reputation: i64,
        performance_score: f64,
    },
    CommandResponse {
        command: String,
        status: String,
        message: String,
        data: Option<serde_json::Value>,
    },
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    RegisterValidator {
        did: String,
        initial_reputation: i64,
    },
    SubmitTransaction {
        transaction: serde_json::Value,
    },
    QueryStatus,
    Subscribe {
        events: Vec<String>,
    },
}

#[derive(Clone)]
struct ConnectionInfo {
    tx: Sender<WebSocketMessage>,
    subscriptions: Vec<String>,
    connected_at: chrono::DateTime<Utc>,
    last_active: chrono::DateTime<Utc>,
}

pub struct WebSocketHandler {
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    connection_counter: Arc<AtomicU64>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        WebSocketHandler {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            connection_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn handle_connection(&self, ws: WebSocket, did: String) {
        println!("New WebSocket connection from: {}", did);
        let (mut ws_sink, mut ws_stream) = ws.split();
        let (tx, mut rx) = mpsc::channel(32);

        let connection_id = self.connection_counter.fetch_add(1, Ordering::SeqCst);

        // Register connection
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(did.clone(), ConnectionInfo {
                tx: tx.clone(),
                subscriptions: vec!["all".to_string()],
                connected_at: Utc::now(),
                last_active: Utc::now(),
            });
        }

        // Handle outgoing messages
        let connections = Arc::clone(&self.connections);
        let did_for_cleanup = did.clone();
        
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if ws_sink.send(Message::text(json)).await.is_err() {
                        break;
                    }
                }
            }
            let mut connections = connections.lock().unwrap();
            connections.remove(&did_for_cleanup);
            println!("Send task completed for connection {}", connection_id);
        });

        // Handle incoming messages
        let handler = Arc::new(self.clone());
        let did_for_receive = did.clone();

        let receive_task = tokio::spawn(async move {
            while let Some(result) = ws_stream.next().await {
                match result {
                    Ok(message) => {
                        if let Ok(text) = message.to_str() {
                            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                                if let Err(e) = handle_client_message(handler.clone(), &did_for_receive, client_msg).await {
                                    println!("Error handling message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("WebSocket error from {}: {}", did_for_receive, e);
                        break;
                    }
                }
            }
        });

        tokio::select! {
            _ = send_task => println!("Send task completed for {}", did),
            _ = receive_task => println!("Receive task completed for {}", did),
        }
    }

    fn broadcast_message(&self, message: WebSocketMessage) {
        // Create a vector of all transmitters while holding the lock
        let txs: Vec<_> = {
            let connections = self.connections.lock().unwrap();
            connections.values()
                .map(|info| info.tx.clone())
                .collect()
        }; // Lock is dropped here

        // Send messages without holding the lock
        for tx in txs {
            let message = message.clone();
            tokio::spawn(async move {
                if let Err(e) = tx.send(message).await {
                    println!("Failed to broadcast message: {}", e);
                }
            });
        }
    }

    async fn send_to_client(&self, did: &str, message: WebSocketMessage) -> Result<(), String> {
        let tx = {
            let connections = self.connections.lock().unwrap();
            connections.get(did)
                .map(|info| info.tx.clone())
        };

        if let Some(tx) = tx {
            tx.send(message)
                .await
                .map_err(|e| format!("Failed to send message: {}", e))?;
        }
        Ok(())
    }

    pub fn broadcast_consensus_update(&self, round: &ConsensusRound) {
        let message = WebSocketMessage::ConsensusUpdate {
            round_number: round.round_number,
            status: round.status.clone(),
            coordinator: round.coordinator.clone(),
            votes_count: round.votes.len(),
            participation_rate: round.stats.participation_rate,
            remaining_time_ms: (round.timeout - Utc::now()).num_milliseconds().max(0),
        };
        self.broadcast_message(message);
    }

    pub fn broadcast_block_finalized(&self, block: &Block) {
        let message = WebSocketMessage::BlockFinalized {
            block_number: block.index,
            transactions_count: block.transactions.len(),
            timestamp: block.timestamp,
            proposer: block.proposer.clone(),
        };
        self.broadcast_message(message);
    }

    pub fn broadcast_reputation_update(
        &self,
        did: String,
        change: i64,
        new_total: i64,
        reason: String,
        context: String
    ) {
        let message = WebSocketMessage::ReputationUpdate {
            did,
            change,
            new_total,
            reason,
            context,
        };
        self.broadcast_message(message);
    }

    pub fn broadcast_validator_update(
        &self,
        validator: ValidatorInfo,
        round: u64,
        status: String
    ) {
        let message = WebSocketMessage::ValidatorUpdate {
            did: validator.did.clone(),
            round,
            status,
            reputation: validator.reputation,
            performance_score: validator.performance_score,
        };
        self.broadcast_message(message);
    }

    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    pub fn cleanup_inactive_connections(&self, timeout_seconds: i64) {
        let mut connections = self.connections.lock().unwrap();
        let now = Utc::now();
        connections.retain(|_, info| {
            (now - info.last_active).num_seconds() < timeout_seconds
        });
    }
}

async fn handle_client_message(
    handler: Arc<WebSocketHandler>,
    did: &str,
    message: ClientMessage
) -> Result<(), String> {
    match message {
        ClientMessage::Subscribe { events } => {
            let response = WebSocketMessage::CommandResponse {
                command: "subscribe".to_string(),
                status: "success".to_string(),
                message: format!("Subscribed to {} events", events.len()),
                data: Some(serde_json::json!({ "events": events })),
            };
            handler.send_to_client(did, response).await
        },
        _ => {
            let response = WebSocketMessage::Error {
                code: "UNSUPPORTED".to_string(),
                message: "Message type not supported".to_string(),
                details: None,
            };
            handler.send_to_client(did, response).await
        }
    }
}

impl Clone for WebSocketHandler {
    fn clone(&self) -> Self {
        WebSocketHandler {
            connections: Arc::clone(&self.connections),
            broadcast_tx: self.broadcast_tx.clone(),
            connection_counter: Arc::clone(&self.connection_counter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_management() {
        let handler = WebSocketHandler::new();
        assert_eq!(handler.connection_count(), 0);
    }

    #[test]
    fn test_message_serialization() {
        let message = WebSocketMessage::ConsensusUpdate {
            round_number: 1,
            status: RoundStatus::Voting,
            coordinator: "did:icn:test".to_string(),
            votes_count: 3,
            participation_rate: 0.75,
            remaining_time_ms: 5000,
        };

        let serialized = serde_json::to_string(&message).unwrap();
        assert!(!serialized.is_empty());
    }
}