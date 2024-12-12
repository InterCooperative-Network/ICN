// File: crates/icn-core/src/telemetry.rs
//
// Telemetry system for ICN node monitoring and metrics collection.
// Handles logging, metrics, and tracing for system monitoring and debugging.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Unit};
use prometheus::{Registry, TextEncoder};
use tracing::{debug, error, info, warn};

use crate::config::TelemetryConfig;
use crate::error::{Error, Result};

/// Core metrics tracked by the telemetry system
#[derive(Debug)]
pub struct CoreMetrics {
    /// Number of active peers
    active_peers: Gauge,
    
    /// Total blocks processed
    blocks_processed: Counter,
    
    /// Block processing time histogram
    block_processing_time: Histogram,
    
    /// Network bandwidth usage
    network_bandwidth: Gauge,
    
    /// Memory usage
    memory_usage: Gauge,
    
    /// CPU usage percentage
    cpu_usage: Gauge,
    
    /// Consensus rounds completed
    consensus_rounds: Counter,
    
    /// Failed consensus rounds
    consensus_failures: Counter,
    
    /// Transaction pool size
    transaction_pool_size: Gauge,
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    
    /// System is degraded but functioning
    Degraded,
    
    /// System is unhealthy
    Unhealthy,
    
    /// System status unknown
    Unknown,
}

/// System resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    
    /// Memory usage in bytes
    pub memory_bytes: u64,
    
    /// Disk usage in bytes
    pub disk_bytes: u64,
    
    /// Network bandwidth in bytes/sec
    pub network_bandwidth: f64,
}

/// Main telemetry manager
#[derive(Debug)]
pub struct TelemetryManager {
    /// Configuration
    config: Arc<TelemetryConfig>,
    
    /// Prometheus registry
    registry: Registry,
    
    /// Core system metrics
    metrics: Arc<CoreMetrics>,
    
    /// Health check status
    health_status: Arc<RwLock<HealthStatus>>,
    
    /// Resource metrics
    resource_metrics: Arc<RwLock<ResourceMetrics>>,
    
    /// Running flag
    running: Arc<AtomicBool>,
    
    /// Start time
    start_time: Instant,
    
    /// Total uptime in seconds
    uptime_seconds: Arc<AtomicU64>,
}

impl TelemetryManager {
    /// Creates a new telemetry manager
    pub fn new(config: &TelemetryConfig) -> Result<Self> {
        let registry = Registry::new();
        
        let metrics = Arc::new(CoreMetrics::new(&registry)?);
        
        Ok(Self {
            config: Arc::new(config.clone()),
            registry,
            metrics,
            health_status: Arc::new(RwLock::new(HealthStatus::Unknown)),
            resource_metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
            running: Arc::new(AtomicBool::new(false)),
            start_time: Instant::now(),
            uptime_seconds: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Starts the telemetry system
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            debug!("Telemetry system already running");
            return Ok(());
        }

        info!("Starting telemetry system");

        // Start metric collection tasks
        self.start_resource_monitoring().await?;
        self.start_uptime_tracking().await?;

        Ok(())
    }

    /// Stops the telemetry system
    pub async fn stop(&self) -> Result<()> {
        if !self.running.swap(false, Ordering::SeqCst) {
            debug!("Telemetry system already stopped");
            return Ok(());
        }

        info!("Stopping telemetry system");
        Ok(())
    }

    /// Updates system health status
    pub async fn update_health_status(&self, status: HealthStatus) {
        *self.health_status.write().await = status;
        
        match status {
            HealthStatus::Healthy => info!("System health status: Healthy"),
            HealthStatus::Degraded => warn!("System health status: Degraded"),
            HealthStatus::Unhealthy => error!("System health status: Unhealthy"),
            HealthStatus::Unknown => warn!("System health status: Unknown"),
        }
    }

    /// Gets current health status
    pub async fn get_health_status(&self) -> HealthStatus {
        *self.health_status.read().await
    }

    /// Records a block being processed
    pub fn record_block_processed(&self, processing_time: Duration) {
        self.metrics.blocks_processed.inc(1);
        self.metrics.block_processing_time.record(processing_time.as_secs_f64());
    }

    /// Updates peer count
    pub fn update_peer_count(&self, count: i64) {
        self.metrics.active_peers.set(count as f64);
    }

    /// Records a consensus round completion
    pub fn record_consensus_round(&self, success: bool) {
        self.metrics.consensus_rounds.inc(1);
        if !success {
            self.metrics.consensus_failures.inc(1);
        }
    }

    /// Updates transaction pool size
    pub fn update_transaction_pool_size(&self, size: usize) {
        self.metrics.transaction_pool_size.set(size as f64);
    }

    /// Gets current resource metrics
    pub async fn get_resource_metrics(&self) -> ResourceMetrics {
        self.resource_metrics.read().await.clone()
    }

    /// Gets system uptime
    pub fn get_uptime(&self) -> Duration {
        Duration::from_secs(self.uptime_seconds.load(Ordering::Relaxed))
    }

    /// Gets metrics in Prometheus format
    pub fn get_prometheus_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        
        encoder.encode_to_string(&metric_families)
            .map_err(|e| Error::system(format!("Failed to encode metrics: {}", e)))
    }

    // Private helper methods

    async fn start_resource_monitoring(&self) -> Result<()> {
        let running = self.running.clone();
        let metrics = self.resource_metrics.clone();
        let update_interval = Duration::from_secs(15);

        tokio::spawn(async move {
            let mut interval = interval(update_interval);

            while running.load(Ordering::Relaxed) {
                interval.tick().await;

                // Update resource metrics
                let new_metrics = Self::collect_resource_metrics().await;
                *metrics.write().await = new_metrics;
            }
        });

        Ok(())
    }

    async fn collect_resource_metrics() -> ResourceMetrics {
        // This is a placeholder - in a real implementation we would:
        // - Use sysinfo or similar to get CPU usage
        // - Use memory_stats for memory usage
        // - Use disk APIs for storage metrics
        // - Use network APIs for bandwidth
        ResourceMetrics::default()
    }

    async fn start_uptime_tracking(&self) -> Result<()> {
        let running = self.running.clone();
        let uptime = self.uptime_seconds.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            while running.load(Ordering::Relaxed) {
                interval.tick().await;
                uptime.fetch_add(1, Ordering::Relaxed);
            }
        });

        Ok(())
    }
}

impl CoreMetrics {
    /// Creates new core metrics and registers them with Prometheus
    fn new(registry: &Registry) -> Result<Self> {
        let metrics = Self {
            active_peers: Gauge::new("active_peers", "Number of active peers"),
            blocks_processed: Counter::new("blocks_processed", "Total blocks processed"),
            block_processing_time: Histogram::with_bounds(
                "block_processing_time",
                "Block processing time in seconds",
                vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
            ),
            network_bandwidth: Gauge::new("network_bandwidth", "Network bandwidth usage in bytes/sec"),
            memory_usage: Gauge::new("memory_usage", "Memory usage in bytes"),
            cpu_usage: Gauge::new("cpu_usage", "CPU usage percentage"),
            consensus_rounds: Counter::new("consensus_rounds", "Total consensus rounds completed"),
            consensus_failures: Counter::new("consensus_failures", "Failed consensus rounds"),
            transaction_pool_size: Gauge::new("transaction_pool_size", "Current transaction pool size"),
        };

        // Register all metrics
        registry.register(Box::new(metrics.active_peers.clone()))?;
        registry.register(Box::new(metrics.blocks_processed.clone()))?;
        registry.register(Box::new(metrics.block_processing_time.clone()))?;
        registry.register(Box::new(metrics.network_bandwidth.clone()))?;
        registry.register(Box::new(metrics.memory_usage.clone()))?;
        registry.register(Box::new(metrics.cpu_usage.clone()))?;
        registry.register(Box::new(metrics.consensus_rounds.clone()))?;
        registry.register(Box::new(metrics.consensus_failures.clone()))?;
        registry.register(Box::new(metrics.transaction_pool_size.clone()))?;

        Ok(metrics)
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            disk_bytes: 0,
            network_bandwidth: 0.0,
        }
    }
}

#[async_trait::async_trait]
impl crate::shutdown::ShutdownHandler for TelemetryManager {
    fn name(&self) -> &str {
        "telemetry"
    }

    fn priority(&self) -> i32 {
        100 // Shut down late to keep collecting metrics
    }

    async fn shutdown(&self) -> Result<()> {
        self.stop().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_telemetry_lifecycle() {
        let config = TelemetryConfig {
            enable_metrics: true,
            metrics_endpoint: "127.0.0.1:9100".to_string(),
            log_level: crate::config::LogLevel::Info,
            enable_debug: false,
        };

        let telemetry = TelemetryManager::new(&config).unwrap();
        
        assert!(telemetry.start().await.is_ok());
        assert!(telemetry.running.load(Ordering::Relaxed));
        
        assert!(telemetry.stop().await.is_ok());
        assert!(!telemetry.running.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_health_status_updates() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetryManager::new(&config).unwrap();

        assert_eq!(telemetry.get_health_status().await, HealthStatus::Unknown);

        telemetry.update_health_status(HealthStatus::Healthy).await;
        assert_eq!(telemetry.get_health_status().await, HealthStatus::Healthy);

        telemetry.update_health_status(HealthStatus::Degraded).await;
        assert_eq!(telemetry.get_health_status().await, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetryManager::new(&config).unwrap();

        telemetry.record_block_processed(Duration::from_millis(100));
        telemetry.update_peer_count(5);
        telemetry.record_consensus_round(true);
        telemetry.update_transaction_pool_size(100);

        let metrics = telemetry.get_prometheus_metrics().unwrap();
        assert!(metrics.contains("blocks_processed"));
        assert!(metrics.contains("active_peers"));
        assert!(metrics.contains("consensus_rounds"));
        assert!(metrics.contains("transaction_pool_size"));
    }

    #[tokio::test]
    async fn test_resource_metrics() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetryManager::new(&config).unwrap();

        telemetry.start().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        let metrics = telemetry.get_resource_metrics().await;
        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.memory_bytes >= 0);
        assert!(metrics.disk_bytes >= 0);
        assert!(metrics.network_bandwidth >= 0.0);

        telemetry.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_uptime_tracking() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetryManager::new(&config).unwrap();

        telemetry.start().await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;

        let uptime = telemetry.get_uptime();
        assert!(uptime.as_secs() >= 1);

        telemetry.stop().await.unwrap();
    }
}