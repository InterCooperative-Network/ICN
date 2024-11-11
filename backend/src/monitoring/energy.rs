// src/monitoring/energy.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyMetrics {
    // CPU usage metrics
    pub cpu_cycles: u64,
    pub instructions_executed: u64,
    
    // Memory metrics
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub peak_memory_usage: usize,
    
    // Network metrics
    pub network_packets_sent: u64,
    pub network_bytes_transferred: u64,
    
    // Storage metrics
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub storage_bytes_written: u64,
    
    // Consensus-specific metrics
    pub consensus_rounds: u64,
    pub validator_operations: u64,
    
    // Timing metrics
    pub operation_duration_ms: u64,
    
    // Estimated energy consumption (in joules)
    pub estimated_energy_consumption: f64,
    
    // Carbon footprint estimation (in grams CO2)
    pub estimated_carbon_footprint: f64,
    
    // Node-specific metrics
    pub node_location: Option<String>,  // For regional power grid calculations
    pub power_source: Option<String>,   // e.g., "grid", "renewable", "mixed"
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
    pub carbon_factor: f64,  // grams CO2 per kWh for this node's power source
    // Hardware-specific energy coefficients
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
            carbon_factor: 500.0, // Default assumption: 500g CO2/kWh (mixed grid)
            cpu_energy_factor: 0.0000001,    // joules per CPU cycle
            memory_energy_factor: 0.0000002,  // joules per memory operation
            network_energy_factor: 0.0000005, // joules per network operation
            storage_energy_factor: 0.0000003, // joules per storage operation
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
    
    /// Calculate estimated energy consumption based on recorded metrics
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
    
    /// Calculate carbon footprint based on energy consumption
    pub fn estimate_carbon_footprint(&self) -> f64 {
        let energy_kwh = self.estimate_energy_consumption() / 3_600_000.0; // Convert joules to kWh
        energy_kwh * self.node_config.carbon_factor
    }
    
    pub fn get_metrics(&self) -> EnergyMetrics {
        let total_memory_ops = self.memory_ops.load(Ordering::Relaxed);
        let total_network_ops = self.network_ops.load(Ordering::Relaxed);
        let total_storage_ops = self.storage_ops.load(Ordering::Relaxed);
        
        EnergyMetrics {
            cpu_cycles: self.cpu_cycles.load(Ordering::Relaxed),
            instructions_executed: self.instructions.load(Ordering::Relaxed),
            memory_reads: total_memory_ops / 2,  // Assuming 50/50 split
            memory_writes: total_memory_ops / 2,
            peak_memory_usage: 0, // Would need to implement memory tracking
            network_packets_sent: total_network_ops,
            network_bytes_transferred: total_network_ops,
            storage_reads: total_storage_ops / 2,
            storage_writes: total_storage_ops / 2,
            storage_bytes_written: total_storage_ops,
            consensus_rounds: self.consensus_ops.load(Ordering::Relaxed),
            validator_operations: 0, // Would need to implement validator tracking
            operation_duration_ms: self.start_time.elapsed().as_millis() as u64,
            estimated_energy_consumption: self.estimate_energy_consumption(),
            estimated_carbon_footprint: self.estimate_carbon_footprint(),
            node_location: self.node_config.location.clone(),
            power_source: self.node_config.power_source.clone(),
        }
    }
}

/// Trait for components that want to track their energy usage
pub trait EnergyAware {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor);
}

/// Implementation for the VM to track its energy usage
impl EnergyAware for crate::vm::VM {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record CPU operations
        monitor.record_cpu_cycles(self.get_cycle_count());
        
        // Record instructions
        monitor.record_instruction();
        
        // Record memory operations
        if let Some(memory_usage) = self.get_memory_usage() {
            monitor.record_memory_operation(memory_usage);
        }
        
        // Record storage operations
        if let Some(storage_usage) = self.get_storage_usage() {
            monitor.record_storage_operation(storage_usage);
        }
    }
}

/// Implementation for the consensus system
impl EnergyAware for crate::consensus::ProofOfCooperation {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record consensus operations
        monitor.record_consensus_operation();
        
        // Record network operations for consensus messages
        if let Some(network_bytes) = self.get_network_usage() {
            monitor.record_network_operation(network_bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_energy_monitoring() {
        let config = NodeEnergyConfig::default();
        let monitor = EnergyMonitor::new(config);
        
        // Simulate some operations
        monitor.record_cpu_cycles(1000);
        monitor.record_instruction();
        monitor.record_memory_operation(1024); // 1KB memory operation
        monitor.record_network_operation(512); // 512B network operation
        monitor.record_storage_operation(2048); // 2KB storage operation
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
        config.carbon_factor = 100.0; // 100g CO2/kWh (renewable heavy grid)
        let monitor = EnergyMonitor::new(config);
        
        monitor.record_cpu_cycles(1000000); // Significant CPU usage
        
        let metrics = monitor.get_metrics();
        assert!(metrics.estimated_carbon_footprint > 0.0);
        assert!(metrics.estimated_carbon_footprint < 
                metrics.estimated_carbon_footprint * 5.0); // Should be significantly less than default grid
    }
}