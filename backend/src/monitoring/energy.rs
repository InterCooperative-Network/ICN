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
}

pub struct EnergyMonitor {
    start_time: Instant,
    cpu_cycles: AtomicU64,
    instructions: AtomicU64,
    memory_ops: AtomicU64,
    network_ops: AtomicU64,
    consensus_ops: AtomicU64,
}

impl EnergyMonitor {
    pub fn new() -> Self {
        EnergyMonitor {
            start_time: Instant::now(),
            cpu_cycles: AtomicU64::new(0),
            instructions: AtomicU64::new(0),
            memory_ops: AtomicU64::new(0),
            network_ops: AtomicU64::new(0),
            consensus_ops: AtomicU64::new(0),
        }
    }
    
    pub fn record_cpu_cycles(&self, cycles: u64) {
        self.cpu_cycles.fetch_add(cycles, Ordering::Relaxed);
    }
    
    pub fn record_instruction(&self) {
        self.instructions.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_memory_operation(&self) {
        self.memory_ops.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_network_operation(&self) {
        self.network_ops.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_consensus_operation(&self) {
        self.consensus_ops.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Calculate estimated energy consumption based on recorded metrics
    pub fn estimate_energy_consumption(&self) -> f64 {
        // Constants for energy calculations (these would need to be calibrated)
        const ENERGY_PER_CPU_CYCLE: f64 = 0.0000001; // joules per CPU cycle
        const ENERGY_PER_MEMORY_OP: f64 = 0.0000002; // joules per memory operation
        const ENERGY_PER_NETWORK_OP: f64 = 0.0000005; // joules per network operation
        const ENERGY_PER_CONSENSUS_OP: f64 = 0.0000010; // joules per consensus operation
        
        let cpu_energy = self.cpu_cycles.load(Ordering::Relaxed) as f64 * ENERGY_PER_CPU_CYCLE;
        let memory_energy = self.memory_ops.load(Ordering::Relaxed) as f64 * ENERGY_PER_MEMORY_OP;
        let network_energy = self.network_ops.load(Ordering::Relaxed) as f64 * ENERGY_PER_NETWORK_OP;
        let consensus_energy = self.consensus_ops.load(Ordering::Relaxed) as f64 * ENERGY_PER_CONSENSUS_OP;
        
        cpu_energy + memory_energy + network_energy + consensus_energy
    }
    
    pub fn get_metrics(&self) -> EnergyMetrics {
        EnergyMetrics {
            cpu_cycles: self.cpu_cycles.load(Ordering::Relaxed),
            instructions_executed: self.instructions.load(Ordering::Relaxed),
            memory_reads: self.memory_ops.load(Ordering::Relaxed) / 2, // Assuming 50/50 split
            memory_writes: self.memory_ops.load(Ordering::Relaxed) / 2,
            peak_memory_usage: 0, // Would need to implement memory tracking
            network_packets_sent: self.network_ops.load(Ordering::Relaxed),
            network_bytes_transferred: 0, // Would need to implement byte counting
            storage_reads: 0, // Would need to implement storage tracking
            storage_writes: 0,
            storage_bytes_written: 0,
            consensus_rounds: self.consensus_ops.load(Ordering::Relaxed),
            validator_operations: 0, // Would need to implement validator tracking
            operation_duration_ms: self.start_time.elapsed().as_millis() as u64,
            estimated_energy_consumption: self.estimate_energy_consumption(),
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
        if self.uses_memory() {
            monitor.record_memory_operation();
        }
        
        // Record network operations
        if self.uses_network() {
            monitor.record_network_operation();
        }
    }
}

/// Implementation for the consensus system
impl EnergyAware for crate::consensus::ProofOfCooperation {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record consensus operations
        monitor.record_consensus_operation();
        
        // Record network operations for consensus messages
        monitor.record_network_operation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_energy_monitoring() {
        let monitor = EnergyMonitor::new();
        
        // Simulate some operations
        monitor.record_cpu_cycles(1000);
        monitor.record_instruction();
        monitor.record_memory_operation();
        monitor.record_network_operation();
        monitor.record_consensus_operation();
        
        let metrics = monitor.get_metrics();
        
        assert_eq!(metrics.cpu_cycles, 1000);
        assert_eq!(metrics.instructions_executed, 1);
        assert!(metrics.estimated_energy_consumption > 0.0);
    }
}