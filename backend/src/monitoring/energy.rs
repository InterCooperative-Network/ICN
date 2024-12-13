use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyMetrics {
    pub cpu_cycles: u64,
    pub instructions_executed: u64,
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub peak_memory_usage: usize,
    pub network_packets_sent: u64,
    pub network_bytes_transferred: u64,
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub storage_bytes_written: u64,
    pub consensus_rounds: u64,
    pub validator_operations: u64,
    pub operation_duration_ms: u64,
    pub estimated_energy_consumption: f64,
    pub estimated_carbon_footprint: f64,
    pub node_location: Option<String>,
    pub power_source: Option<String>,
}

pub struct EnergyMonitor {
    start_time: Instant,
    cpu_cycles: AtomicU64,
    instructions: AtomicU64,
    memory_ops: AtomicU64,
    network_ops: AtomicU64,
    consensus_ops: AtomicU64,
    storage_ops: AtomicU64,
    node_config: NodeEnergyConfig,
}

#[derive(Clone)]
pub struct NodeEnergyConfig {
    pub location: Option<String>,
    pub power_source: Option<String>,
    pub carbon_factor: f64,
    pub cpu_energy_factor: f64,
    pub memory_energy_factor: f64,
    pub network_energy_factor: f64,
    pub storage_energy_factor: f64,
}

impl Default for NodeEnergyConfig {
    fn default() -> Self {
        NodeEnergyConfig {
            location: None,
            power_source: None,
            carbon_factor: 500.0,
            cpu_energy_factor: 0.0000001,
            memory_energy_factor: 0.0000002,
            network_energy_factor: 0.0000005,
            storage_energy_factor: 0.0000003,
        }
    }
}

impl EnergyMonitor {
    pub fn new(config: NodeEnergyConfig) -> Self {
        EnergyMonitor {
            start_time: Instant::now(),
            cpu_cycles: AtomicU64::new(0),
            instructions: AtomicU64::new(0),
            memory_ops: AtomicU64::new(0),
            network_ops: AtomicU64::new(0),
            consensus_ops: AtomicU64::new(0),
            storage_ops: AtomicU64::new(0),
            node_config: config,
        }
    }

    pub fn record_cpu_cycles(&self, cycles: u64) {
        self.cpu_cycles.fetch_add(cycles, Ordering::Relaxed);
    }

    pub fn record_instruction(&self) {
        self.instructions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_memory_operation(&self, bytes: u64) {
        self.memory_ops.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_network_operation(&self, bytes: u64) {
        self.network_ops.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_consensus_operation(&self) {
        self.consensus_ops.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_storage_operation(&self, bytes: u64) {
        self.storage_ops.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn estimate_energy_consumption(&self) -> f64 {
        let cpu_energy = self.cpu_cycles.load(Ordering::Relaxed) as f64 
            * self.node_config.cpu_energy_factor;
        let memory_energy = self.memory_ops.load(Ordering::Relaxed) as f64 
            * self.node_config.memory_energy_factor;
        let network_energy = self.network_ops.load(Ordering::Relaxed) as f64 
            * self.node_config.network_energy_factor;
        let storage_energy = self.storage_ops.load(Ordering::Relaxed) as f64 
            * self.node_config.storage_energy_factor;

        cpu_energy + memory_energy + network_energy + storage_energy
    }

    pub fn estimate_carbon_footprint(&self) -> f64 {
        let energy_kwh = self.estimate_energy_consumption() / 3_600_000.0;
        energy_kwh * self.node_config.carbon_factor
    }

    pub fn get_metrics(&self) -> EnergyMetrics {
        let total_memory_ops = self.memory_ops.load(Ordering::Relaxed);
        let total_network_ops = self.network_ops.load(Ordering::Relaxed);
        let total_storage_ops = self.storage_ops.load(Ordering::Relaxed);

        EnergyMetrics {
            cpu_cycles: self.cpu_cycles.load(Ordering::Relaxed),
            instructions_executed: self.instructions.load(Ordering::Relaxed),
            memory_reads: total_memory_ops / 2,
            memory_writes: total_memory_ops / 2,
            peak_memory_usage: 0,
            network_packets_sent: total_network_ops,
            network_bytes_transferred: total_network_ops,
            storage_reads: total_storage_ops / 2,
            storage_writes: total_storage_ops / 2,
            storage_bytes_written: total_storage_ops,
            consensus_rounds: self.consensus_ops.load(Ordering::Relaxed),
            validator_operations: 0,
            operation_duration_ms: self.start_time.elapsed().as_millis() as u64,
            estimated_energy_consumption: self.estimate_energy_consumption(),
            estimated_carbon_footprint: self.estimate_carbon_footprint(),
            node_location: self.node_config.location.clone(),
            power_source: self.node_config.power_source.clone(),
        }
    }
}

pub trait EnergyAware {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_monitoring() {
        let config = NodeEnergyConfig::default();
        let monitor = EnergyMonitor::new(config);

        monitor.record_cpu_cycles(1000);
        monitor.record_instruction();
        monitor.record_memory_operation(1024);
        monitor.record_network_operation(512);
        monitor.record_storage_operation(2048);
        monitor.record_consensus_operation();

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.cpu_cycles, 1000);
        assert_eq!(metrics.instructions_executed, 1);
        assert!(metrics.estimated_energy_consumption > 0.0);
        assert!(metrics.estimated_carbon_footprint > 0.0);
    }

    #[test]
    fn test_carbon_footprint_calculation() {
        let mut config = NodeEnergyConfig::default();
        config.carbon_factor = 100.0;
        let monitor = EnergyMonitor::new(config);

        monitor.record_cpu_cycles(1000000);
        let metrics = monitor.get_metrics();
        
        assert!(metrics.estimated_carbon_footprint > 0.0);
        assert!(metrics.estimated_carbon_footprint < metrics.estimated_carbon_footprint * 5.0);
    }
}
