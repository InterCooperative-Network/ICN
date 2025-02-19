use std::sync::Arc;
use crate::storage::StorageManager;
use crate::networking::NetworkManager;
use crate::identity::IdentityManager;
use crate::reputation::ReputationManager;
use async_trait::async_trait;
use icn_consensus::ConsensusEngine;

pub struct Core {
    _storage_manager: Arc<StorageManager>,
    _network_manager: Arc<NetworkManager>,
    _runtime_manager: Arc<RuntimeManager>,
    _telemetry_manager: Arc<TelemetryManager>,
    _identity_manager: Arc<IdentityManager>,
    _reputation_manager: Arc<ReputationManager>,
}

pub struct TelemetryManager;
pub struct PrometheusMetrics;
pub struct Logger;
pub struct TracingSystem;
pub struct RuntimeManager;

impl Core {
    pub fn new(
        storage_manager: Arc<StorageManager>,
        network_manager: Arc<NetworkManager>,
        runtime_manager: Arc<RuntimeManager>,
        telemetry_manager: Arc<TelemetryManager>,
        identity_manager: Arc<IdentityManager>,
        reputation_manager: Arc<ReputationManager>,
    ) -> Self {
        Core {
            _storage_manager: storage_manager,
            _network_manager: network_manager,
            _runtime_manager: runtime_manager,
            _telemetry_manager: telemetry_manager,
            _identity_manager: identity_manager,
            _reputation_manager: reputation_manager,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        self._telemetry_manager.log("Starting Core...");
        if let Err(e) = self._network_manager.start().await {
            return Err(format!("Failed to start network manager: {}", e));
        }
        if let Err(e) = self._runtime_manager.start().await {
            return Err(format!("Failed to start runtime manager: {}", e));
        }
        if let Err(e) = self._identity_manager.start().await {
            return Err(format!("Failed to start identity manager: {}", e));
        }
        if let Err(e) = self._reputation_manager.start().await {
            return Err(format!("Failed to start reputation manager: {}", e));
        }
        self._telemetry_manager.log("Core started.");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        self._telemetry_manager.log("Stopping Core...");
        if let Err(e) = self._runtime_manager.stop().await {
            return Err(format!("Failed to stop runtime manager: {}", e));
        }
        if let Err(e) = self._network_manager.stop().await {
            return Err(format!("Failed to stop network manager: {}", e));
        }
        if let Err(e) = self._identity_manager.stop().await {
            return Err(format!("Failed to stop identity manager: {}", e));
        }
        if let Err(e) = self._reputation_manager.stop().await {
            return Err(format!("Failed to stop reputation manager: {}", e));
        }
        self._telemetry_manager.log("Core stopped.");
        Ok(())
    }
}

#[async_trait]
impl ConsensusEngine for Core {
    async fn start(&self) -> Result<(), String> {
        self.start().await
    }

    async fn stop(&self) -> Result<(), String> {
        self.stop().await
    }
}
