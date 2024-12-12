// File: crates/icn-core/src/metrics.rs
//
// Unified metrics system for the ICN network. This module provides centralized
// metrics collection and monitoring capabilities across all system components.
// It integrates with both Prometheus for metrics exposition and internal
// monitoring systems.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use metrics::{Counter, Gauge, Histogram, Key, Unit};
use prometheus::{Registry, TextEncoder};
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};

/// Core metrics tracked across the system
#[derive(Debug)]
pub struct SystemMetrics {
    // Consensus metrics
    pub consensus_rounds_total: AtomicU64,
    pub consensus_rounds_failed: AtomicU64,
    pub consensus_time_ms: AtomicU64,
    pub active_validators: AtomicI64,
    
    // Network metrics
    pub connected_peers: AtomicI64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub active_connections: AtomicI64,
    
    // Storage metrics
    pub blocks_stored: AtomicU64,
    pub transactions_stored: AtomicU64,
    pub storage_bytes_used: AtomicU64,
    pub query_time_ms: AtomicU64,
    
    // Runtime metrics
    pub active_tasks: AtomicI64,
    pub task_queue_size: AtomicI64,
    pub task_complete_time_ms: AtomicU64,
    pub task_errors: AtomicU64,
}

/// Metrics for a specific consensus round
#[derive(Debug, Clone)]
pub struct ConsensusMetrics {
    /// Round identifier
    pub round_id: u64,
    
    /// Number of participating validators
    pub validator_count: usize,
    
    /// Total voting power participating
    pub total_voting_power: u64,
    
    /// Time taken for round completion
    pub round_time: Duration,
    
    /// Whether consensus was achieved
    pub consensus_achieved: bool,
}

/// Network performance metrics
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Current bandwidth usage (bytes/sec)
    pub bandwidth_usage: f64,
    
    /// Connected peer count
    pub peer_count: usize,
    
    /// Average message latency
    pub avg_latency_ms: f64,
    
    /// Message success rate (0.0-1.0)
    pub message_success_rate: f64,
}

/// Storage performance metrics
#[derive(Debug, Clone)]
pub struct StorageMetrics {
    /// Average query time
    pub avg_query_time_ms: f64,
    
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    
    /// Storage space used (bytes)
    pub storage_used_bytes: u64,
    
    /// IOPS (IO operations per second)
    pub iops: f64,
}

impl SystemMetrics {
    /// Create new system metrics
    pub fn new() -> Self {
        Self {
            // Consensus metrics
            consensus_rounds_total: AtomicU64::new(0),
            consensus_rounds_failed: AtomicU64::new(0),
            consensus_time_ms: AtomicU64::new(0),
            active_validators: AtomicI64::new(0),
            
            // Network metrics
            connected_peers: AtomicI64::new(0),
            bytes_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            active_connections: AtomicI64::new(0),
            
            // Storage metrics
            blocks_stored: AtomicU64::new(0),
            transactions_stored: AtomicU64::new(0),
            storage_bytes_used: AtomicU64::new(0),
            query_time_ms: AtomicU64::new(0),
            
            // Runtime metrics
            active_tasks: AtomicI64::new(0),
            task_queue_size: AtomicI64::new(0),
            task_complete_time_ms: AtomicU64::new(0),
            task_errors: AtomicU64::new(0),
        }
    }

    /// Record a completed consensus round
    pub fn record_consensus_round(&self, metrics: ConsensusMetrics) {
        self.consensus_rounds_total.fetch_add(1, Ordering::Relaxed);
        
        if !metrics.consensus_achieved {
            self.consensus_rounds_failed.fetch_add(1, Ordering::Relaxed);
        }
        
        self.consensus_time_ms.fetch_add(
            metrics.round_time.as_millis() as u64,
            Ordering::Relaxed
        );
        
        self.active_validators.store(
            metrics.validator_count as i64,
            Ordering::Relaxed
        );
    }

    /// Record network metrics
    pub fn record_network_metrics(&self, metrics: NetworkMetrics) {
        self.connected_peers.store(
            metrics.peer_count as i64,
            Ordering::Relaxed
        );
        
        // Update bandwidth metrics (bytes per second)
        let bandwidth = metrics.bandwidth_usage as u64;
        self.bytes_received.fetch_add(bandwidth / 2, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bandwidth / 2, Ordering::Relaxed);
    }

    /// Record storage metrics
    pub fn record_storage_metrics(&self, metrics: StorageMetrics) {
        self.storage_bytes_used.store(
            metrics.storage_used_bytes,
            Ordering::Relaxed
        );
        
        self.query_time_ms.store(
            metrics.avg_query_time_ms as u64,
            Ordering::Relaxed
        );
    }

    /// Record task completion
    pub fn record_task_completion(&self, duration: Duration, success: bool) {
        self.task_complete_time_ms.fetch_add(
            duration.as_millis() as u64,
            Ordering::Relaxed
        );
        
        if !success {
            self.task_errors.fetch_add(1, Ordering::Relaxed);
        }
        
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get current consensus success rate
    pub fn consensus_success_rate(&self) -> f64 {
        let total = self.consensus_rounds_total.load(Ordering::Relaxed);
        let failed = self.consensus_rounds_failed.load(Ordering::Relaxed);
        
        if total == 0 {
            return 1.0;
        }
        
        (total - failed) as f64 / total as f64
    }

    /// Get average consensus round time
    pub fn avg_consensus_time(&self) -> Duration {
        let total_time = self.consensus_time_ms.load(Ordering::Relaxed);
        let rounds = self.consensus_rounds_total.load(Ordering::Relaxed);
        
        if rounds == 0 {
            return Duration::from_millis(0);
        }
        
        Duration::from_millis(total_time / rounds)
    }

    /// Get network throughput (bytes/sec)
    pub fn network_throughput(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed) +
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.consensus_rounds_total.store(0, Ordering::Relaxed);
        self.consensus_rounds_failed.store(0, Ordering::Relaxed);
        self.consensus_time_ms.store(0, Ordering::Relaxed);
        self.active_validators.store(0, Ordering::Relaxed);
        self.connected_peers.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        self.blocks_stored.store(0, Ordering::Relaxed);
        self.transactions_stored.store(0, Ordering::Relaxed);
        self.storage_bytes_used.store(0, Ordering::Relaxed);
        self.query_time_ms.store(0, Ordering::Relaxed);
        self.active_tasks.store(0, Ordering::Relaxed);
        self.task_queue_size.store(0, Ordering::Relaxed);
        self.task_complete_time_ms.store(0, Ordering::Relaxed);
        self.task_errors.store(0, Ordering::Relaxed);
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_consensus_metrics_recording() {
        let metrics = SystemMetrics::new();
        
        let round_metrics = ConsensusMetrics {
            round_id: 1,
            validator_count: 10,
            total_voting_power: 1000,
            round_time: Duration::from_millis(100),
            consensus_achieved: true,
        };
        
        metrics.record_consensus_round(round_metrics);
        
        assert_eq!(metrics.consensus_rounds_total.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.consensus_rounds_failed.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.active_validators.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_network_metrics_recording() {
        let metrics = SystemMetrics::new();
        
        let network_metrics = NetworkMetrics {
            bandwidth_usage: 1000.0,
            peer_count: 5,
            avg_latency_ms: 50.0,
            message_success_rate: 0.99,
        };
        
        metrics.record_network_metrics(network_metrics);
        
        assert_eq!(metrics.connected_peers.load(Ordering::Relaxed), 5);
        assert!(metrics.network_throughput() > 0);
    }

    #[test]
    fn test_storage_metrics_recording() {
        let metrics = SystemMetrics::new();
        
        let storage_metrics = StorageMetrics {
            avg_query_time_ms: 10.0,
            cache_hit_rate: 0.8,
            storage_used_bytes: 1024 * 1024,
            iops: 100.0,
        };
        
        metrics.record_storage_metrics(storage_metrics);
        
        assert_eq!(
            metrics.storage_bytes_used.load(Ordering::Relaxed),
            1024 * 1024
        );
        assert_eq!(metrics.query_time_ms.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_task_metrics_recording() {
        let metrics = SystemMetrics::new();
        
        // Record successful task
        metrics.active_tasks.fetch_add(1, Ordering::Relaxed);
        metrics.record_task_completion(Duration::from_millis(50), true);
        
        // Record failed task
        metrics.active_tasks.fetch_add(1, Ordering::Relaxed);
        metrics.record_task_completion(Duration::from_millis(100), false);
        
        assert_eq!(metrics.task_errors.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.active_tasks.load(Ordering::Relaxed), -1);
        assert_eq!(
            metrics.task_complete_time_ms.load(Ordering::Relaxed),
            150
        );
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = SystemMetrics::new();
        
        // Record some metrics
        metrics.consensus_rounds_total.store(10, Ordering::Relaxed);
        metrics.bytes_received.store(1000, Ordering::Relaxed);
        metrics.active_tasks.store(5, Ordering::Relaxed);
        
        // Reset metrics
        metrics.reset();
        
        assert_eq!(metrics.consensus_rounds_total.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.bytes_received.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.active_tasks.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_consensus_success_rate() {
        let metrics = SystemMetrics::new();
        
        // No rounds yet
        assert_eq!(metrics.consensus_success_rate(), 1.0);
        
        // 8/10 successful rounds
        metrics.consensus_rounds_total.store(10, Ordering::Relaxed);
        metrics.consensus_rounds_failed.store(2, Ordering::Relaxed);
        
        assert_eq!(metrics.consensus_success_rate(), 0.8);
    }

    #[test]
    fn test_average_consensus_time() {
        let metrics = SystemMetrics::new();
        
        // No rounds yet
        assert_eq!(metrics.avg_consensus_time(), Duration::from_millis(0));
        
        // Record some round times
        metrics.consensus_rounds_total.store(2, Ordering::Relaxed);
        metrics.consensus_time_ms.store(100, Ordering::Relaxed);
        
        assert_eq!(metrics.avg_consensus_time(), Duration::from_millis(50));
    }
}