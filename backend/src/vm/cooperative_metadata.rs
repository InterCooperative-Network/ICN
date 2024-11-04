use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    pub cpu_intensity: u8,
    pub memory_usage: u8,
    pub network_usage: u8,
    pub storage_usage: u8,
    pub bandwidth_usage: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CooperativeMetadata {
    pub creator_did: String,
    pub cooperative_id: String,
    pub purpose: String,
    pub resource_impact: ResourceImpact,
    pub federation_id: Option<String>,
    pub creation_timestamp: u64,
    pub last_updated: u64,
    pub member_count: u64,
    pub resource_allocation: HashMap<String, u64>,
}
