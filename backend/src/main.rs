use tokio::runtime::Runtime;
use log::{info, error};
use env_logger;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use sha2::{Sha256, Digest};
use warp::Filter;
use futures_util::future::join_all;
use async_trait::async_trait;
use icn_core::{Core, TelemetryManager, PrometheusMetrics, Logger, TracingSystem};
use icn_consensus::ProofOfCooperation;
use icn_crypto::KeyPair;
use icn_p2p::networking::NetworkManager;
use icn_runtime::RuntimeManager;
use icn_storage::{StorageManager, StorageBackend, StorageResult};
use icn_types::{Block, Transaction};
use tokio::signal;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use reqwest::Client;

#[derive(Deserialize)]
struct Config {
    database_url: String,
    log_level: String,
    reputation_decay_rate: f64,
    reputation_adjustment_interval: u64,
    reputation_initial_score: i64,
    reputation_positive_contribution_weight: f64,
    reputation_negative_contribution_weight: f64,
    notification_email: String,
    notification_sms: String,
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

struct NotificationManager {
    client: Client,
    email: String,
    sms: String,
}

impl NotificationManager {
    fn new(email: String, sms: String) -> Self {
        NotificationManager {
            client: Client::new(),
            email,
            sms,
        }
    }

    async fn send_email(&self, subject: &str, body: &str) -> Result<(), reqwest::Error> {
        self.client.post(&self.email)
            .body(format!("Subject: {}\n\n{}", subject, body))
            .send()
            .await?;
        Ok(())
    }

    async fn send_sms(&self, message: &str) -> Result<(), reqwest::Error> {
        self.client.post(&self.sms)
            .body(message.to_string())
            .send()
            .await?;
        Ok(())
    }

    async fn send_notification(&self, subject: &str, body: &str) {
        if let Err(e) = self.send_email(subject, body).await {
            error!("Failed to send email notification: {}", e);
            if let Err(e) = self.send_sms(body).await {
                error!("Failed to send SMS notification: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    info!("Starting backend application...");

    // Load configuration
    let config: Config = match load_config() {
        Ok(config) => {
            info!("Configuration loaded successfully.");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return;
        }
    };

    // Initialize components
    let storage_manager = StorageManager::new(Box::new(MockStorageBackend));
    let network_manager = NetworkManager::new();
    let runtime_manager = RuntimeManager::new();
    let telemetry_manager = TelemetryManager::new(PrometheusMetrics, Logger, TracingSystem);
    let identity_manager = IdentityManager::new();
    let reputation_manager = ReputationManager::new(
        config.reputation_decay_rate,
        config.reputation_adjustment_interval,
        config.reputation_initial_score,
        config.reputation_positive_contribution_weight,
        config.reputation_negative_contribution_weight,
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
        return;
    }

    // Set up Warp server
    let create_proposal = warp::path!("api" / "governance" / "proposals")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |proposal: Proposal| {
            let notification_manager = notification_manager.clone();
            async move {
                handle_create_proposal(proposal, notification_manager).await
            }
        });

    let vote_on_proposal = warp::path!("api" / "governance" / "proposals" / String / "vote")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |proposal_id: String, vote: Vote| {
            let notification_manager = notification_manager.clone();
            async move {
                handle_vote_on_proposal(proposal_id, vote, notification_manager).await
            }
        });

    let routes = create_proposal.or(vote_on_proposal);

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
}

async fn handle_create_proposal(proposal: Proposal, notification_manager: NotificationManager) -> Result<impl warp::Reply, warp::Rejection> {
    // Logic to handle proposal creation
    let subject = format!("New Proposal Created: {}", proposal.title);
    let body = format!("A new proposal has been created by {}. Description: {}", proposal.created_by, proposal.description);
    notification_manager.send_notification(&subject, &body).await;
    Ok(warp::reply::json(&proposal))
}

async fn handle_vote_on_proposal(proposal_id: String, vote: Vote, notification_manager: NotificationManager) -> Result<impl warp::Reply, warp::Rejection> {
    // Logic to handle voting on a proposal
    let subject = format!("New Vote on Proposal: {}", proposal_id);
    let body = format!("A new vote has been cast by {}. Approve: {}", vote.voter, vote.approve);
    notification_manager.send_notification(&subject, &body).await;
    Ok(warp::reply::json(&vote))
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

struct MockStorageBackend;

#[async_trait]
impl StorageBackend for MockStorageBackend {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        Ok(())
    }

    async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        Ok(true)
    }
}
