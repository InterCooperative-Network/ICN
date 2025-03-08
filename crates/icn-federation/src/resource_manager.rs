use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use thiserror::Error;

use crate::resource_sharing::{ResourceSharingAgreement, SharingAgreementStatus};

/// Error types for federation resource operations
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Agreement not found: {0}")]
    AgreementNotFound(String),
    
    #[error("Insufficient resources: requested {requested}, available {available}")]
    InsufficientResources { requested: u64, available: u64 },
    
    #[error("Invalid agreement state: {0}")]
    InvalidState(String),
    
    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),
    
    #[error("Insufficient reputation: required {required}, actual {actual}")]
    InsufficientReputation { required: i64, actual: i64 },
    
    #[error("Resource system error: {0}")]
    ResourceSystemError(String),
}

/// Defines methods for managing resource operations
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// Check if a federation has sufficient resources of a given type
    async fn has_sufficient_resources(
        &self,
        federation_id: &str, 
        resource_type: &str, 
        amount: u64
    ) -> Result<bool, ResourceError>;
    
    /// Check a federation's reputation score
    async fn get_federation_reputation(
        &self, 
        federation_id: &str
    ) -> Result<i64, ResourceError>;
    
    /// Reserve resources from a federation
    async fn reserve_resources(
        &self,
        federation_id: &str,
        resource_type: &str,
        amount: u64
    ) -> Result<(), ResourceError>;
    
    /// Release resources back to a federation
    async fn release_resources(
        &self,
        federation_id: &str,
        resource_type: &str,
        amount: u64
    ) -> Result<(), ResourceError>;
}

/// Manages cross-federation resource sharing agreements
pub struct FederationResourceManager {
    /// Store of all resource sharing agreements
    agreements: RwLock<HashMap<String, ResourceSharingAgreement>>,
    
    /// Provider of resource and reputation operations
    resource_provider: Arc<dyn ResourceProvider>,
}

impl FederationResourceManager {
    /// Create a new FederationResourceManager
    pub fn new(resource_provider: Arc<dyn ResourceProvider>) -> Self {
        Self {
            agreements: RwLock::new(HashMap::new()),
            resource_provider,
        }
    }
    
    /// Propose a new resource sharing agreement
    pub async fn propose_agreement(
        &self,
        source_federation_id: String,
        target_federation_id: String,
        resource_type: String,
        amount: u64,
        duration_seconds: Option<u64>,
        terms: String,
        min_reputation_score: i64,
    ) -> Result<String, ResourceError> {
        // Check if source federation has sufficient resources
        let has_resources = self.resource_provider
            .has_sufficient_resources(&source_federation_id, &resource_type, amount)
            .await?;
        
        if !has_resources {
            return Err(ResourceError::InsufficientResources {
                requested: amount,
                available: 0, // Actual amount unknown at this level
            });
        }
        
        // Create the agreement
        let agreement = ResourceSharingAgreement::new(
            source_federation_id,
            target_federation_id,
            resource_type,
            amount,
            duration_seconds,
            terms,
            min_reputation_score,
        );
        
        let agreement_id = agreement.id.clone();
        
        // Store the agreement
        let mut agreements = self.agreements.write().await;
        agreements.insert(agreement_id.clone(), agreement);
        
        Ok(agreement_id)
    }
    
    /// Accept a proposed sharing agreement
    pub async fn accept_agreement(
        &self,
        agreement_id: &str,
        target_federation_id: &str,
        signer_did: &str,
        signature: String,
    ) -> Result<(), ResourceError> {
        let mut agreements = self.agreements.write().await;
        
        // Find agreement
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| ResourceError::AgreementNotFound(agreement_id.to_string()))?;
        
        // Verify the federation matches
        if agreement.target_federation_id != target_federation_id {
            return Err(ResourceError::Unauthorized(
                "Federation is not the target of this agreement".to_string()
            ));
        }
        
        // Verify federation meets reputation requirements
        let reputation = self.resource_provider
            .get_federation_reputation(target_federation_id)
            .await?;
        
        if reputation < agreement.min_reputation_score {
            return Err(ResourceError::InsufficientReputation {
                required: agreement.min_reputation_score,
                actual: reputation,
            });
        }
        
        // Accept the agreement
        agreement.accept(signer_did, signature)
            .map_err(|e| ResourceError::InvalidState(e))?;
        
        // Reserve the resources from the source federation
        self.resource_provider
            .reserve_resources(
                &agreement.source_federation_id,
                &agreement.resource_type,
                agreement.amount
            )
            .await?;
        
        Ok(())
    }
    
    /// Allocate resources from a sharing agreement
    pub async fn allocate_from_agreement(
        &self,
        agreement_id: &str,
        requester_federation_id: &str,
        requester_did: &str,
        amount: u64,
    ) -> Result<String, ResourceError> {
        let mut agreements = self.agreements.write().await;
        
        // Find agreement
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| ResourceError::AgreementNotFound(agreement_id.to_string()))?;
        
        // Verify federation matches
        if agreement.target_federation_id != requester_federation_id {
            return Err(ResourceError::Unauthorized(
                "Federation is not authorized to use this agreement".to_string()
            ));
        }
        
        // Allocate resources
        let allocation_id = agreement.allocate(requester_did, amount)
            .map_err(|e| ResourceError::InvalidState(e))?;
        
        Ok(allocation_id)
    }
    
    /// Release resources back to a sharing agreement
    pub async fn release_to_agreement(
        &self,
        agreement_id: &str,
        allocation_id: &str,
        amount: u64,
    ) -> Result<(), ResourceError> {
        let mut agreements = self.agreements.write().await;
        
        // Find agreement
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| ResourceError::AgreementNotFound(agreement_id.to_string()))?;
        
        // Release resources
        agreement.release(allocation_id, amount)
            .map_err(|e| ResourceError::InvalidState(e))?;
        
        Ok(())
    }
    
    /// Terminate a sharing agreement
    pub async fn terminate_agreement(
        &self,
        agreement_id: &str,
        federation_id: &str,
        reason: &str,
    ) -> Result<(), ResourceError> {
        let mut agreements = self.agreements.write().await;
        
        // Find agreement
        let agreement = agreements.get_mut(agreement_id)
            .ok_or_else(|| ResourceError::AgreementNotFound(agreement_id.to_string()))?;
        
        // Verify federation is involved
        if agreement.source_federation_id != federation_id && agreement.target_federation_id != federation_id {
            return Err(ResourceError::Unauthorized(
                "Federation is not involved in this agreement".to_string()
            ));
        }
        
        // Terminate agreement
        agreement.terminate(reason)
            .map_err(|e| ResourceError::InvalidState(e))?;
        
        // If there are any unused resources, release them back to the source federation
        let unused = agreement.amount - agreement.usage_metrics.total_used;
        if unused > 0 {
            self.resource_provider
                .release_resources(
                    &agreement.source_federation_id,
                    &agreement.resource_type,
                    unused
                )
                .await?;
        }
        
        Ok(())
    }
    
    /// Get all agreements for a federation (as source or target)
    pub async fn get_federation_agreements(
        &self,
        federation_id: &str,
    ) -> Vec<ResourceSharingAgreement> {
        let agreements = self.agreements.read().await;
        
        agreements.values()
            .filter(|a| a.source_federation_id == federation_id || a.target_federation_id == federation_id)
            .cloned()
            .collect()
    }
    
    /// Get a specific agreement by ID
    pub async fn get_agreement(
        &self,
        agreement_id: &str,
    ) -> Option<ResourceSharingAgreement> {
        let agreements = self.agreements.read().await;
        agreements.get(agreement_id).cloned()
    }
}