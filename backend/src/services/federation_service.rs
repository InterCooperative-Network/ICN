use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;
use thiserror::Error;
use log::{info, error, debug};

use icn_federation::{
    FederationManager, FederationType, FederationTerms, FederationProposal, FederationError, Vote,
    ResourceSharingAgreement, ResourceAllocation, FederationResourceManager, ResourceError,
};
use icn_reputation::ReputationManager;
use icn_resource::ResourceAllocationSystem;
use crate::errors::IcnError;

/// Federation service to manage federation operations
pub struct FederationService {
    federation_manager: Arc<Mutex<FederationManager>>,
    reputation_manager: Arc<dyn ReputationManager>,
    resource_system: Arc<ResourceAllocationSystem>,
    federation_resource_manager: Arc<FederationResourceManager>,
}

#[derive(Error, Debug)]
pub enum FederationServiceError {
    #[error("Federation error: {0}")]
    FederationError(#[from] FederationError),
    
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Resource allocation error: {0}")]
    ResourceAllocationError(String),
}

/// Integration between ResourceManager trait and the ResourceAllocationSystem
struct ResourceSystemAdapter {
    resource_system: Arc<ResourceAllocationSystem>,
    reputation_manager: Arc<dyn ReputationManager>,
}

#[async_trait::async_trait]
impl icn_federation::ResourceProvider for ResourceSystemAdapter {
    async fn has_sufficient_resources(
        &self,
        federation_id: &str,
        resource_type: &str, 
        amount: u64
    ) -> Result<bool, ResourceError> {
        self.resource_system.check_resource_availability(federation_id, resource_type, amount)
            .await
            .map_err(|e| ResourceError::ResourceSystemError(e.to_string()))
    }
    
    async fn get_federation_reputation(
        &self,
        federation_id: &str
    ) -> Result<i64, ResourceError> {
        self.reputation_manager.get_federation_reputation(federation_id)
            .await
            .map_err(|e| ResourceError::ResourceSystemError(format!("Failed to get reputation: {}", e)))
    }
    
    async fn reserve_resources(
        &self,
        federation_id: &str,
        resource_type: &str,
        amount: u64
    ) -> Result<(), ResourceError> {
        self.resource_system.reserve_resources(federation_id, resource_type, amount)
            .await
            .map_err(|e| ResourceError::ResourceSystemError(e.to_string()))
    }
    
    async fn release_resources(
        &self,
        federation_id: &str,
        resource_type: &str,
        amount: u64
    ) -> Result<(), ResourceError> {
        self.resource_system.release_resources(federation_id, resource_type, amount)
            .await
            .map_err(|e| ResourceError::ResourceSystemError(e.to_string()))
    }
}

impl FederationService {
    /// Create a new federation service
    pub fn new(
        federation_manager: Arc<Mutex<FederationManager>>,
        reputation_manager: Arc<dyn ReputationManager>,
        resource_system: Arc<ResourceAllocationSystem>,
    ) -> Self {
        // Create adapter for the resource system
        let resource_provider = Arc::new(ResourceSystemAdapter {
            resource_system: resource_system.clone(),
            reputation_manager: reputation_manager.clone(),
        });
        
        // Create federation resource manager
        let federation_resource_manager = Arc::new(FederationResourceManager::new(resource_provider));
        
        Self {
            federation_manager,
            reputation_manager,
            resource_system,
            federation_resource_manager,
        }
    }

    /// Create a new federation
    pub async fn create_federation(
        &mut self,
        name: String,
        federation_type: FederationType,
        terms: FederationTerms,
        founding_member: String,
    ) -> Result<String, FederationServiceError> {
        let manager = self.federation_manager.lock().await;
        let federation_id = manager.create_federation(name, federation_type, terms, founding_member).await?;
        Ok(federation_id)
    }

    /// Join an existing federation
    pub async fn join_federation(
        &mut self,
        federation_id: &str,
        member_id: &str,
        commitment: Vec<String>,
    ) -> Result<(), FederationServiceError> {
        let manager = self.federation_manager.lock().await;
        manager.join_federation(federation_id, member_id, commitment).await?;
        Ok(())
    }

    /// Submit a proposal to a federation
    pub async fn submit_proposal(
        &mut self,
        federation_id: &str,
        proposal: FederationProposal,
    ) -> Result<(), FederationServiceError> {
        let manager = self.federation_manager.lock().await;
        manager.submit_proposal(federation_id, proposal).await?;
        Ok(())
    }

    /// Vote on a proposal in a federation
    pub async fn vote(
        &mut self,
        federation_id: &str,
        vote: Vote,
    ) -> Result<(), FederationServiceError> {
        let manager = self.federation_manager.lock().await;
        manager.vote(federation_id, vote).await?;
        Ok(())
    }

    // --- Federation Resource Sharing Methods ---

    /// Share resources between federations
    pub async fn share_federation_resources(
        &mut self,
        source_federation_id: String,
        target_federation_id: String,
        resource_type: String,
        amount: u64,
        duration_seconds: Option<u64>,
        terms: String,
        min_reputation_score: i64,
    ) -> Result<String, FederationServiceError> {
        info!(
            "Creating resource sharing agreement: {} -> {}, type: {}, amount: {}", 
            source_federation_id, target_federation_id, resource_type, amount
        );
        
        // Verify federations exist
        let manager = self.federation_manager.lock().await;
        
        // Create the resource sharing agreement
        let agreement_id = self.federation_resource_manager.propose_agreement(
            source_federation_id,
            target_federation_id,
            resource_type,
            amount,
            duration_seconds,
            terms,
            min_reputation_score,
        ).await?;
        
        info!("Resource sharing agreement created: {}", agreement_id);
        Ok(agreement_id)
    }
    
    /// Accept a resource sharing agreement
    pub async fn accept_sharing_agreement(
        &mut self,
        agreement_id: &str,
        target_federation_id: &str,
        signer_did: &str,
        signature: String,
    ) -> Result<(), FederationServiceError> {
        info!("Accepting resource sharing agreement: {}", agreement_id);
        
        self.federation_resource_manager.accept_agreement(
            agreement_id,
            target_federation_id,
            signer_did,
            signature,
        ).await?;
        
        info!("Resource sharing agreement accepted: {}", agreement_id);
        Ok(())
    }
    
    /// Allocate resources from a sharing agreement
    pub async fn allocate_shared_resource(
        &mut self,
        agreement_id: &str,
        requester_did: &str,
        amount: u64,
    ) -> Result<String, FederationServiceError> {
        info!(
            "Allocating resources from agreement {}: requester {}, amount {}", 
            agreement_id, requester_did, amount
        );
        
        // Get agreement to find the requester's federation
        let agreement = self.federation_resource_manager.get_agreement(agreement_id).await
            .ok_or_else(|| ResourceError::AgreementNotFound(agreement_id.to_string()))?;
        
        // Allocate resources
        let allocation_id = self.federation_resource_manager.allocate_from_agreement(
            agreement_id,
            &agreement.target_federation_id, // Federation ID from the agreement
            requester_did,
            amount,
        ).await?;
        
        info!("Resources allocated, allocation ID: {}", allocation_id);
        Ok(allocation_id)
    }
    
    /// Release resources back to a sharing agreement
    pub async fn release_shared_resource(
        &mut self,
        agreement_id: &str,
        allocation_id: &str,
        amount: u64,
    ) -> Result<(), FederationServiceError> {
        info!(
            "Releasing resources to agreement {}: allocation {}, amount {}", 
            agreement_id, allocation_id, amount
        );
        
        self.federation_resource_manager.release_to_agreement(
            agreement_id,
            allocation_id,
            amount,
        ).await?;
        
        info!("Resources released successfully");
        Ok(())
    }
    
    /// Terminate a resource sharing agreement
    pub async fn terminate_sharing_agreement(
        &mut self,
        agreement_id: &str,
        federation_id: &str,
        reason: &str,
    ) -> Result<(), FederationServiceError> {
        info!(
            "Terminating resource sharing agreement {}: federation {}, reason: {}", 
            agreement_id, federation_id, reason
        );
        
        self.federation_resource_manager.terminate_agreement(
            agreement_id,
            federation_id,
            reason,
        ).await?;
        
        info!("Resource sharing agreement terminated");
        Ok(())
    }
    
    /// List all sharing agreements for a federation
    pub async fn list_federation_sharing_agreements(
        &self,
        federation_id: &str,
    ) -> Result<Vec<ResourceSharingAgreement>, FederationServiceError> {
        let agreements = self.federation_resource_manager.get_federation_agreements(federation_id).await;
        Ok(agreements)
    }

    /// Transfer resources between federations (direct transfer not through agreement)
    pub async fn transfer_resource(
        &mut self,
        resource_id: String,
        recipient_id: String,
        amount: u64,
    ) -> Result<(), FederationServiceError> {
        // Implement direct resource transfer
        // This is a simplified implementation and would need more logic in production
        Ok(())
    }

    /// Allocate resource shares within a federation
    pub async fn allocate_resource_shares(
        &mut self,
        resource_id: String,
        shares: HashMap<String, u64>,
    ) -> Result<(), FederationServiceError> {
        // Implement resource share allocation within a federation
        // This is a simplified implementation and would need more logic in production
        Ok(())
    }
}