// src/monitoring/prometheus.rs

use super::metrics::MetricsBackend;
use async_trait::async_trait;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Opts, Registry,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
}

/// Prometheus implementation of the metrics backend
pub struct PrometheusBackend {
    counters: Arc<RwLock<HashMap<String, CounterVec>>>,
    gauges: Arc<RwLock<HashMap<String, GaugeVec>>>,
    histograms: Arc<RwLock<HashMap<String, HistogramVec>>>,
}

impl PrometheusBackend {
    /// Create a new Prometheus metrics backend
    pub fn new() -> Self {
        let instance = Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register default metrics
        tokio::spawn(instance.register_default_metrics());

        instance
    }

    /// Register the default set of metrics
    async fn register_default_metrics(&self) {
        // Consensus metrics
        self.create_counter("consensus_rounds_started", "Number of consensus rounds started")
            .await;
        self.create_counter("consensus_rounds_completed", "Number of consensus rounds completed")
            .await;
        self.create_counter("consensus_votes_cast", "Number of votes cast")
            .await;
        self.create_counter(
            "consensus_transactions_processed",
            "Number of transactions processed",
        )
        .await;

        // Create histograms with appropriate buckets
        self.create_histogram(
            "consensus_round_duration_ms",
            "Consensus round duration in milliseconds",
            vec![100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0],
        )
        .await;

        // System metrics
        self.create_gauge("system_cpu_usage", "CPU usage percentage")
            .await;
        self.create_gauge("system_memory_usage", "Memory usage percentage")
            .await;
        self.create_gauge("system_disk_usage", "Disk usage percentage")
            .await;
        self.create_gauge("system_network_in", "Network input bytes/sec")
            .await;
        self.create_gauge("system_network_out", "Network output bytes/sec")
            .await;
    }

    async fn create_counter(&self, name: &str, help: &str) {
        let counter = CounterVec::new(
            Opts::new(name, help),
            &["instance", "validator"],
        ).unwrap();
        
        REGISTRY.register(Box::new(counter.clone())).unwrap();
        
        let mut counters = self.counters.write().await;
        counters.insert(name.to_string(), counter);
    }

    async fn create_gauge(&self, name: &str, help: &str) {
        let gauge = GaugeVec::new(
            Opts::new(name, help),
            &["instance"],
        ).unwrap();
        
        REGISTRY.register(Box::new(gauge.clone())).unwrap();
        
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), gauge);
    }

    async fn create_histogram(&self, name: &str, help: &str, buckets: Vec<f64>) {
        let histogram = HistogramVec::new(
            Opts::new(name, help),
            &["instance", "validator"],
            buckets,
        ).unwrap();
        
        REGISTRY.register(Box::new(histogram.clone())).unwrap();
        
        let mut histograms = self.histograms.write().await;
        histograms.insert(name.to_string(), histogram);
    }

    /// Get the Prometheus registry
    pub fn registry(&self) -> &Registry {
        &REGISTRY
    }
}

#[async_trait]
impl MetricsBackend for PrometheusBackend {
    async fn record_counter(&self, name: &str, value: i64, labels: HashMap<String, String>) {
        if let Some(counter) = self.counters.read().await.get(name) {
            let label_values: Vec<&str> = labels
                .values()
                .map(|s| s.as_str())
                .collect();
                
            counter
                .with_label_values(&label_values)
                .inc_by(value as f64);
        }
    }

    async fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Some(gauge) = self.gauges.read().await.get(name) {
            let label_values: Vec<&str> = labels
                .values()
                .map(|s| s.as_str())
                .collect();
                
            gauge
                .with_label_values(&label_values)
                .set(value);
        }
    }

    async fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Some(histogram) = self.histograms.read().await.get(name) {
            let label_values: Vec<&str> = labels
                .values()
                .map(|s| s.as_str())
                .collect();
                
            histogram
                .with_label_values(&label_values)
                .observe(value);
        }
    }
}

/// HTTP handler for Prometheus metrics endpoint
pub async fn metrics_handler() -> impl warp::Reply {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    
    String::from_utf8(buffer).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_prometheus_metrics() {
        let backend = PrometheusBackend::new();
        
        // Record some test metrics
        let mut labels = HashMap::new();
        labels.insert("instance".to_string(), "test".to_string());
        labels.insert("validator".to_string(), "validator1".to_string());

        backend.record_counter("consensus_rounds_started", 1, labels.clone()).await;
        backend.record_gauge("system_cpu_usage", 45.5, labels.clone()).await;
        backend.record_histogram("consensus_round_duration_ms", 1000.0, labels).await;

        // Get the metrics output
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&backend.registry().gather(), &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Verify the metrics were recorded
        assert!(output.contains("consensus_rounds_started"));
        assert!(output.contains("system_cpu_usage"));
        assert!(output.contains("consensus_round_duration_ms"));
    }
}