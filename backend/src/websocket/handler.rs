// backend/src/websocket/handler.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};
use serde::{Serialize, Deserialize};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;

use crate::blockchain::Blockchain;
use crate::consensus::RoundStatus;

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
    },
    Error {
        code: String,
        message: String,
    }
}

pub struct WebSocketHandler {
    connections: Arc<Mutex<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl WebSocketHandler {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        WebSocketHandler {
            connections: Arc::new(Mutex::new(HashMap::new())),
            blockchain,
        }
    }

    pub async fn handle_connection(&self, ws: WebSocket, did: String) {
        let (mut tx, mut rx) = ws.split();
        
        // Create a channel for this connection
        let (broadcast_tx, mut broadcast_rx) = broadcast::channel(100);
        
        // Store the sender
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(did.clone(), broadcast_tx);
        }

        // Handle incoming messages
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

        // Handle outgoing messages
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(e) = tx.send(Message::text(json)).await {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
            }
        }

        // Clean up connection
        let mut connections = self.connections.lock().unwrap();
        connections.remove(&did);
    }

    pub fn broadcast_consensus_update(&self, round_number: u64, status: RoundStatus, 
                                    coordinator: String, votes_count: usize) {
        let msg = WebSocketMessage::ConsensusUpdate {
            round_number,
            status,
            coordinator,
            votes_count,
        };
        self.broadcast_message(msg);
    }

    pub fn broadcast_block_finalized(&self, block_number: u64, 
                                   transactions_count: usize, timestamp: u64) {
        let msg = WebSocketMessage::BlockFinalized {
            block_number,
            transactions_count,
            timestamp,
        };
        self.broadcast_message(msg);
    }

    pub fn broadcast_reputation_update(&self, did: String, change: i64, new_total: i64) {
        let msg = WebSocketMessage::ReputationUpdate {
            did,
            change,
            new_total,
        };
        self.broadcast_message(msg);
    }

    fn broadcast_message(&self, msg: WebSocketMessage) {
        let connections = self.connections.lock().unwrap();
        for (_, sender) in connections.iter() {
            let _ = sender.send(msg.clone());
        }
    }
}