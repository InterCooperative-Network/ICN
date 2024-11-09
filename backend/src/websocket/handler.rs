// src/websocket/handler.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::consensus::types::{ValidatorInfo, ConsensusRound, RoundStatus};
use crate::blockchain::Block;
use crate::reputation::ReputationChange;

/// Represents different types of WebSocket messages that can be sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Updates about consensus rounds
    ConsensusUpdate {
        round_number: u64,
        status: RoundStatus,
        coordinator: String,
        votes_count: usize,
        participation_rate: f64,
        remaining_time_ms: i64,
    },

    /// Notification of block finalization
    BlockFinalized {
        block_number: u64,
        transactions_count: usize,
        timestamp: u64,
        proposer: String,
        size_bytes: u64,
    },

    /// Updates to reputation scores
    ReputationUpdate {
        did: String,
        change: i64,
        new_total: i64,
        reason: String,
        context: String,
    },

    /// Updates about validator status
    ValidatorUpdate {
        did: String,
        round: u64,
        status: String,
        reputation: i64,
        performance_score: f64,
    },

    /// Generic command responses
    CommandResponse {
        command: String,
        status: String,
        message: String,
        data: Option<serde_json::Value>,
    },

    /// Error messages
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
}

/// Messages that can be received from clients
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Register as a validator
    RegisterValidator {
        did: String,
        initial_reputation: i64,
    },

    /// Submit a transaction
    SubmitTransaction {
        transaction: serde_json::Value,
    },

    /// Query current network status
    QueryStatus,

    /// Propose a new block
    ProposeBlock {
        block: serde_json::Value,
    },

    /// Submit a vote on current round
    SubmitVote {
        vote: serde_json::Value,
    },

    /// Query reputation for a DID
    QueryReputation {
        did: String,
    },

    /// Subscribe to specific event types
    Subscribe {
        events: Vec<String>,
    },
}

/// Manages WebSocket connections and message broadcasting
pub struct WebSocketHandler {
    /// Active connections mapped by DID
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
    
    /// Broadcast channel for system-wide messages
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

/// Information about an active connection
struct ConnectionInfo {
    /// Sender for this connection
    tx: mpsc::Sender<WebSocketMessage>,
    
    /// Subscribed event types
    subscriptions: Vec<String>,
    
    /// Connection timestamp
    connected_at: chrono::DateTime<Utc>,
    
    /// Last activity timestamp
    last_active: chrono::DateTime<Utc>,
}

impl WebSocketHandler {
    /// Creates a new WebSocket handler
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        
        WebSocketHandler {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Handles a new WebSocket connection
    pub async fn handle_connection(&self, ws: WebSocket, did: String) {
        println!("New WebSocket connection from: {}", did);

        let (mut ws_sink, mut ws_stream) = ws.split();
        let (tx, mut rx) = mpsc::channel(32);

        // Register the connection
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(did.clone(), ConnectionInfo {
                tx,
                subscriptions: vec!["all".to_string()],
                connected_at: Utc::now(),
                last_active: Utc::now(),
            });
            println!("Registered connection for: {}", did);
        }

        // Clone data for use within async tasks
        let connections_clone = self.connections.clone();
        let did_clone = did.clone();

        // Handle outgoing messages
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&message) {
                    if ws_sink.send(Message::text(json)).await.is_err() {
                        eprintln!("Error sending message to {}", did_clone);
                        break;
                    }
                }
            }

            // Clean up connection on exit
            let mut connections = connections_clone.lock().unwrap();
            connections.remove(&did_clone);
            println!("Connection closed for: {}", did_clone);
        });

        // Handle incoming messages
        let receive_task = tokio::spawn(async move {
            while let Some(result) = ws_stream.next().await {
                match result {
                    Ok(message) => {
                        if let Ok(text) = message.to_str() {
                            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                                self.handle_client_message(&did, client_msg).await;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error from {}: {}", did, e);
                        break;
                    }
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = send_task => println!("Send task completed for {}", did),
            _ = receive_task => println!("Receive task completed for {}", did),
        }
    }

    /// Handles messages received from clients
    async fn handle_client_message(&self, did: &str, message: ClientMessage) {
        match message {
            ClientMessage::RegisterValidator { did: validator_did, initial_reputation } => {
                self.send_to_client(did, WebSocketMessage::CommandResponse {
                    command: "register_validator".to_string(),
                    status: "success".to_string(),
                    message: format!("Validator {} registered", validator_did),
                    data: Some(serde_json::json!({
                        "did": validator_did,
                        "reputation": initial_reputation
                    })),
                }).await;
            }

            ClientMessage::Subscribe { events } => {
                if let Some(connection) = self.connections.lock().unwrap().get_mut(did) {
                    connection.subscriptions = events.clone();
                    self.send_to_client(did, WebSocketMessage::CommandResponse {
                        command: "subscribe".to_string(),
                        status: "success".to_string(),
                        message: format!("Subscribed to {} events", events.len()),
                        data: Some(serde_json::json!({ "events": events })),
                    }).await;
                }
            }

            // Handle other message types...
            _ => {
                self.send_to_client(did, WebSocketMessage::Error {
                    code: "UNSUPPORTED".to_string(),
                    message: "Message type not supported".to_string(),
                    details: None,
                }).await;
            }
        }
    }

    /// Sends a message to a specific client
    async fn send_to_client(&self, did: &str, message: WebSocketMessage) {
        if let Some(connection) = self.connections.lock().unwrap().get(did) {
            if let Err(e) = connection.tx.send(message).await {
                eprintln!("Error sending message to {}: {}", did, e);
            }
        }
    }

    /// Broadcasts consensus updates to all connected clients
    pub fn broadcast_consensus_update(&self, round: &ConsensusRound) {
        let message = WebSocketMessage::ConsensusUpdate {
            round_number: round.round_number,
            status: round.status.clone(),
            coordinator: round.coordinator.clone(),
            votes_count: round.votes.len(),
            participation_rate: round.stats.participation_rate,
            remaining_time_ms: (round.timeout - Utc::now())
                .num_milliseconds()
                .max(0),
        };

        self.broadcast_message(message);
    }

    /// Broadcasts block finalization to all connected clients
    pub fn broadcast_block_finalized(&self, block: &Block) {
        let message = WebSocketMessage::BlockFinalized {
            block_number: block.index,
            transactions_count: block.transactions.len(),
            timestamp: block.timestamp,
            proposer: block.proposer.clone(),
            size_bytes: block.metadata.size,
        };

        self.broadcast_message(message);
    }

    /// Broadcasts reputation updates to all connected clients
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

    /// Broadcasts validator updates to all connected clients
    pub fn broadcast_validator_update(
        &self,
        validator: ValidatorInfo,
        round: u64,
        status: String
    ) {
        let message = WebSocketMessage::ValidatorUpdate {
            did: validator.did,
            round,
            status,
            reputation: validator.reputation,
            performance_score: validator.performance_score,
        };

        self.broadcast_message(message);
    }

    /// Broadcasts a message to all connected clients
    fn broadcast_message(&self, message: WebSocketMessage) {
        if let Ok(connections) = self.connections.lock() {
            for (did, connection) in connections.iter() {
                if let Err(e) = connection.tx.try_send(message.clone()) {
                    eprintln!("Failed to broadcast to {}: {}", did, e);
                }
            }
        }
    }

    /// Gets the number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    /// Cleans up inactive connections
    pub fn cleanup_inactive_connections(&self, timeout_seconds: i64) {
        let mut connections = self.connections.lock().unwrap();
        let now = Utc::now();
        
        connections.retain(|_, info| {
            (now - info.last_active).num_seconds() < timeout_seconds
        });
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

    // Additional tests...
}
