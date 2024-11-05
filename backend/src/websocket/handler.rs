// src/websocket/handler.rs

// Previous imports remain the same
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};
use serde::{Serialize, Deserialize};
use futures_util::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde_json::json;

use crate::consensus::RoundStatus;
use crate::consensus::types::{ValidatorInfo, ConsensusRound};
use crate::blockchain::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    ConsensusUpdate {
        round_number: u64,
        status: RoundStatus,
        coordinator: String,
        votes_count: usize,
    },
    BlockFinalized {
        block_number: u64,
        transactions_count: usize,
        timestamp: u64,
    },
    ReputationUpdate {
        did: String,
        change: i64,
        new_total: i64,
        reason: String,
    },
    ValidatorUpdate {
        did: String,
        round: u64,
        status: String,
        reputation: i64,
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
    }
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
    ProposeBlock {
        block: serde_json::Value,
    },
    SubmitVote {
        vote: serde_json::Value,
    },
    QueryReputation {
        did: String,
    },
}

pub struct WebSocketHandler {
    connections: Arc<Mutex<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        WebSocketHandler {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_connection(&self, ws: WebSocket, did: String) {
        println!("New WebSocket connection from: {}", did);
        
        let (ws_sink, ws_stream) = ws.split();
        let (tx, rx) = mpsc::channel(32);
        
        let (broadcast_tx, _) = broadcast::channel(100);
        
        // Register the connection
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(did.clone(), broadcast_tx);
            println!("Registered connection for: {}", did);
        }

        // Spawn the sender task
        let sender_task = tokio::spawn(Self::handle_sending(ws_sink, rx));
        
        // Spawn the receiver task
        let receiver_task = tokio::spawn(Self::handle_receiving(ws_stream, did.clone(), tx.clone()));

        // Send welcome message
        let welcome_msg = WebSocketMessage::CommandResponse {
            command: "connect".to_string(),
            status: "success".to_string(),
            message: format!("Welcome {}! Connected successfully.", did),
            data: None,
        };
        
        let _ = tx.send(welcome_msg).await;

        // Wait for either task to finish
        tokio::select! {
            _ = sender_task => println!("Sender task completed for {}", did),
            _ = receiver_task => println!("Receiver task completed for {}", did),
        }

        // Cleanup
        let mut connections = self.connections.lock().unwrap();
        connections.remove(&did);
        println!("Connection cleaned up for: {}", did);
    }

    async fn handle_sending(
        mut ws_sink: SplitSink<WebSocket, Message>,
        mut rx: mpsc::Receiver<WebSocketMessage>,
    ) {
        while let Some(message) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if let Err(e) = ws_sink.send(Message::text(json)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_receiving(
        mut ws_stream: SplitStream<WebSocket>,
        did: String,
        tx: mpsc::Sender<WebSocketMessage>,
    ) {
        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_str() {
                        println!("Received message from {}: {}", did, text);
                        
                        let response = match serde_json::from_str::<ClientMessage>(text) {
                            Ok(client_msg) => match client_msg {
                                ClientMessage::RegisterValidator { did, initial_reputation } => {
                                    WebSocketMessage::CommandResponse {
                                        command: "register_validator".to_string(),
                                        status: "success".to_string(),
                                        message: format!("Validator {} registered with {} reputation", did, initial_reputation),
                                        data: Some(json!({
                                            "did": did,
                                            "reputation": initial_reputation
                                        })),
                                    }
                                },
                                ClientMessage::SubmitTransaction { transaction } => {
                                    WebSocketMessage::CommandResponse {
                                        command: "submit_transaction".to_string(),
                                        status: "success".to_string(),
                                        message: "Transaction submitted successfully".to_string(),
                                        data: Some(transaction),
                                    }
                                },
                                ClientMessage::QueryStatus => {
                                    WebSocketMessage::CommandResponse {
                                        command: "query_status".to_string(),
                                        status: "success".to_string(),
                                        message: "Current blockchain status".to_string(),
                                        data: Some(json!({
                                            "height": 1,
                                            "validators": 1,
                                            "pending_transactions": 0
                                        })),
                                    }
                                },
                                ClientMessage::ProposeBlock { block } => {
                                    WebSocketMessage::CommandResponse {
                                        command: "propose_block".to_string(),
                                        status: "success".to_string(),
                                        message: "Block proposed successfully".to_string(),
                                        data: Some(block),
                                    }
                                },
                                ClientMessage::SubmitVote { vote } => {
                                    WebSocketMessage::CommandResponse {
                                        command: "submit_vote".to_string(),
                                        status: "success".to_string(),
                                        message: "Vote submitted successfully".to_string(),
                                        data: Some(vote),
                                    }
                                },
                                ClientMessage::QueryReputation { did } => {
                                    WebSocketMessage::CommandResponse {
                                        command: "query_reputation".to_string(),
                                        status: "success".to_string(),
                                        message: format!("Current reputation for {}", did),
                                        data: Some(json!({
                                            "did": did,
                                            "reputation": 100,
                                            "last_updated": chrono::Utc::now().timestamp()
                                        })),
                                    }
                                },
                            },
                            Err(e) => WebSocketMessage::Error {
                                code: "INVALID_MESSAGE".to_string(),
                                message: format!("Failed to parse message: {}", e),
                            },
                        };

                        if let Err(e) = tx.send(response).await {
                            eprintln!("Error sending response through channel: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error from {}: {}", did, e);
                    break;
                }
            }
        }
    }

    pub fn broadcast_consensus_update(&self, round: &ConsensusRound) {
        let msg = WebSocketMessage::ConsensusUpdate {
            round_number: round.round_number,
            status: round.status.clone(),
            coordinator: round.coordinator.clone(),
            votes_count: round.votes.len(),
        };
        self.broadcast_message(msg);
    }

    // Removed unused coordinator parameter
    pub fn broadcast_block_finalized(&self, block: &Block) {
        let msg = WebSocketMessage::BlockFinalized {
            block_number: block.index,
            transactions_count: block.transactions.len(),
            timestamp: block.timestamp as u64,
        };
        self.broadcast_message(msg);
    }

    pub fn broadcast_reputation_update(&self, did: String, change: i64, new_total: i64, reason: String) {
        let msg = WebSocketMessage::ReputationUpdate {
            did,
            change,
            new_total,
            reason,
        };
        self.broadcast_message(msg);
    }

    pub fn broadcast_validator_update(&self, validator: ValidatorInfo, round: u64, status: String) {
        let msg = WebSocketMessage::ValidatorUpdate {
            did: validator.did,
            round,
            status,
            reputation: validator.reputation,
        };
        self.broadcast_message(msg);
    }

    fn broadcast_message(&self, msg: WebSocketMessage) {
        if let Ok(connections) = self.connections.lock() {
            println!("Broadcasting message to {} connections", connections.len());
            let msg_str = serde_json::to_string(&msg).unwrap_or_default();
            println!("Broadcast content: {}", msg_str);
            
            for (did, sender) in connections.iter() {
                if let Err(e) = sender.send(msg.clone()) {
                    eprintln!("Failed to broadcast to {}: {}", did, e);
                } else {
                    println!("Successfully broadcast to {}", did);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::CommandResponse {
            command: "test".to_string(),
            status: "success".to_string(),
            message: "Test message".to_string(),
            data: Some(json!({"test": "data"})),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        assert!(!serialized.is_empty());
        println!("Serialized message: {}", serialized);
    }
}