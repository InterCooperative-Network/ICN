use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Resource impact metrics for cooperative operations
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ResourceImpact {
    /// CPU intensity of operations (0-10)
    pub cpu_intensity: u8,
    
    /// Memory usage in MB
    pub memory_usage: u8,
    
    /// Network usage in MB/s
    pub network_usage: u8,
    
    /// Storage usage in MB
    pub storage_usage: u8,
    
    /// Bandwidth usage in MB/s
    pub bandwidth_usage: u8,
}

/// Metadata about a cooperative's operations and resource usage
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CooperativeMetadata {
    /// DID of the cooperative's creator
    #[serde(default)]
    pub creator_did: String,
    
    /// Unique identifier for the cooperative
    #[serde(default)]
    pub cooperative_id: String,
    
    /// Description of cooperative's purpose
    #[serde(default)]
    pub purpose: String,
    
    /// Resource impact metrics
    #[serde(default)]
    pub resource_impact: ResourceImpact,
    
    /// Federation ID if part of a federation
    #[serde(default)]
    pub federation_id: Option<String>,
    
    /// When the cooperative was created
    #[serde(default)]
    pub creation_timestamp: u64,
    
    /// When metadata was last updated
    #[serde(default)]
    pub last_updated: u64,
    
    /// Current number of members
    #[serde(default)]
    pub member_count: u64,
    
    /// Resource allocation per member
    #[serde(default)]
    pub resource_allocation: HashMap<String, u64>,

    /// Energy usage metrics in kWh
    #[serde(default)]
    pub energy_usage: HashMap<String, f64>,
}

impl CooperativeMetadata {
    /// Creates new metadata for a cooperative
    pub fn new(
        creator_did: String, 
        cooperative_id: String,
        purpose: String
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        CooperativeMetadata {
            creator_did,
            cooperative_id,
            purpose,
            creation_timestamp: now,
            last_updated: now,
            member_count: 1, // Start with creator as member
            resource_impact: ResourceImpact::default(),
            federation_id: None,
            resource_allocation: HashMap::new(),
            energy_usage: HashMap::new(),
        }
    }

    /// Updates the metadata timestamp
    pub fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Adds a member and updates count
    pub fn add_member(&mut self) -> u64 {
        self.member_count += 1;
        self.update_timestamp();
        self.member_count
    }

    /// Removes a member and updates count
    pub fn remove_member(&mut self) -> u64 {
        if self.member_count > 0 {
            self.member_count -= 1;
        }
        self.update_timestamp();
        self.member_count
    }

    /// Allocates resources to a member
    pub fn allocate_resources(&mut self, member_did: String, amount: u64) {
        self.resource_allocation.insert(member_did, amount);
        self.update_timestamp();
    }

    /// Records energy usage for a specific operation
    pub fn record_energy_usage(&mut self, operation: &str, kwh: f64) {
        let current = self.energy_usage.get(operation).copied().unwrap_or(0.0);
        self.energy_usage.insert(operation.to_string(), current + kwh);
        self.update_timestamp();
    }

    /// Gets total energy usage across all operations
    pub fn total_energy_usage(&self) -> f64 {
        self.energy_usage.values().sum()
    }

    /// Checks if cooperative is part of federation
    pub fn is_federated(&self) -> bool {
        self.federation_id.is_some()
    }

    /// Sets federation membership
    pub fn join_federation(&mut self, federation_id: String) {
        self.federation_id = Some(federation_id);
        self.update_timestamp();
    }

    /// Removes federation membership
    pub fn leave_federation(&mut self) {
        self.federation_id = None;
        self.update_timestamp();
    }

    /// Gets resource allocation for a member
    pub fn get_member_resources(&self, member_did: &str) -> u64 {
        self.resource_allocation.get(member_did).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_metadata() -> CooperativeMetadata {
        CooperativeMetadata::new(
            "did:icn:test".to_string(),
            "coop-1".to_string(),
            "Test cooperative".to_string()
        )
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.creator_did, "did:icn:test");
        assert_eq!(metadata.cooperative_id, "coop-1");
        assert_eq!(metadata.member_count, 1);
    }

    #[test]
    fn test_member_management() {
        let mut metadata = create_test_metadata();
        assert_eq!(metadata.add_member(), 2);
        assert_eq!(metadata.add_member(), 3);
        assert_eq!(metadata.remove_member(), 2);
    }

    #[test]
    fn test_resource_allocation() {
        let mut metadata = create_test_metadata();
        metadata.allocate_resources("member1".to_string(), 100);
        assert_eq!(metadata.get_member_resources("member1"), 100);
        assert_eq!(metadata.get_member_resources("nonexistent"), 0);
    }

    #[test]
    fn test_federation_membership() {
        let mut metadata = create_test_metadata();
        assert!(!metadata.is_federated());
        
        metadata.join_federation("fed-1".to_string());
        assert!(metadata.is_federated());
        assert_eq!(metadata.federation_id, Some("fed-1".to_string()));
        
        metadata.leave_federation();
        assert!(!metadata.is_federated());
    }

    #[test]
    fn test_energy_usage_tracking() {
        let mut metadata = create_test_metadata();
        metadata.record_energy_usage("compute", 10.5);
        metadata.record_energy_usage("storage", 5.2);
        metadata.record_energy_usage("compute", 4.5);
        
        assert_eq!(metadata.energy_usage.get("compute"), Some(&15.0));
        assert!(metadata.total_energy_usage() > 20.0);
    }

    #[test]
    fn test_default_implementation() {
        let metadata = CooperativeMetadata::default();
        assert!(metadata.creator_did.is_empty());
        assert!(metadata.cooperative_id.is_empty());
        assert_eq!(metadata.member_count, 0);
        assert!(metadata.resource_allocation.is_empty());
    }
}
