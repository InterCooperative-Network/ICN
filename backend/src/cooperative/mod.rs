use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::claims::Claim;
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};

/// Represents a cooperative within the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooperative {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub members: HashMap<String, MemberRole>, // DID -> Role mapping
    pub resources: HashMap<String, Resource>,
    pub policies: Vec<Policy>,
    pub federation_ids: Vec<String>, // Federated cooperatives
    pub community_id: String,        // Associated civic community
}

/// Represents the role of a member within the cooperative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRole {
    pub role: String,
    pub permissions: Vec<String>,
    pub joined_at: DateTime<Utc>,
    pub verified_claims: Vec<Claim>,
}

/// Represents a resource managed by the cooperative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub resource_type: ResourceType,
    pub quantity: f64,
    pub unit: String,
    pub availability: ResourceAvailability,
    pub tags: Vec<String>,
    pub shared_with: Vec<String>, // Cooperative IDs
}

/// Defines types of resources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Physical,
    Digital,
    Service,
    Skill,
    Space,
    Equipment,
    Other(String),
}

/// Represents the availability status of a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailability {
    pub status: AvailabilityStatus,
    pub schedule: Option<Schedule>,
    pub conditions: Vec<String>,
}

/// Defines the possible availability statuses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AvailabilityStatus {
    Available,
    InUse,
    Reserved,
    Maintenance,
    Unavailable,
}

/// Represents a schedule for resource availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub recurring: bool,
    pub frequency: Option<String>,
}

/// Represents a policy enforced by the cooperative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub policy_type: PolicyType,
    pub description: String,
    pub rules: Vec<Rule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Defines the types of policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    ResourceSharing,
    MembershipRequirement,
    FederationRule,
    Other(String),
}

/// Represents a rule within a policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub condition: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
}

impl Cooperative {
    /// Creates a new cooperative instance.
    pub fn new(
        id: String,
        name: String,
        description: String,
        community_id: String,
    ) -> Self {
        Cooperative {
            id,
            name,
            description,
            created_at: Utc::now(),
            members: HashMap::new(),
            resources: HashMap::new(),
            policies: Vec::new(),
            federation_ids: Vec::new(),
            community_id,
        }
    }

    /// Adds a member to the cooperative.
    pub fn add_member(&mut self, did: String, role: MemberRole) -> Result<(), String> {
        if self.members.contains_key(&did) {
            return Err("Member already exists".to_string());
        }
        self.members.insert(did, role);
        Ok(())
    }

    /// Adds a resource to the cooperative.
    pub fn add_resource(&mut self, resource: Resource) -> Result<(), String> {
        if self.resources.contains_key(&resource.id) {
            return Err("Resource already exists".to_string());
        }
        self.resources.insert(resource.id.clone(), resource);
        Ok(())
    }

    /// Adds a policy to the cooperative.
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Federates with another cooperative.
    pub fn federate_with(&mut self, cooperative_id: String) {
        if !self.federation_ids.contains(&cooperative_id) {
            self.federation_ids.push(cooperative_id);
        }
    }
}

/// Implements energy metrics tracking for cooperatives.
impl EnergyAware for Cooperative {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        monitor.record_instruction();

        // Track resource storage.
        let storage_size = (self.resources.len() * std::mem::size_of::<Resource>()) as u64;
        monitor.record_storage_operation(storage_size);

        // Track member memory usage.
        let members_size = (self.members.len() * std::mem::size_of::<MemberRole>()) as u64;
        monitor.record_memory_operation(members_size);
    }
}
