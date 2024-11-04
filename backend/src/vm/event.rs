use std::collections::HashMap;

pub struct Event {
    pub event_type: String,
    pub cooperative_id: String,
    pub data: HashMap<String, String>,
    pub timestamp: u64,
}
