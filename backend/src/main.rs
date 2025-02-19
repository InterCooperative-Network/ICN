mod config;
mod core;
mod db;
mod federation;
mod notification;
mod reputation;
mod websocket;

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
async fn main() -> Result<(), AppError> {
    // Initialize logging with env_logger
    env_logger::init();

    // Load and validate configuration
    let config: Config = config::load_config().map_err(|e| AppError::ConfigError(e.to_string()))?;
    config.validate()?;
    
    // Set up database connection pool
    let db_pool = PgPool::connect(&config.database_url)
        .await
        .map_err(AppError::DatabaseError)?;

    // Initialize WebSocket clients using DashMap
    let websocket_clients: WebSocketClients = Arc::new(DashMap::new());

    // Initialize components
    let storage_manager = StorageManager::new(Box::new(DatabaseStorageBackend::new(db_pool.clone())));
    let network_manager = NetworkManager::new();
    let runtime_manager = RuntimeManager::new();
    let telemetry_manager = TelemetryManager::new(PrometheusMetrics, Logger, TracingSystem);
    let identity_manager = IdentityManager::new();
    let reputation_manager = ReputationManager::new(
        config.governance_decay_rate,
        config.resource_sharing_decay_rate,
        config.technical_contributions_decay_rate,
        config.decay_exemptions.clone(),
    );

    let notification_manager = NotificationManager::new(config.notification_email.clone(), config.notification_sms.clone());

    // Create core system
    let core = Core::new(
        Arc::new(storage_manager),
        Arc::new(network_manager),
        Arc::new(runtime_manager),
        Arc::new(telemetry_manager),
        Arc::new(identity_manager),
        Arc::new(reputation_manager),
    );

    // Start core system
    if let Err(e) = core.start().await {
        error!("Failed to start core system: {}", e);
        return Err(AppError::ConfigError(e));
    }

    // Set up WebSocket server
    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let websocket_clients = websocket_clients.clone();
            ws.on_upgrade(move |socket| handle_websocket(socket, websocket_clients))
        });

    // Set up Warp server
    let create_proposal = warp::path!("api" / "governance" / "proposals")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |proposal: Proposal| {
            let notification_manager = notification_manager.clone();
            let websocket_clients = websocket_clients.clone();
            async move {
                handle_create_proposal(proposal, notification_manager, websocket_clients, db_pool.clone()).await
            }
        });

    let vote_on_proposal = warp::path!("api" / "governance" / "proposals" / String / "vote")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |proposal_id: String, vote: Vote| {
            let notification_manager = notification_manager.clone();
            let websocket_clients = websocket_clients.clone();
            async move {
                handle_vote_on_proposal(proposal_id, vote, notification_manager, websocket_clients, db_pool.clone()).await
            }
        });

    let federation_routes = warp::path("api/federation")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |operation: FederationOperation| {
            let notification_manager = notification_manager.clone();
            async move {
                handle_federation_operation(operation, notification_manager).await
            }
        });

    let query_shared_resources = warp::path!("api" / "resources" / "query")
        .and(warp::get())
        .and_then(move || {
            async move {
                handle_query_shared_resources().await
            }
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec!["content-type"]);

    let routes = create_proposal
        .or(vote_on_proposal)
        .or(federation_routes)
        .or(query_shared_resources)
        .or(websocket_route)
        .with(cors);

    let server = warp::serve(routes).run(([0, 0, 0, 0], 8081));

    // Handle graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
    };

    info!("Warp server started.");
    let (_, server_result) = tokio::join!(shutdown_signal, server);

    if let Err(e) = server_result {
        error!("Warp server encountered an error: {}", e);
    }

    // Stop core system
    if let Err(e) = core.stop().await {
        error!("Failed to stop core system: {}", e);
    }

    info!("Backend application stopped.");
    Ok(())
}

async fn handle_create_proposal(proposal: Proposal, notification_manager: NotificationManager, websocket_clients: WebSocketClients, db_pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    // Store proposal in the database
    let db = Database::new(db_pool).await.map_err(|e| warp::reject::custom(AppError::DatabaseError(e)))?;
    db.create_proposal(&proposal).await.map_err(|e| warp::reject::custom(AppError::DatabaseError(e)))?;

    // Logic to handle proposal creation
    let subject = format!("New Proposal Created: {}", proposal.title);
    let body = format!("A new proposal has been created by {}. Description: {}", proposal.created_by, proposal.description);
    notification_manager.send_notification(&subject, &body).await;

    // Broadcast proposal update via WebSocket
    let message = warp::ws::Message::text(serde_json::to_string(&proposal).unwrap());
    broadcast_message(&message, websocket_clients).await;

    Ok(warp::reply::json(&proposal))
}

async fn handle_vote_on_proposal(proposal_id: String, vote: Vote, notification_manager: NotificationManager, websocket_clients: WebSocketClients, db_pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    // Store vote in the database
    let db = Database::new(db_pool).await.map_err(|e| warp::reject::custom(AppError::DatabaseError(e)))?;
    db.record_vote(&vote).await.map_err(|e| warp::reject::custom(AppError::DatabaseError(e)))?;

    // Logic to handle voting on a proposal
    let subject = format!("New Vote on Proposal: {}", proposal_id);
    let body = format!("A new vote has been cast by {}. Approve: {}", vote.voter, vote.approve);
    notification_manager.send_notification(&subject, &body).await;

    // Broadcast vote update via WebSocket
    let message = warp::ws::Message::text(serde_json::to_string(&vote).unwrap());
    broadcast_message(&message, websocket_clients).await;

    Ok(warp::reply::json(&vote))
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
