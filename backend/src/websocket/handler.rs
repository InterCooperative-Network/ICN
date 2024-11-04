// src/websocket/handler.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};
use serde::{Serialize, Deserialize};
use futures_util::{StreamExt, SinkExt};

use crate::consensus::RoundStatus;
use crate::consensus::types::{ValidatorInfo, ConsensusRound};
use crate::blockchain::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Error {
        code: String,
        message: String,
    }
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
        let (mut tx, mut rx) = ws.split();
        let (broadcast_tx, mut broadcast_rx) = broadcast::channel(100);
        
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(did.clone(), broadcast_tx);
        }

        let did_clone = did.clone();
        tokio::spawn(async move {
            while let Some(result) = rx.next().await {
                match result {
                    Ok(msg) => {
                        println!("Received message from {}: {:?}", did_clone, msg);
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(e) = tx.send(Message::text(json)).await {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
            }
        }

        let mut connections = self.connections.lock().unwrap();
        connections.remove(&did);
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

    pub fn broadcast_block_finalized(&self, block: &Block, _coordinator: String) {
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

    pub fn broadcast_error(&self, code: String, message: String) {
        let msg = WebSocketMessage::Error { code, message };
        self.broadcast_message(msg);
    }

    fn broadcast_message(&self, msg: WebSocketMessage) {
        if let Ok(connections) = self.connections.lock() {
            for (_, sender) in connections.iter() {
                let _ = sender.send(msg.clone());
            }
        }
    }
}

// Tests for WebSocket Handler
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_handler_creation() {
        let handler = WebSocketHandler::new();
        assert!(handler.connections.lock().unwrap().is_empty());
    }

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::ConsensusUpdate {
            round_number: 1,
            status: RoundStatus::Voting,
            coordinator: "did:icn:1".to_string(),
            votes_count: 3,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        assert!(!serialized.is_empty());
    }
}