// File: crates/icn-core/src/lib.rs

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub mod api;
pub mod config;
pub mod cooperative;
pub mod error;
pub mod governance;
pub mod service;
pub mod shutdown;
pub mod telemetry;

use icn_consensus as consensus;
use icn_p2p as p2p;
use icn_runtime as runtime;
use icn_storage as storage;
use icn_types::*;

pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use shutdown::ShutdownManager;
pub use telemetry::TelemetryManager;

/// Core system state and configuration
#[derive(Debug)]
pub struct Core {
    /// Global configuration
    config: Arc<Config>,
    
    /// Consensus engine
    consensus: Arc<consensus::ConsensusEngine>,
    
    /// Storage manager
    storage: Arc<storage::StorageManager>,
    
    /// P2P networking
    network: Arc<p2p::NetworkManager>,
    
    /// Runtime environment
    runtime: Arc<runtime::RuntimeManager>,
    
    /// Telemetry system
    telemetry: Arc<TelemetryManager>,
    
    /// Shutdown coordinator
    shutdown: Arc<ShutdownManager>,
    
    /// System event broadcaster
    event_tx: broadcast::Sender<SystemEvent>,
}

/// System-wide events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// New block added
    BlockAdded {
        height: u64,
        hash: String,
    },
    /// Consensus round completed
    ConsensusComplete {
        round_id: u64,
        success: bool,
    },
    /// Node status changed
    StatusChanged {
        status: NodeStatus,
        timestamp: i64,
    },
    /// System error occurred
    Error {
        code: String,
        message: String,
        severity: ErrorSeverity,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Starting,
    Ready,
    Syncing,
    Shutdown,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Core {
    /// Create a new Core instance with the provided configuration
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let (event_tx, _) = broadcast::channel(1000);
        
        let telemetry = Arc::new(TelemetryManager::new(&config)?);
        let shutdown = Arc::new(ShutdownManager::new());
        
        // Initialize subsystems
        let storage = Arc::new(
            storage::StorageManager::new(config.storage.clone()).await?
        );
        
        let consensus = Arc::new(
            consensus::ConsensusEngine::new(config.consensus.clone()).await?
        );
        
        let network = Arc::new(
            p2p::NetworkManager::new(config.network.clone()).await?
        );
        
        let runtime = Arc::new(
            runtime::RuntimeManager::new(config.runtime.clone())?
        );

        Ok(Self {
            config,
            consensus,
            storage,
            network,
            runtime,
            telemetry,
            shutdown,
            event_tx,
        })
    }

    /// Start the core system
    pub async fn start(&self) -> Result<()> {
        info!("Starting ICN core system...");
        
        // Initialize telemetry first for monitoring
        self.telemetry.start().await?;
        
        // Start subsystems in order
        self.storage.start().await?;
        self.consensus.start().await?;
        self.network.start().await?;
        self.runtime.start().await?;
        
        self.broadcast_status(NodeStatus::Ready);
        info!("ICN core system started successfully");
        
        Ok(())
    }

    /// Gracefully shutdown the system
    pub async fn shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown...");
        self.broadcast_status(NodeStatus::Shutdown);
        
        // Shutdown in reverse order
        self.runtime.shutdown().await?;
        self.network.shutdown().await?;
        self.consensus.shutdown().await?;
        self.storage.shutdown().await?;
        
        // Shutdown telemetry last
        self.telemetry.shutdown().await?;
        
        info!("System shutdown complete");
        Ok(())
    }

    /// Get a receiver for system events
    pub fn subscribe_events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_tx.subscribe()
    }

    /// Access the consensus engine
    pub fn consensus(&self) -> Arc<consensus::ConsensusEngine> {
        self.consensus.clone()
    }

    /// Access the storage manager
    pub fn storage(&self) -> Arc<storage::StorageManager> {
        self.storage.clone()
    }

    /// Access the network manager
    pub fn network(&self) -> Arc<p2p::NetworkManager> {
        self.network.clone()
    }

    /// Access the runtime manager
    pub fn runtime(&self) -> Arc<runtime::RuntimeManager> {
        self.runtime.clone()
    }

    /// Access the telemetry manager
    pub fn telemetry(&self) -> Arc<TelemetryManager> {
        self.telemetry.clone()
    }

    /// Access the shutdown manager
    pub fn shutdown_manager(&self) -> Arc<ShutdownManager> {
        self.shutdown.clone()
    }

    // Internal helper to broadcast status changes
    fn broadcast_status(&self, status: NodeStatus) {
        let event = SystemEvent::StatusChanged {
            status,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.event_tx.send(event) {
            error!("Failed to broadcast status change: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_core_lifecycle() {
        let config = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();
            
        let core = Core::new(config).await.unwrap();
        
        assert!(core.start().await.is_ok());
        assert!(core.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();
            
        let core = Core::new(config).await.unwrap();
        let mut rx = core.subscribe_events();
        
        core.start().await.unwrap();
        
        if let Ok(SystemEvent::StatusChanged { status, .. }) = rx.recv().await {
            assert_eq!(status, NodeStatus::Ready);
        } else {
            panic!("Expected status change event");
        }
        
        core.shutdown().await.unwrap();
    }
}