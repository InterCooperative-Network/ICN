use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

/// Status of a resource sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharingAgreementStatus {
    /// Agreement has been proposed but not yet activated
    Proposed,
    /// Agreement is active and resources can be allocated
    Active,
    /// Agreement has expired based on its duration
    Expired,
    /// Agreement has been manually terminated
    Terminated,
    /// Agreement is in dispute resolution
    Disputed,
}

/// Metrics for tracking resource usage in a sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    /// Total amount of resources allocated from this agreement
    pub total_allocated: u64,
    /// Total amount of resources that have been used and released
    pub total_used: u64,
    /// Timestamp of the last activity on this agreement
    pub last_activity: u64,
}

/// Represents a federation resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Unique identifier for this allocation
    pub id: String,
    /// ID of the agreement this allocation is from
    pub agreement_id: String,
    /// DID of the requester who received the allocation
    pub requester_did: String,
    /// Amount of resources allocated
    pub amount: u64,
    /// When the allocation was created
    pub allocated_at: u64,
    /// When the allocation expires (if applicable)
    pub expires_at: Option<u64>,
}

/// Represents a cross-federation resource sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingAgreement {
    /// Unique identifier for this agreement
    pub id: String,
    /// ID of the source federation that owns the resources
    pub source_federation_id: String,
    /// ID of the target federation receiving the resources
    pub target_federation_id: String,
    /// Type of resource being shared
    pub resource_type: String,
    /// Total amount of resources available in this agreement
    pub amount: u64,
    /// When the agreement started
    pub start_time: u64,
    /// When the agreement ends (if it has a set duration)
    pub end_time: Option<u64>,
    /// Terms of the resource sharing agreement
    pub terms: String,
    /// Current status of the agreement
    pub status: SharingAgreementStatus,
    /// Usage metrics for monitoring and accounting
    pub usage_metrics: ResourceUsageMetrics,
    /// Minimum reputation score required to access resources
    pub min_reputation_score: i64,
    /// Signatures from approving parties (DID -> signature)
    pub approval_signatures: HashMap<String, String>,
    /// Current allocations from this agreement (allocation ID -> allocation)
    pub active_allocations: HashMap<String, ResourceAllocation>,
}

impl ResourceSharingAgreement {
    /// Create a new resource sharing agreement
    pub fn new(
        source_federation_id: String,
        target_federation_id: String,
        resource_type: String,
        amount: u64,
        duration_seconds: Option<u64>,
        terms: String,
        min_reputation_score: i64,
    ) -> Self {
        let id = format!("share_agreement_{}", uuid::Uuid::new_v4());
        let now = Utc::now().timestamp() as u64;
        let end_time = duration_seconds.map(|duration| now + duration);
        
        Self {
            id,
            source_federation_id,
            target_federation_id,
            resource_type,
            amount,
            start_time: now,
            end_time,
            terms,
            status: SharingAgreementStatus::Proposed,
            usage_metrics: ResourceUsageMetrics {
                total_allocated: 0,
                total_used: 0,
                last_activity: now,
            },
            min_reputation_score,
            approval_signatures: HashMap::new(),
            active_allocations: HashMap::new(),
        }
    }
    
    /// Check if the agreement is currently valid for use
    pub fn is_valid(&self) -> bool {
        // Must be in active status
        if self.status != SharingAgreementStatus::Active {
            return false;
        }
        
        // Check if the agreement has expired
        if let Some(end_time) = self.end_time {
            let now = Utc::now().timestamp() as u64;
            if now > end_time {
                return false;
            }
        }
        
        true
    }
    
    /// Calculate how many resources are available for allocation
    pub fn available_resources(&self) -> u64 {
        if self.status != SharingAgreementStatus::Active {
            return 0;
        }
        
        if self.amount <= self.usage_metrics.total_allocated {
            return 0;
        }
        
        self.amount - self.usage_metrics.total_allocated
    }
    
    /// Create a new allocation from this agreement
    pub fn allocate(
        &mut self,
        requester_did: &str,
        amount: u64,
    ) -> Result<String, String> {
        // Check if agreement is valid
        if !self.is_valid() {
            return Err(format!("Agreement is not active: {:?}", self.status));
        }
        
        // Check if enough resources are available
        let available = self.available_resources();
        if amount > available {
            return Err(format!("Insufficient resources: requested {}, available {}", amount, available));
        }
        
        // Create allocation
        let allocation_id = format!("alloc_{}", uuid::Uuid::new_v4());
        let now = Utc::now().timestamp() as u64;
        
        let allocation = ResourceAllocation {
            id: allocation_id.clone(),
            agreement_id: self.id.clone(),
            requester_did: requester_did.to_string(),
            amount,
            allocated_at: now,
            expires_at: None,
        };
        
        // Update metrics
        self.usage_metrics.total_allocated += amount;
        self.usage_metrics.last_activity = now;
        
        // Store allocation
        self.active_allocations.insert(allocation_id.clone(), allocation);
        
        Ok(allocation_id)
    }
    
    /// Release resources from an allocation back to the pool
    pub fn release(&mut self, allocation_id: &str, amount: u64) -> Result<(), String> {
        // Find the allocation
        let allocation = self.active_allocations.get(allocation_id)
            .ok_or_else(|| format!("Allocation not found: {}", allocation_id))?;
        
        // Check if enough resources are allocated
        if amount > allocation.amount {
            return Err(format!(
                "Cannot release more than allocated: requested {}, allocated {}",
                amount, allocation.amount
            ));
        }
        
        // Update metrics
        self.usage_metrics.total_used += amount;
        self.usage_metrics.total_allocated -= amount;
        self.usage_metrics.last_activity = Utc::now().timestamp() as u64;
        
        // If the entire allocation is released, remove it
        if amount == allocation.amount {
            self.active_allocations.remove(allocation_id);
        } else {
            // Otherwise update the allocation amount
            let mut updated_allocation = allocation.clone();
            updated_allocation.amount -= amount;
            self.active_allocations.insert(allocation_id.to_string(), updated_allocation);
        }
        
        Ok(())
    }
    
    /// Accept the agreement, updating its status to Active
    pub fn accept(&mut self, signer_did: &str, signature: String) -> Result<(), String> {
        if self.status != SharingAgreementStatus::Proposed {
            return Err(format!("Agreement is not in proposed state: {:?}", self.status));
        }
        
        // Add signature
        self.approval_signatures.insert(signer_did.to_string(), signature);
        
        // In a real system, we might require multiple signatures
        // For simplicity, we'll activate with just one signature
        self.status = SharingAgreementStatus::Active;
        
        Ok(())
    }
    
    /// Terminate the agreement
    pub fn terminate(&mut self, reason: &str) -> Result<(), String> {
        if self.status != SharingAgreementStatus::Active {
            return Err(format!("Agreement is not active: {:?}", self.status));
        }
        
        self.status = SharingAgreementStatus::Terminated;
        
        // In a real system, we would log the termination reason
        // and handle active allocations
        
        Ok(())
    }
}