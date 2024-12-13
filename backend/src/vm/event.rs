use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Type of event that occurred (e.g., "ContractCreated", "ProposalSubmitted")
    pub event_type: String,
    
    /// ID of the cooperative that generated the event
    pub cooperative_id: String,
    
    /// Additional event-specific data as key-value pairs
    pub data: HashMap<String, String>,
    
    /// When the event occurred
    pub timestamp: u64,
    
    /// Additional context for the event
    pub context: Option<EventContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    /// DID of the entity that triggered the event
    pub triggered_by: String,
    
    /// Block number when event occurred
    pub block_number: u64,
    
    /// Module that generated the event (e.g., "governance", "reputation")
    pub source_module: String,
    
    /// Any related transaction ID
    pub transaction_id: Option<String>,
}

impl Event {
    pub fn new(
        event_type: String,
        cooperative_id: String,
        data: HashMap<String, String>,
        timestamp: u64
    ) -> Self {
        Event {
            event_type,
            cooperative_id,
            data,
            timestamp,
            context: None,
        }
    }

    pub fn with_context(
        event_type: String,
        cooperative_id: String,
        data: HashMap<String, String>,
        timestamp: u64,
        context: EventContext
    ) -> Self {
        Event {
            event_type,
            cooperative_id,
            data,
            timestamp,
            context: Some(context),
        }
    }

    /// Add additional data to the event
    pub fn add_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Set the event context
    pub fn set_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }

    /// Get formatted timestamp
    pub fn get_timestamp_utc(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now())
    }
}

impl EventContext {
    pub fn new(
        triggered_by: String,
        block_number: u64,
        source_module: String
    ) -> Self {
        EventContext {
            triggered_by,
            block_number,
            source_module,
            transaction_id: None,
        }
    }

    pub fn with_transaction(
        triggered_by: String,
        block_number: u64,
        source_module: String,
        transaction_id: String
    ) -> Self {
        EventContext {
            triggered_by,
            block_number,
            source_module,
            transaction_id: Some(transaction_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        
        let event = Event::new(
            "TestEvent".to_string(),
            "coop-1".to_string(),
            data,
            1000,
        );

        assert_eq!(event.event_type, "TestEvent");
        assert_eq!(event.cooperative_id, "coop-1");
        assert_eq!(event.data.get("key").unwrap(), "value");
    }

    #[test]
    fn test_event_with_context() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        
        let context = EventContext::new(
            "did:icn:test".to_string(),
            1,
            "test_module".to_string(),
        );

        let event = Event::with_context(
            "TestEvent".to_string(),
            "coop-1".to_string(),
            data,
            1000,
            context,
        );

        assert!(event.context.is_some());
        let context = event.context.unwrap();
        assert_eq!(context.triggered_by, "did:icn:test");
        assert_eq!(context.block_number, 1);
        assert_eq!(context.source_module, "test_module");
    }

    #[test]
    fn test_add_data() {
        let mut event = Event::new(
            "TestEvent".to_string(),
            "coop-1".to_string(),
            HashMap::new(),
            1000,
        );

        event.add_data("test_key", "test_value");
        assert_eq!(event.data.get("test_key").unwrap(), "test_value");
    }

    #[test]
    fn test_set_context() {
        let mut event = Event::new(
            "TestEvent".to_string(),
            "coop-1".to_string(),
            HashMap::new(),
            1000,
        );

        let context = EventContext::new(
            "did:icn:test".to_string(),
            1,
            "test_module".to_string(),
        );

        event.set_context(context);
        assert!(event.context.is_some());
    }

    #[test]
    fn test_event_context_with_transaction() {
        let context = EventContext::with_transaction(
            "did:icn:test".to_string(),
            1,
            "test_module".to_string(),
            "tx-1".to_string(),
        );

        assert_eq!(context.transaction_id.unwrap(), "tx-1");
    }
}
