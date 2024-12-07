use prometheus::{Counter, Gauge, Opts, Registry};
use std::sync::Arc;

pub struct ConsensusMetrics {
    pub rounds_total: Counter,
    pub active_validators: Gauge,
    registry: Arc<Registry>,
}

impl ConsensusMetrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());
        
        let rounds_total = Counter::with_opts(Opts::new(
            "consensus_rounds_total",
            "Total number of consensus rounds completed"
        )).unwrap();
        
        let active_validators = Gauge::with_opts(Opts::new(
            "consensus_active_validators",
            "Number of currently active validators"
        )).unwrap();
        
        registry.register(Box::new(rounds_total.clone())).unwrap();
        registry.register(Box::new(active_validators.clone())).unwrap();
        
        Self {
            rounds_total,
            active_validators,
            registry,
        }
    }
    
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

impl Default for ConsensusMetrics {
    fn default() -> Self {
        Self::new()
    }
}
