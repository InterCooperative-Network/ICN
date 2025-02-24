use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub sequence: u64,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct EventStore {
    events: Arc<RwLock<Vec<StoredEvent>>>,
    snapshots: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn append_event(&self, event: StoredEvent) {
        let mut events = self.events.write().await;
        events.push(event);
        
        // Take snapshot every N events
        if events.len() % 100 == 0 {
            self.create_snapshot().await;
        }
    }

    pub async fn get_events(&self, after_sequence: u64) -> Vec<StoredEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.sequence > after_sequence)
            .cloned()
            .collect()
    }

    async fn create_snapshot(&self) {
        // Create point-in-time snapshot of current state
        let mut snapshots = self.snapshots.write().await;
        // Snapshot creation logic
    }
}
