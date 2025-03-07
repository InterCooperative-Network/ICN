mod config;
mod core;
mod db;
mod federation;
mod notification;
mod reputation;
mod websocket;
mod middleware;
mod api;
mod networking;

use crate::config::Config;
use crate::core::{Core, TelemetryManager, PrometheusMetrics, Logger, TracingSystem, RuntimeManager};
use crate::db::Database;
use crate::federation::{FederationOperation, FederationTerms, FederationType};
use crate::notification::NotificationManager;
use crate::reputation::ReputationManager;
use crate::websocket::{WebSocketClients, handle_websocket, broadcast_message};
use tokio;
use log::{info, error};
use env_logger;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures_util::StreamExt;
use warp::{Filter, ws::{WebSocket, Message}};
use dashmap::DashMap;
use sqlx::PgPool;
use thiserror::Error;
use reqwest::Client;
use tokio::signal;
use async_trait::async_trait;
use crate::storage::{StorageManager, StorageBackend, StorageResult, StorageError};
use warp::http::Method;
use warp::cors::Cors;
use crate::db::create_pool;
use middleware::rate_limit::with_rate_limit;
use middleware::rate_limit::with_reputation_rate_limit;
use networking::p2p::{P2PManager, FederationEvent, GovernanceEvent, IdentityEvent, ReputationEvent};
use icn_crypto::KeyPair;
use icn_backend::api::ApiServer;
use icn_backend::core::Core;
use icn_backend::database::Database;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Serialize, Deserialize)]
struct Proposal {
    id: String,
    title: String,
    description: String,
    status: String,
    votes_for: i64,
    votes_against: i64,
    created_by: String,
    ends_at: String,
}

#[derive(Serialize, Deserialize)]
struct Vote {
    proposal_id: String,
    voter: String,
    approve: bool,
}

#[derive(Serialize, Deserialize)]
struct TokenizedResource {
    resource_id: String,
    owner: String,
    quantity: u64,
    price_per_unit: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv().ok();
    
    // Initialize logging
    env_logger::init();
    
    info!("Starting ICN backend...");
    
    // Get port from environment variable or default to 8080
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    // Create the core system
    let core = Core::new();
    
    // Start core services
    core.start().await?;
    
    info!("Core services started.");
    
    // Create API server with core services
    let api_server = ApiServer::new(
        port,
        core.blockchain_service.clone(),
        core.identity_service.clone(),
        core.governance_service.clone(),
    );
    
    info!("Starting API server on port {}", port);
    
    // Run the server (this will block until the server exits)
    api_server.run().await?;
    
    info!("API server stopped. Shutting down...");
    
    // Graceful shutdown
    core.shutdown().await?;
    
    info!("ICN backend stopped.");
    
    Ok(())
}

async fn handle_federation_operation(operation: FederationOperation, notification_manager: NotificationManager) -> Result<impl warp::Reply, warp::Rejection> {
    // Logic to handle federation operations
    let subject = match &operation {
        FederationOperation::InitiateFederation { federation_type, partner_id, terms } => {
            format!("Federation Initiated: {:?}", federation_type)
        }
        FederationOperation::JoinFederation { federation_id, commitment } => {
            format!("Joined Federation: {}", federation_id)
        }
        FederationOperation::LeaveFederation { federation_id, reason } => {
            format!("Left Federation: {}", federation_id)
        }
        FederationOperation::ProposeAction { federation_id, action_type, description, resources } => {
            format!("Action Proposed in Federation: {}", federation_id)
        }
        FederationOperation::VoteOnProposal { federation_id, proposal_id, approve, notes } => {
            format!("Vote on Federation Proposal: {}", proposal_id)
        }
        FederationOperation::ShareResources { federation_id, resource_type, amount, recipient_id } => {
            format!("Resources Shared in Federation: {}", federation_id)
        }
        FederationOperation::UpdateFederationTerms { federation_id, new_terms } => {
            format!("Federation Terms Updated: {}", federation_id)
        }
    };

    let body = format!("Federation operation executed: {:?}", operation);
    notification_manager.send_notification(&subject, &body).await;
    Ok(warp::reply::json(&operation))
}

async fn handle_query_shared_resources() -> Result<impl warp::Reply, warp::Rejection> {
    // Logic to handle querying shared resources
    let resources = vec![
        TokenizedResource {
            resource_id: "resource1".to_string(),
            owner: "did:icn:owner1".to_string(),
            quantity: 100,
            price_per_unit: 10.0,
        },
        TokenizedResource {
            resource_id: "resource2".to_string(),
            owner: "did:icn:owner2".to_string(),
            quantity: 200,
            price_per_unit: 20.0,
        },
    ];
    Ok(warp::reply::json(&resources))
}

struct DatabaseStorageBackend {
    pool: PgPool,
}

impl DatabaseStorageBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StorageBackend for DatabaseStorageBackend {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO storage (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key) DO UPDATE SET value = $2
            "#,
            key,
            value
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        let result = sqlx::query!(
            r#"
            SELECT value FROM storage WHERE key = $1
            "#,
            key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(result.value)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM storage WHERE key = $1
            "#,
            key
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM storage WHERE key = $1)
            "#,
            key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(result.exists.unwrap_or(false))
    }
}

fn main() {
    println!("ICN backend is running!");
}
