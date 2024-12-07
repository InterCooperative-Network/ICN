// src/monitoring/service.rs

use super::metrics::{MetricsCollector, ResourceMetrics};
use crate::state::StateManager;
use crate::consensus::ConsensusStateManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use sysinfo::{System, SystemExt, CpuExt};

/// Service for collecting and reporting system metrics
pub struct MonitoringService {
    metrics: Arc<MetricsCollector>,
    state_manager: Arc<StateManager>,
    consensus_manager: Arc<ConsensusStateManager>,
    system_info: System,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(
        metrics: Arc<MetricsCollector>,
        state_manager: Arc<StateManager>,
        consensus_manager: Arc<ConsensusStateManager>,
    ) -> Self {
        Self {
            metrics,
            state_manager,
            consensus_manager,
            system_info: System::new_all(),
        }
    }

    /// Start the monitoring service
    pub async fn start(&mut self) {
        let mut interval = time::interval(Duration::from_secs(15));

        loop {
            interval.tick().await;
            self.collect_metrics().await;
            
            // Log any significant changes or issues
            if let Err(e) = self.verify_system_health().await {
                eprintln!("Health check failed: {}", e);
            }
        }
    }

    /// Collect all system metrics
    async fn collect_metrics(&mut self) {
        // Update system info
        self.system_info.refresh_all();

        // Calculate CPU usage
        let cpu_usage = self.system_info.global_cpu_info().cpu_usage();

        // Calculate memory usage
        let total_memory = self.system_info.total_memory() as f64;
        let used_memory = self.system_info.used_memory() as f64;
        let memory_usage = (used_memory / total_memory) * 100.0;

        // Calculate disk usage
        let total_space = self.system_info.disks().iter()
            .map(|disk| disk.total_space() as f64)
            .sum::<f64>();
        let used_space = self.system_info.disks().iter()
            .map(|disk| (disk.total_space() - disk.available_space()) as f64)
            .sum::<f64>();
        let disk_usage = (used_space / total_space) * 100.0;

        // Get network usage (simplified)
        let network_info = self.system_info.networks();
        let network_in = network_info.iter()
            .map(|(_, data)| data.received() as f64)
            .sum::<f64>();
        let network_out = network_info.iter()
            .map(|(_, data)| data.transmitted() as f64)
            .sum::<f64>();

        // Record resource metrics
        let metrics = ResourceMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_in,
            network_out,
        };
        
        self.metrics.update_resource_metrics(metrics).await;
    }

    /// Verify system health
    async fn verify_system_health(&self) -> Result<(), String> {
        // Check resource usage thresholds
        let metrics = self.metrics.get_resource_metrics().await;

        if metrics.cpu_usage > 90.0 {
            return Err("CPU usage critically high".to_string());
        }

        if metrics.memory_usage > 90.0 {
            return Err("Memory usage critically high".to_string());
        }

        if metrics.disk_usage > 90.0 {
            return Err("Disk usage critically high".to_string());
        }

        // Verify consensus state
        if !self.consensus_manager.verify_state().await.unwrap_or(false) {
            return Err("Consensus state verification failed".to_string());
        }

        // Verify state consistency
        if !self.state_manager.verify_state().await.unwrap_or(false) {
            return Err("State consistency check failed".to_string());
        }

        Ok(())
    }
}

/// Configuration for alert thresholds
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_critical: f64,
    pub memory_critical: f64,
    pub disk_critical: f64,
    pub consensus_delay_critical: Duration,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_critical: 90.0,
            memory_critical: 90.0,
            disk_critical: 90.0,
            consensus_delay_critical: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::MockMetricsCollector;
    use crate::storage::postgres::PostgresStorage;
    
    async fn setup_test_env() -> MonitoringService {
        // Create dependencies
        let metrics = Arc::new(MetricsCollector::new(Box::new(MockMetricsCollector::new())));
        let storage = setup_test_storage().await;
        let state_manager = Arc::new(StateManager::new(Arc::new(storage)).await.unwrap());
        let consensus_manager = Arc::new(ConsensusStateManager::new(
            state_manager.clone(),
            metrics.clone(),
        ).await.unwrap());

        MonitoringService::new(metrics, state_manager, consensus_manager)
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let mut service = setup_test_env().await;
        
        // Collect initial metrics
        service.collect_metrics().await;
        
        // Verify metrics were collected
        let metrics = service.metrics.get_resource_metrics().await;
        
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0);
        assert!(metrics.disk_usage >= 0.0);
        assert!(metrics.network_in >= 0.0);
        assert!(metrics.network_out >= 0.0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let service = setup_test_env().await;
        
        // Initial health check should pass
        assert!(service.verify_system_health().await.is_ok());
        
        // Verify health check with high resource usage
        service.metrics.update_resource_metrics(ResourceMetrics {
            cpu_usage: 95.0,
            memory_usage: 95.0,
            disk_usage: 95.0,
            network_in: 0.0,
            network_out: 0.0,
        }).await;
        
        assert!(service.verify_system_health().await.is_err());
    }
}