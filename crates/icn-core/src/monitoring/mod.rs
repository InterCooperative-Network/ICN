// src/service/mod.rs

use crate::monitoring::{
    service::MonitoringService,
    metrics::{MetricsCollector, PrometheusBackend},
    prometheus::metrics_handler
};
use crate::state::StateManager;
use crate::consensus::ConsensusStateManager;
use crate::storage::postgres::PostgresStorage;
use crate::websocket::WebSocketHandler;

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use warp::Filter;

/// Errors that can occur in service operations
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Configuration for the service
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub database_url: String,
    pub ws_port: u16,
    pub metrics_port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/icn".to_string(),
            ws_port: 8088,
            metrics_port: 9090,
        }
    }
}

/// Manages all services in the system
pub struct ServiceManager {
    config: ServiceConfig,
    state_manager: Arc<StateManager>,
    consensus_manager: Arc<ConsensusStateManager>,
    metrics: Arc<MetricsCollector>,
    ws_handler: Arc<WebSocketHandler>,
}

impl ServiceManager {
    /// Create a new service manager
    pub async fn new(config: ServiceConfig) -> Result<Self, ServiceError> {
        // Initialize storage
        let storage = PostgresStorage::new(&config.database_url)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            
        let storage = Arc::new(StorageManager::new(Box::new(storage)));

        // Initialize state manager
        let state_manager = Arc::new(StateManager::new(storage)
            .await
            .map_err(|e| ServiceError::StateError(e.to_string()))?);

        // Initialize metrics
        let metrics = Arc::new(MetricsCollector::new(Box::new(PrometheusBackend::new())));

        // Initialize consensus manager
        let consensus_manager = Arc::new(ConsensusStateManager::new(
            state_manager.clone(),
            metrics.clone(),
        ).await.map_err(|e| ServiceError::StateError(e.to_string()))?);

        // Initialize WebSocket handler
        let ws_handler = Arc::new(WebSocketHandler::new());

        Ok(Self {
            config,
            state_manager,
            consensus_manager,
            metrics,
            ws_handler,
        })
    }

    /// Start all services
    pub async fn start(&self) -> Result<(), ServiceError> {
        // Start monitoring service
        let mut monitoring = MonitoringService::new(
            self.metrics.clone(),
            self.state_manager.clone(),
            self.consensus_manager.clone(),
        );
        
        tokio::spawn(async move {
            monitoring.start().await;
        });

        // Start WebSocket server
        let ws_routes = self.setup_ws_routes();
        let ws_addr = ([127, 0, 0, 1], self.config.ws_port);
        
        tokio::spawn(async move {
            warp::serve(ws_routes)
                .run(ws_addr)
                .await;
        });

        // Start metrics server
        let metrics_routes = warp::path("metrics").and_then(metrics_handler);
        let metrics_addr = ([127, 0, 0, 1], self.config.metrics_port);
        
        tokio::spawn(async move {
            warp::serve(metrics_routes)
                .run(metrics_addr)
                .await;
        });

        Ok(())
    }

    /// Set up WebSocket routes
    fn setup_ws_routes(&self) -> impl Filter<Extract = impl warp::Reply> + Clone {
        let ws_handler = self.ws_handler.clone();
        
        warp::path("ws")
            .and(warp::ws())
            .and(warp::header::<String>("X-DID"))
            .and(warp::any().map(move || ws_handler.clone()))
            .map(|ws: warp::ws::Ws, did: String, handler: Arc<WebSocketHandler>| {
                ws.on_upgrade(move |socket| async move {
                    handler.handle_connection(socket, did).await;
                })
            })
    }

    /// Stop all services
    pub async fn stop(&self) -> Result<(), ServiceError> {
        // Implement graceful shutdown logic
        Ok(())
    }

    /// Get current service status
    pub async fn get_status(&self) -> ServiceStatus {
        let consensus_metrics = self.metrics.get_consensus_metrics().await;
        let resource_metrics = self.metrics.get_resource_metrics().await;
        
        ServiceStatus {
            consensus_height: consensus_metrics.last_block_height,
            active_validators: consensus_metrics.active_validators,
            cpu_usage: resource_metrics.cpu_usage,
            memory_usage: resource_metrics.memory_usage,
            disk_usage: resource_metrics.disk_usage,
        }
    }
}

/// Current status of the service
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub consensus_height: u64,
    pub active_validators: i64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    async fn setup_test_service() -> ServiceManager {
        let config = ServiceConfig {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test".to_string()),
            ws_port: 8089,
            metrics_port: 9091,
        };

        ServiceManager::new(config)
            .await
            .expect("Failed to create service manager")
    }

    #[tokio::test]
    #[serial]
    async fn test_service_lifecycle() {
        let service = setup_test_service().await;
        
        // Start services
        service.start().await.expect("Failed to start services");
        
        // Wait a bit for services to initialize
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Check status
        let status = service.get_status().await;
        assert!(status.cpu_usage >= 0.0);
        assert!(status.memory_usage >= 0.0);
        assert!(status.disk_usage >= 0.0);
        
        // Stop services
        service.stop().await.expect("Failed to stop services");
    }

    #[tokio::test]
    #[serial]
    async fn test_metrics_endpoint() {
        let service = setup_test_service().await;
        
        // Start services
        service.start().await.expect("Failed to start services");
        
        // Wait for metrics to initialize
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Make request to metrics endpoint
        let response = reqwest::get(&format!(
            "http://localhost:{}/metrics",
            service.config.metrics_port
        ))
        .await
        .expect("Failed to get metrics");
        
        assert!(response.status().is_success());
        
        let body = response.text().await.unwrap();
        
        // Verify metrics content
        assert!(body.contains("consensus_rounds_started"));
        assert!(body.contains("system_cpu_usage"));
    }

    #[tokio::test]
    #[serial]
    async fn test_websocket_connection() {
        let service = setup_test_service().await;
        
        // Start services
        service.start().await.expect("Failed to start services");
        
        // Connect to WebSocket
        let url = format!("ws://localhost:{}/ws", service.config.ws_port);
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Failed to connect to WebSocket");
            
        // Verify connection
        assert!(ws_stream.closed().is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_state_consistency() {
        let service = setup_test_service().await;
        
        // Start services
        service.start().await.expect("Failed to start services");
        
        // Verify state managers are initialized
        assert!(service.state_manager.verify_state().await.unwrap());
        assert!(service.consensus_manager.verify_state().await.unwrap());
    }
}