use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CooperativeMetadata {
    pub resource_impact: ResourceImpact,
    pub contract_version: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceImpact {
    pub cpu: u32,
    pub memory: u32,
    pub bandwidth: u32,
}