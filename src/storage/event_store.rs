use std::sync::RwLock;
use std::collections::HashMap;
use crate::services::event_service::DomainEvent;

pub struct EventStore {
    events: RwLock<HashMap<String, Vec<DomainEvent>>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
        }
    }

    pub fn append(&self, stream_id: &str, event: DomainEvent) -> Result<(), String> {
        let mut events = self.events.write().map_err(|_| "Lock error")?;
        events.entry(stream_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);
        Ok(())
    }

    pub fn get_stream(&self, stream_id: &str) -> Result<Vec<DomainEvent>, String> {
        let events = self.events.read().map_err(|_| "Lock error")?;
        Ok(events.get(stream_id)
            .cloned()
            .unwrap_or_default())
    }
}
