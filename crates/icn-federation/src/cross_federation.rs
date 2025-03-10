use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::Utc;

use crate::{
    Federation, FederationError, ResourceManager,
    ResourcePool, FederationResourcePool, FederationAccessControl
};

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use icn_types::FederationId;
use icn_crypto::KeyPair;

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

/// Error types for cross-federation operations
#[derive(Error, Debug)]
pub enum CrossFederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(FederationId),
    #[error("Unauthorized federation access: {0}")]
    UnauthorizedAccess(FederationId),
    #[error("Message validation failed: {0}")]
    MessageValidationFailed(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

/// Types of cross-federation messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request to initiate collaboration between federations
    CollaborationRequest,
    /// Response to a collaboration request
    CollaborationResponse,
    /// Resource sharing proposal between federations
    ResourceSharingProposal,
    /// Notification of resource usage
    ResourceUsageUpdate,
    /// Dispute raised between federations
    DisputeNotification,
    /// Governance proposal that affects multiple federations
    GovernanceProposal,
    /// Vote on a cross-federation proposal
    FederationVote,
    /// Generic notification or update
    Notification,
}

/// A secure message exchanged between federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFederationMessage {
    /// Unique identifier for this message
    pub id: String,
    /// Sender federation ID
    pub sender: FederationId,
    /// Recipient federation ID 
    pub recipient: FederationId,
    /// Type of message
    pub message_type: MessageType,
    /// Message content in encrypted format
    pub encrypted_content: Vec<u8>,
    /// Time the message was created
    pub timestamp: DateTime<Utc>,
    /// Digital signature of the message content
    pub signature: String,
    /// Threshold required for multi-signature messages (optional)
    pub threshold: Option<u32>,
    /// Additional signatures for multi-signature messages (optional)
    pub additional_signatures: Option<HashMap<String, String>>,
    /// References to related messages
    pub references: Vec<String>,
    /// Time-to-live in seconds
    pub ttl: u64,
}

/// Status of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message has been sent but not yet processed
    Pending,
    /// Message has been delivered to the recipient
    Delivered,
    /// Message has been processed by the recipient
    Processed,
    /// Message handling is waiting for additional actions
    Waiting,
    /// Message has failed to be delivered or processed
    Failed,
    /// Message has expired
    Expired,
}

/// Manager for cross-federation communication
pub struct CrossFederationMessenger {
    /// Federation ID of this instance
    federation_id: FederationId,
    /// Cryptographic key pair for message signing
    key_pair: KeyPair,
    /// Message registry
    message_registry: Arc<RwLock<HashMap<String, MessageStatus>>>,
    /// Federation registry mapping federation ID to their public keys
    federation_registry: Arc<RwLock<HashMap<FederationId, Vec<u8>>>>,
    /// Queue of outgoing messages
    outgoing_queue: Arc<Mutex<Vec<CrossFederationMessage>>>,
    /// Queue of incoming messages
    incoming_queue: Arc<Mutex<Vec<CrossFederationMessage>>>,
    /// Trusted federation relationships
    trusted_federations: Arc<RwLock<HashMap<FederationId, TrustLevel>>>,
}

/// Trust level between federations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Full trust with all permissions
    FullTrust,
    /// Trust for specific operations only
    LimitedTrust(Vec<String>),
    /// Trust with approval required for each operation
    ApprovalRequired,
    /// No trust established
    Untrusted,
}

impl CrossFederationMessenger {
    /// Create a new cross-federation messenger
    pub fn new(federation_id: FederationId, key_pair: KeyPair) -> Self {
        Self {
            federation_id,
            key_pair,
            message_registry: Arc::new(RwLock::new(HashMap::new())),
            federation_registry: Arc::new(RwLock::new(HashMap::new())),
            outgoing_queue: Arc::new(Mutex::new(Vec::new())),
            incoming_queue: Arc::new(Mutex::new(Vec::new())),
            trusted_federations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a federation and its public key
    pub async fn register_federation(&self, federation_id: FederationId, public_key: Vec<u8>) {
        let mut registry = self.federation_registry.write().await;
        registry.insert(federation_id, public_key);
    }

    /// Set trust level for a federation
    pub async fn set_trust_level(&self, federation_id: FederationId, trust_level: TrustLevel) {
        let mut trusted = self.trusted_federations.write().await;
        trusted.insert(federation_id, trust_level);
    }

    /// Check if a federation is trusted for a specific operation
    pub async fn is_trusted_for_operation(&self, federation_id: &FederationId, operation: &str) -> bool {
        let trusted = self.trusted_federations.read().await;
        
        match trusted.get(federation_id) {
            Some(TrustLevel::FullTrust) => true,
            Some(TrustLevel::LimitedTrust(permissions)) => permissions.contains(&operation.to_string()),
            Some(TrustLevel::ApprovalRequired) => false, // Requires explicit approval
            Some(TrustLevel::Untrusted) | None => false,
        }
    }

    /// Create a new cross-federation message
    pub fn create_message(
        &self,
        recipient: FederationId,
        message_type: MessageType,
        content: &[u8],
        references: Vec<String>,
        ttl: u64,
    ) -> Result<CrossFederationMessage, CrossFederationError> {
        // Encrypt content using recipient's public key
        let encrypted_content = self.encrypt_for_recipient(&recipient, content)?;
        
        // Generate a unique message ID
        let message_id = Uuid::new_v4().to_string();
        
        // Create message timestamp
        let timestamp = Utc::now();
        
        // Create the signature
        let signature_data = format!(
            "{}:{}:{}:{}:{}",
            message_id, self.federation_id, recipient, timestamp, hex::encode(&encrypted_content)
        );
        
        let signature = match self.key_pair.sign(signature_data.as_bytes()) {
            Ok(sig) => hex::encode(sig),
            Err(_) => return Err(CrossFederationError::CryptoError("Failed to sign message".to_string())),
        };
        
        Ok(CrossFederationMessage {
            id: message_id,
            sender: self.federation_id.clone(),
            recipient,
            message_type,
            encrypted_content,
            timestamp,
            signature,
            threshold: None,
            additional_signatures: None,
            references,
            ttl,
        })
    }

    /// Encrypt content for a specific recipient federation
    fn encrypt_for_recipient(&self, recipient_id: &FederationId, content: &[u8]) -> Result<Vec<u8>, CrossFederationError> {
        // In a real implementation, this would use the recipient's public key to encrypt
        // For now, we're using a placeholder implementation
        let mut encrypted = Vec::new();
        encrypted.extend_from_slice(b"ENCRYPTED:"); // Prefix to simulate encryption
        encrypted.extend_from_slice(content);
        
        Ok(encrypted)
    }

    /// Decrypt content meant for this federation
    fn decrypt_message(&self, encrypted_content: &[u8]) -> Result<Vec<u8>, CrossFederationError> {
        // In a real implementation, this would use this federation's private key to decrypt
        // For now, we're using a placeholder implementation
        if encrypted_content.len() < 10 || &encrypted_content[0..10] != b"ENCRYPTED:" {
            return Err(CrossFederationError::CryptoError("Invalid encrypted format".to_string()));
        }
        
        Ok(encrypted_content[10..].to_vec())
    }

    /// Queue a message to be sent
    pub async fn queue_message(&self, message: CrossFederationMessage) -> Result<(), CrossFederationError> {
        // Add to outgoing queue
        let mut queue = self.outgoing_queue.lock().await;
        queue.push(message.clone());
        
        // Update message registry
        let mut registry = self.message_registry.write().await;
        registry.insert(message.id, MessageStatus::Pending);
        
        Ok(())
    }

    /// Send all queued outgoing messages
    pub async fn send_queued_messages(&self) -> Result<usize, CrossFederationError> {
        let mut queue = self.outgoing_queue.lock().await;
        let message_count = queue.len();
        
        for message in queue.iter() {
            // In a real implementation, we'd actually send the message over the network
            // For now, just update the message status
            let mut registry = self.message_registry.write().await;
            registry.insert(message.id.clone(), MessageStatus::Delivered);
        }
        
        // Clear the queue
        queue.clear();
        
        Ok(message_count)
    }

    /// Process a received message
    pub async fn process_message(&self, message: CrossFederationMessage) -> Result<Vec<u8>, CrossFederationError> {
        // Verify the message signature
        self.verify_message_signature(&message).await?;
        
        // Check if the sender is trusted
        if !self.is_trusted_for_operation(&message.sender, "receive_message").await {
            return Err(CrossFederationError::UnauthorizedAccess(message.sender));
        }
        
        // Decrypt the content
        let decrypted_content = self.decrypt_message(&message.encrypted_content)?;
        
        // Add to registry
        let mut registry = self.message_registry.write().await;
        registry.insert(message.id.clone(), MessageStatus::Processed);
        
        Ok(decrypted_content)
    }

    /// Queue a received message for processing
    pub async fn queue_received_message(&self, message: CrossFederationMessage) -> Result<(), CrossFederationError> {
        // Basic validation first
        if message.recipient != self.federation_id {
            return Err(CrossFederationError::UnauthorizedAccess(message.recipient));
        }
        
        // Add to incoming queue
        let mut queue = self.incoming_queue.lock().await;
        queue.push(message.clone());
        
        // Update registry
        let mut registry = self.message_registry.write().await;
        registry.insert(message.id, MessageStatus::Pending);
        
        Ok(())
    }

    /// Process all queued incoming messages
    pub async fn process_queued_messages(&self) -> Result<Vec<Vec<u8>>, CrossFederationError> {
        let mut queue = self.incoming_queue.lock().await;
        let mut results = Vec::new();
        
        for message in std::mem::take(&mut *queue) {
            match self.process_message(message).await {
                Ok(content) => results.push(content),
                Err(err) => {
                    // Log error but continue processing other messages
                    eprintln!("Error processing message: {:?}", err);
                }
            }
        }
        
        Ok(results)
    }

    /// Verify a message signature
    async fn verify_message_signature(&self, message: &CrossFederationMessage) -> Result<bool, CrossFederationError> {
        // Get the sender's public key
        let registry = self.federation_registry.read().await;
        let sender_public_key = registry.get(&message.sender)
            .ok_or_else(|| CrossFederationError::FederationNotFound(message.sender.clone()))?;
        
        // Recreate the signature data
        let signature_data = format!(
            "{}:{}:{}:{}:{}",
            message.id, message.sender, message.recipient, message.timestamp, hex::encode(&message.encrypted_content)
        );
        
        // Verify the signature
        // In a real implementation, we'd use proper cryptographic verification
        // For now, just check if it exists and isn't empty
        if message.signature.is_empty() {
            return Err(CrossFederationError::MessageValidationFailed("Empty signature".to_string()));
        }
        
        Ok(true)
    }

    /// Get the status of a message
    pub async fn get_message_status(&self, message_id: &str) -> Option<MessageStatus> {
        let registry = self.message_registry.read().await;
        registry.get(message_id).cloned()
    }

    /// Start the background message processor
    pub async fn start_background_processor(messenger: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                // Process incoming messages
                if let Err(err) = messenger.process_queued_messages().await {
                    eprintln!("Error processing queued messages: {:?}", err);
                }
                
                // Send outgoing messages
                if let Err(err) = messenger.send_queued_messages().await {
                    eprintln!("Error sending queued messages: {:?}", err);
                }
                
                // Sleep for a bit before next cycle
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }
}