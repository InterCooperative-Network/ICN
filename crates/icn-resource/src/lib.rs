use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
// use icn_federation::ResourceManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub owner: String,
    pub total_amount: u64,
    pub available_amount: u64,
    pub price_per_unit: f64,
    pub constraints: ResourceConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraints {
    pub min_reputation: i64,
    pub max_allocation: u64,
    pub time_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationResourcePool {
    pub federation_id: String,
    pub resources: HashMap<String, Resource>,
    pub access_control: FederationAccessControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationAccessControl {
    pub allowed_federations: Vec<String>,
    pub min_reputation: i64,
    pub max_allocation_per_federation: u64,
}

pub struct ResourceAllocationSystem {
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    allocations: Arc<RwLock<HashMap<String, Vec<ResourceAllocation>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub resource_id: String,
    pub recipient: String,
    pub amount: u64,
    pub allocated_at: u64,
    pub expires_at: Option<u64>,
}

impl ResourceAllocationSystem {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_resource(
        &self,
        owner: String,
        resource_type: String,
        amount: u64,
        price: f64,
        constraints: ResourceConstraints,
    ) -> Result<String, ResourceError> {
        let resource_id = format!("res_{}", uuid::Uuid::new_v4());
        let resource = Resource {
            id: resource_id.clone(),
            resource_type,
            owner,
            total_amount: amount,
            available_amount: amount,
            price_per_unit: price,
            constraints,
        };

        let mut resources = self.resources.write().await;
        resources.insert(resource_id.clone(), resource);

        Ok(resource_id)
    }

    pub async fn allocate(
        &self,
        resource_id: &str,
        recipient: String,
        amount: u64,
    ) -> Result<String, ResourceError> {
        let mut resources = self.resources.write().await;
        let resource = resources.get_mut(resource_id)
            .ok_or(ResourceError::ResourceNotFound)?;

        if resource.available_amount < amount {
            return Err(ResourceError::InsufficientResources);
        }

        let allocation_id = format!("alloc_{}", uuid::Uuid::new_v4());
        let allocation = ResourceAllocation {
            allocation_id: allocation_id.clone(),
            resource_id: resource_id.to_string(),
            recipient,
            amount,
            allocated_at: chrono::Utc::now().timestamp() as u64,
            expires_at: None,
        };

        resource.available_amount -= amount;

        let mut allocations = self.allocations.write().await;
        allocations.entry(resource_id.to_string())
            .or_insert_with(Vec::new)
            .push(allocation);

        Ok(allocation_id)
    }

    pub async fn release(
        &self,
        allocation_id: &str,
    ) -> Result<(), ResourceError> {
        let mut allocations = self.allocations.write().await;
        for (resource_id, resource_allocations) in allocations.iter_mut() {
            if let Some(index) = resource_allocations.iter().position(|alloc| alloc.allocation_id == allocation_id) {
                let allocation = resource_allocations.remove(index);
                let mut resources = self.resources.write().await;
                if let Some(resource) = resources.get_mut(&allocation.resource_id) {
                    resource.available_amount += allocation.amount;
                }
                return Ok(());
            }
        }
        Err(ResourceError::InvalidAllocation)
    }

    pub async fn adjust_price(
        &self,
        resource_id: &str,
        new_price: f64,
    ) -> Result<(), ResourceError> {
        let mut resources = self.resources.write().await;
        if let Some(resource) = resources.get_mut(resource_id) {
            resource.price_per_unit = new_price;
            Ok(())
        } else {
            Err(ResourceError::ResourceNotFound)
        }
    }

    pub async fn create_federation_pool(
        &self,
        federation_id: String,
        access_control: FederationAccessControl,
    ) -> Result<(), ResourceError> {
        let pool = FederationResourcePool {
            federation_id: federation_id.clone(),
            resources: HashMap::new(),
            access_control,
        };
        
        let mut resources = self.resources.write().await;
        resources.insert(federation_id, pool);
        Ok(())
    }

    pub async fn share_with_federation(
        &self,
        source_federation: &str,
        target_federation: &str, 
        resource_id: &str,
        amount: u64,
    ) -> Result<String, ResourceError> {
        let mut resources = self.resources.write().await;
        
        // Verify federation access permissions
        let source_pool = resources.get(source_federation)
            .ok_or(ResourceError::ResourceNotFound)?;
            
        if !source_pool.access_control.allowed_federations.contains(&target_federation.to_string()) {
            return Err(ResourceError::UnauthorizedAction);
        }

        // Create allocation record
        let allocation_id = format!("alloc_{}", uuid::Uuid::new_v4());
        
        // Allocate resources to target federation
        self.allocate(resource_id, target_federation.to_string(), amount).await?;

        Ok(allocation_id)
    }
}

// Commented out ResourceManager trait implementation
/*
#[async_trait]
impl ResourceManager for ResourceAllocationSystem {
    async fn allocate_resources(&self, allocation: icn_federation::ResourceAllocation) -> Result<(), String> {
        self.allocate(&allocation.resource_type, allocation.recipient, allocation.amount)
            .await
            .map_err(|e| e.to_string())
    }

    async fn release_resources(&self, resource_type: &str, amount: u64) -> Result<(), String> {
        // Implementation for releasing resources back to the pool
        Ok(())
    }
}
*/

#[derive(Debug)]
pub enum ResourceError {
    ResourceNotFound,
    InsufficientResources,
    InvalidAllocation,
    UnauthorizedAction,
}
