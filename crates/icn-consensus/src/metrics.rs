// src/monitoring/metrics.rs

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Core metrics interface
#[async_trait]
pub trait MetricsBackend: Send + Sync {
    async fn record_counter(&self, name: &str, value: i64, labels: HashMap<String, String>);
    async fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>);
    async fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>);
}

/// Consensus-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    pub rounds_started: i64,
    pub rounds_completed: i64,
    pub total_votes: i64,
    pub average_round_time_ms: f64,
    pub last_block_height: u64,
    pub active_validators: i64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_in: f64,
    pub network_out: f64,
}

/// Main metrics collector
pub struct MetricsCollector {
    backend: Box<dyn MetricsBackend>,
    consensus_metrics: Arc<RwLock<ConsensusMetrics>>,
    resource_metrics: Arc<RwLock<ResourceMetrics>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(backend: Box<dyn MetricsBackend>) -> Self {
        Self {
            backend,
            consensus_metrics: Arc::new(RwLock::new(ConsensusMetrics {
                rounds_started: 0,
                rounds_completed: 0,
                total_votes: 0,
                average_round_time_ms: 0.0,
                last_block_height: 0,
                active_validators: 0,
            })),
            resource_metrics: Arc::new(RwLock::new(ResourceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_in: 0.0,
                network_out: 0.0,
            })),
        }
    }

    /// Record the start of a consensus round
    pub async fn record_consensus_round_started(&self, round: u64, coordinator: &str) {
        let mut metrics = self.consensus_metrics.write().await;
        metrics.rounds_started += 1;
        
        let mut labels = HashMap::new();
        labels.insert("round".to_string(), round.to_string());
        labels.insert("coordinator".to_string(), coordinator.to_string());
        
        self.backend
            .record_counter("consensus_rounds_started", 1, labels)
            .await;
    }

    /// Record a vote being cast
    pub async fn record_vote_cast(&self, round: u64, validator: &str, vote: bool) {
        let mut metrics = self.consensus_metrics.write().await;
        metrics.total_votes += 1;
        
        let mut labels = HashMap::new();
        labels.insert("round".to_string(), round.to_string());
        labels.insert("validator".to_string(), validator.to_string());
        labels.insert("vote".to_string(), vote.to_string());
        
        self.backend
            .record_counter("consensus_votes_cast", 1, labels)
            .await;
    }

    /// Record a round being finalized
    pub async fn record_round_finalized(
        &self,
        round: u64,
        block_height: u64,
        duration_ms: i64,
        transaction_count: usize,
    ) {
        let mut metrics = self.consensus_metrics.write().await;
        metrics.rounds_completed += 1;
        metrics.last_block_height = block_height;
        
        // Update average round time
        let total_rounds = metrics.rounds_completed as f64;
        metrics.average_round_time_ms = (
            (metrics.average_round_time_ms * (total_rounds - 1.0) + duration_ms as f64)
        ) / total_rounds;
        
        let mut labels = HashMap::new();
        labels.insert("round".to_string(), round.to_string());
        labels.insert("block_height".to_string(), block_height.to_string());
        
        self.backend
            .record_counter("consensus_rounds_completed", 1, labels.clone())
            .await;
            
        self.backend
            .record_histogram("consensus_round_duration_ms", duration_ms as f64, labels.clone())
            .await;
            
        self.backend
            .record_counter(
                "consensus_transactions_processed",
                transaction_count as i64,
                labels,
            )
            .await;
    }

    /// Update system resource metrics
    pub async fn update_resource_metrics(&self, metrics: ResourceMetrics) {
        let mut current = self.resource_metrics.write().await;
        *current = metrics.clone();
        
        let mut labels = HashMap::new();
        
        // Record CPU usage
        self.backend
            .record_gauge("system_cpu_usage", metrics.cpu_usage, labels.clone())
            .await;
            
        // Record memory usage
        self.backend
            .record_gauge("system_memory_usage", metrics.memory_usage, labels.clone())
            .await;
            
        // Record disk usage
        self.backend
            .record_gauge("system_disk_usage", metrics.disk_usage, labels.clone())
            .await;
            
        // Record network metrics
        self.backend
            .record_gauge("system_network_in", metrics.network_in, labels.clone())
            .await;
            
        self.backend
            .record_gauge("system_network_out", metrics.network_out, labels)
            .await;
    }

    /// Get current consensus metrics
    pub async fn get_consensus_metrics(&self) -> ConsensusMetrics {
        self.consensus_metrics.read().await.clone()
    }

    /// Get current resource metrics
    pub async fn get_resource_metrics(&self) -> ResourceMetrics {
        self.resource_metrics.read().await.clone()
    }
}

/// Mock metrics backend for testing
#[derive(Default)]
pub struct MockMetricsCollector {
    counters: Arc<RwLock<HashMap<String, i64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl MockMetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_counter(&self, name: &str) -> i64 {
        self.counters.read().await.get(name).copied().unwrap_or(0)
    }

    pub async fn get_gauge(&self, name: &str) -> f64 {
        self.gauges.read().await.get(name).copied().unwrap_or(0.0)
    }

    pub async fn get_histogram(&self, name: &str) -> Vec<f64> {
        self.histograms.read().await.get(name).cloned().unwrap_or_default()
    }
}

#[async_trait]
impl MetricsBackend for MockMetricsCollector {
    async fn record_counter(&self, name: &str, value: i64, _labels: HashMap<String, String>) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_default() += value;
    }

    async fn record_gauge(&self, name: &str, value: f64, _labels: HashMap<String, String>) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }

    async fn record_histogram(&self, name: &str, value: f64, _labels: HashMap<String, String>) {
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.to_string()).or_default().push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_consensus_metrics() {
        let mock_backend = MockMetricsCollector::new();
        let collector = MetricsCollector::new(Box::new(mock_backend.clone()));

        // Record consensus events
        collector.record_consensus_round_started(1, "validator1").await;
        collector.record_vote_cast(1, "validator1", true).await;
        collector.record_vote_cast(1, "validator2", true).await;
        collector.record_round_finalized(1, 1, 1000, 5).await;

        // Verify metrics
        let metrics = collector.get_consensus_metrics().await;
        assert_eq!(metrics.rounds_started, 1);
        assert_eq!(metrics.rounds_completed, 1);
        assert_eq!(metrics.total_votes, 2);
        assert_eq!(metrics.last_block_height, 1);
        assert!(metrics.average_round_time_ms > 0.0);

        // Verify backend records
        assert_eq!(mock_backend.get_counter("consensus_rounds_started").await, 1);
        assert_eq!(mock_backend.get_counter("consensus_votes_cast").await, 2);
        assert_eq!(mock_backend.get_counter("consensus_rounds_completed").await, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_resource_metrics() {
        let mock_backend = MockMetricsCollector::new();
        let collector = MetricsCollector::new(Box::new(mock_backend.clone()));

        // Update resource metrics
        let metrics = ResourceMetrics {
            cpu_usage: 45.5,
            memory_usage: 75.0,
            disk_usage: 60.0,
            network_in: 1024.0,
            network_out: 512.0,
        };
        collector.update_resource_metrics(metrics.clone()).await;

        // Verify metrics
        let stored_metrics = collector.get_resource_metrics().await;
        assert_eq!(stored_metrics.cpu_usage, metrics.cpu_usage);
        assert_eq!(stored_metrics.memory_usage, metrics.memory_usage);
        assert_eq!(stored_metrics.disk_usage, metrics.disk_usage);
        assert_eq!(stored_metrics.network_in, metrics.network_in);
        assert_eq!(stored_metrics.network_out, metrics.network_out);

        // Verify backend records
        assert_eq!(mock_backend.get_gauge("system_cpu_usage").await, 45.5);
        assert_eq!(mock_backend.get_gauge("system_memory_usage").await, 75.0);
        assert_eq!(mock_backend.get_gauge("system_disk_usage").await, 60.0);
        assert_eq!(mock_backend.get_gauge("system_network_in").await, 1024.0);
        assert_eq!(mock_backend.get_gauge("system_network_out").await, 512.0);
    }
}