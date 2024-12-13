use crate::consensus::types::ValidatorInfo; 
use super::core::ProofOfCooperation;
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};


impl EnergyAware for ProofOfCooperation {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record voting operations
        if let Some(round) = self.round_manager.get_current_round() {
            let vote_count = round.votes.len();
            monitor.record_consensus_operation();
            monitor.record_network_operation((vote_count * 256) as u64); // Estimate network usage
        }
        
        // Record validator state size
        let validator_size = (self.validator_manager.get_validators().len() * std::mem::size_of::<ValidatorInfo>()) as u64;
        monitor.record_memory_operation(validator_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::energy::NodeEnergyConfig;
    use crate::websocket::WebSocketHandler;
    use crate::consensus::types::ConsensusConfig;
    use std::sync::Arc;

    #[test]
    fn test_energy_metrics() {
        let ws_handler = Arc::new(WebSocketHandler::new());
        let config = ConsensusConfig::default();
        let consensus = ProofOfCooperation::new(config, ws_handler);
        let monitor = EnergyMonitor::new(NodeEnergyConfig::default());
        
        consensus.record_energy_metrics(&monitor);
    }
}
