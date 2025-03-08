use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::Utc;

use crate::{
    Federation, FederationError, ResourceManager,
    ResourcePool, FederationResourcePool, FederationAccessControl
};

/// Represents a cross-federation resource sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingAgreement {
    pub id: String,
    pub source_federation_id: String,
    pub target_federation_id: String,
    pub resource_type: String,
    pub amount: u64,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub terms: String,
    pub status: SharingAgreementStatus,
    pub usage_metrics: ResourceUsageMetrics,
    pub min_reputation_score: i64,
    pub approval_signatures: HashMap<String, String>, // DID -> signature
}

/// Status of a resource sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharingAgreementStatus {
    Proposed,
    Active,
    Expired,
    Terminated,
    Disputed,
}

/// Metrics for tracking resource usage in a sharing agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub total_allocated: u64,
    pub total_used: u64,
    pub last_activity: u64,
}

/// Error types specific to cross-federation operations
#[derive(Debug, thiserror::Error)]
pub enum CrossFederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Unauthorized federation: {0}")]
    UnauthorizedFederation(String),
    
    #[error("Insufficient reputation: required {required}, actual {actual}")]
    InsufficientReputation { required: i64, actual: i64 },
    
    #[error("Insufficient resources: requested {requested}, available {available}")]
    InsufficientResources { requested: u64, available: u64 },
    
    #[error("Invalid agreement: {0}")]
    InvalidAgreement(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Agreement already exists: {0}")]
    AgreementAlreadyExists(String),
}

/// Manages cross-federation resource sharing
pub struct CrossFederationManager {
    agreements: RwLock<HashMap<String, ResourceSharingAgreement>>,
    resource_manager: Arc<dyn ResourceManager>,
}

impl CrossFederationManager {
    /// Create a new CrossFederationManager
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            agreements: RwLock::new(HashMap::new()),
            resource_manager,
        }
    }

    /// Propose a new resource sharing agreement
    pub async fn propose_sharing_agreement(
        &self,
        source_federation: &Federation,
        target_federation_id: String,
        resource_type: String,
        amount: u64,
        duration_seconds: Option<u64>,
        terms: String,
        min_reputation_score: i64,
    ) -> Result<String, FederationError> {
        // Verify the source federation has the requested resource
        let has_resource = source_federation.resources
            .get(&resource_type)
            .map(|pool| pool.available_amount >= amount)
            .unwrap_or(false);
        
        if !has_resource {
            return Err(FederationError::ResourceError(
                format!("Insufficient resources: {}", resource_type)
            ));
        }
        
        // Generate agreement ID
        let agreement_id = format!("share_agreement_{}", uuid::Uuid::new_v4());
        
        // Create agreement
        let now = Utc::now().timestamp() as u64;
        let end_time = duration_seconds.map(|duration| now + duration);
        
        let agreement = ResourceSharingAgreement {
            id: agreement_id.clone(),
            source_federation_id: source_federation.id.clone(),
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
        };
        
        // Store the agreement
        let mut agreements = self.agreements.write().await;
        agreements.insert(agreement_id.clone(), agreement);
        
        Ok(agreement_id)
    }

    /// Accept a proposed resource sharing agreement
    pub async fn accept_sharing_agreement(
        &self,
        agreement_id: &str,
        target_federation: &Federation,
        signature: String,
        signer_did: &str,
    ) -> Result<(), FederationError> {
        let mut agreements = self.agreements.write().await;
        
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| FederationError::NotFound(format!("Agreement not found: {}", agreement_id)))?;
        
        // Verify the agreement is for the correct target federation
        if agreement.target_federation_id != target_federation.id {
            return Err(FederationError::Unauthorized(
                "Federation is not the target of this agreement".to_string()
            ));
        }
        
        // Verify the agreement is still in proposed state
        if agreement.status != SharingAgreementStatus::Proposed {
            return Err(FederationError::InvalidState(
                format!("Agreement is not in proposed state: {:?}", agreement.status)
            ));
        }
        
        // Add signature
        agreement.approval_signatures.insert(signer_did.to_string(), signature);
        
        // If we have sufficient signatures, update status to active
        if agreement.approval_signatures.len() >= 1 { // Simplistic; in production could require multiple signatures
            agreement.status = SharingAgreementStatus::Active;
            
            // Log the acceptance in federation's audit log
            // (This would be implemented based on the Federation struct's audit capabilities)
        }
        
        Ok(())
    }

    /// Allocate resources from a sharing agreement
    pub async fn allocate_shared_resource(
        &self,
        agreement_id: &str,
        requester_did: &str,
        requester_federation: &Federation,
        amount: u64,
    ) -> Result<String, FederationError> {
        // Find the agreement
        let mut agreements = self.agreements.write().await;
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| FederationError::NotFound(format!("Agreement not found: {}", agreement_id)))?;
        
        // Verify agreement is active
        if agreement.status != SharingAgreementStatus::Active {
            return Err(FederationError::InvalidState(
                format!("Agreement is not active: {:?}", agreement.status)
            ));
        }
        
        // Verify federation is the target
        if agreement.target_federation_id != requester_federation.id {
            return Err(FederationError::Unauthorized(
                "Federation is not the target of this agreement".to_string()
            ));
        }
        
        // Check if amount is available (comparing against the agreement's allocation)
        let available = agreement.amount - agreement.usage_metrics.total_allocated;
        if amount > available {
            return Err(FederationError::ResourceError(
                format!("Insufficient resources in sharing agreement: requested {}, available {}", amount, available)
            ));
        }
        
        // Update the usage metrics
        agreement.usage_metrics.total_allocated += amount;
        agreement.usage_metrics.last_activity = Utc::now().timestamp() as u64;
        
        // Allocate the resource via the resource manager
        let allocation_result = self.resource_manager.allocate_resources(
            crate::ResourceAllocation {
                resource_type: agreement.resource_type.clone(),
                amount,
                recipient: requester_did.to_string(),
            }
        ).await;
        
        match allocation_result {
            Ok(_) => {
                // Generate allocation ID
                let allocation_id = format!("shared_alloc_{}_{}", agreement_id, uuid::Uuid::new_v4());
                Ok(allocation_id)
            },
            Err(e) => {
                // If allocation fails, revert the metrics update
                agreement.usage_metrics.total_allocated -= amount;
                Err(FederationError::ResourceError(e))
            }
        }
    }

    /// Release resources back to the sharing agreement
    pub async fn release_shared_resource(
        &self,
        agreement_id: &str,
        allocation_id: &str,
        amount: u64,
    ) -> Result<(), FederationError> {
        let mut agreements = self.agreements.write().await;
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| FederationError::NotFound(format!("Agreement not found: {}", agreement_id)))?;
        
        // Update metrics 
        agreement.usage_metrics.total_used += amount;
        agreement.usage_metrics.last_activity = Utc::now().timestamp() as u64;
        
        // Release through resource manager
        self.resource_manager.release_resources(&agreement.resource_type, amount).await
            .map_err(|e| FederationError::ResourceError(e))
    }

    /// Terminate a sharing agreement
    pub async fn terminate_sharing_agreement(
        &self,
        agreement_id: &str,
        federation_id: &str,
        reason: &str,
    ) -> Result<(), FederationError> {
        let mut agreements = self.agreements.write().await;
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| FederationError::NotFound(format!("Agreement not found: {}", agreement_id)))?;
        
        // Verify federation is involved in the agreement
        if agreement.source_federation_id != federation_id && agreement.target_federation_id != federation_id {
            return Err(FederationError::Unauthorized(
                "Federation is not involved in this agreement".to_string()
            ));
        }
        
        // Update status
        agreement.status = SharingAgreementStatus::Terminated;
        
        // In a more complex implementation:
        // 1. Release any remaining allocations
        // 2. Create a dispute if termination is contested
        // 3. Log the termination with the reason
        
        Ok(())
    }

    /// Get a specific sharing agreement
    pub async fn get_sharing_agreement(&self, agreement_id: &str) -> Option<ResourceSharingAgreement> {
        let agreements = self.agreements.read().await;
        agreements.get(agreement_id).cloned()
    }

    /// Get all sharing agreements for a federation
    pub async fn get_federation_agreements(&self, federation_id: &str) -> Vec<ResourceSharingAgreement> {
        let agreements = self.agreements.read().await;
        agreements.values()
            .filter(|a| a.source_federation_id == federation_id || a.target_federation_id == federation_id)
            .cloned()
            .collect()
    }

    /// Create a federation resource pool configured for cross-federation sharing
    pub async fn create_sharing_resource_pool(
        &self,
        federation_id: String,
        allowed_federations: Vec<String>,
        min_reputation: i64,
        max_allocation_per_federation: u64,
    ) -> Result<FederationResourcePool, FederationError> {
        // Create access control for the resource pool
        let access_control = FederationAccessControl {
            allowed_federations,
            min_reputation,
            max_allocation_per_federation,
        };
        
        // Create the resource pool
        let pool = FederationResourcePool {
            federation_id: federation_id.clone(),
            resources: HashMap::new(),
            access_control,
        };
        
        Ok(pool)
    }
}