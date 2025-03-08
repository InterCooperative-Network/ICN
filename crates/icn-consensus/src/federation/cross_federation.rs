use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use icn_crypto::{KeyPair, Algorithm, hash};
use super::{Federation, FederationError};

/// Error types for cross-federation communication
#[derive(Error, Debug)]
pub enum CrossFederationError {
    #[error("Message verification failed: {0}")]
    VerificationFailed(String),

    #[error("Federation not found: {0}")]
    FederationNotFound(String),

    #[error("Not authorized: {0}")]
    NotAuthorized(String),

    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

/// Types of cross-federation messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    ResourceRequest,
    ResourceOffer,
    CollaborationProposal,
    CollaborationResponse,
    JointValidation,
    InfoExchange,
    ConflictResolution,
    FederationStatus,
}

/// Cross-federation message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Unique message ID
    pub id: String,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Source federation ID
    pub source_federation_id: String,
    
    /// Destination federation ID
    pub destination_federation_id: String,
    
    /// Message content (JSON serialized)
    pub content: String,
    
    /// Timestamp when message was created
    pub created_at: u64,
    
    /// Message expiration timestamp
    pub expires_at: Option<u64>,
    
    /// Digital signature from source federation
    pub signature: String,
    
    /// Previous message ID in conversation (if applicable)
    pub previous_message_id: Option<String>,
}

/// Cross-federation protocol adapter
pub struct CrossFederationProtocol {
    /// Local federation ID
    federation_id: String,
    
    /// Federation keypair for signing messages
    keypair: KeyPair,
    
    /// Known federations and their public keys
    federation_keys: HashMap<String, Vec<u8>>,
    
    /// Outgoing message queue
    outgoing_messages: VecDeque<FederationMessage>,
    
    /// Received messages awaiting processing
    incoming_messages: VecDeque<FederationMessage>,
    
    /// Message history by conversation
    message_history: HashMap<String, Vec<FederationMessage>>,
    
    /// Trust levels with other federations (0-100)
    trust_levels: HashMap<String, u8>,
    
    /// Active federations we can communicate with
    active_federations: HashMap<String, Arc<RwLock<Federation>>>,
}

impl CrossFederationProtocol {
    /// Create a new cross-federation protocol instance
    pub fn new(federation_id: String, keypair: KeyPair) -> Self {
        Self {
            federation_id,
            keypair,
            federation_keys: HashMap::new(),
            outgoing_messages: VecDeque::new(),
            incoming_messages: VecDeque::new(),
            message_history: HashMap::new(),
            trust_levels: HashMap::new(),
            active_federations: HashMap::new(),
        }
    }
    
    /// Register a known federation and its public key
    pub fn register_federation(&mut self, federation_id: String, public_key: Vec<u8>, initial_trust: u8) {
        self.federation_keys.insert(federation_id.clone(), public_key);
        self.trust_levels.insert(federation_id, initial_trust);
    }
    
    /// Create a new message to send to another federation
    pub fn create_message(
        &self,
        destination_federation_id: String,
        message_type: MessageType,
        content: String,
        previous_message_id: Option<String>,
        expires_in_seconds: Option<u64>,
    ) -> Result<FederationMessage, CrossFederationError> {
        // Check if we know this federation
        if !self.federation_keys.contains_key(&destination_federation_id) {
            return Err(CrossFederationError::FederationNotFound(destination_federation_id));
        }
        
        let now = chrono::Utc::now().timestamp() as u64;
        let expires_at = expires_in_seconds.map(|s| now + s);
        
        // Create message ID
        let message_id = format!("msg_{}", uuid::Uuid::new_v4());
        
        // Prepare message for signing
        let message_content = format!(
            "{}:{}:{}:{}:{}",
            message_id,
            self.federation_id,
            destination_federation_id,
            content,
            now
        );
        
        // Sign message
        let signature = self.keypair.sign(message_content.as_bytes())
            .map_err(|e| CrossFederationError::ProtocolError(e.to_string()))?;
            
        let signature_hex = hex::encode(signature);
        
        let message = FederationMessage {
            id: message_id,
            message_type,
            source_federation_id: self.federation_id.clone(),
            destination_federation_id,
            content,
            created_at: now,
            expires_at,
            signature: signature_hex,
            previous_message_id,
        };
        
        Ok(message)
    }
    
    /// Queue a message to be sent
    pub fn queue_message(&mut self, message: FederationMessage) {
        // Add to outgoing queue
        self.outgoing_messages.push_back(message.clone());
        
        // Add to message history
        let conversation_id = message.previous_message_id
            .clone()
            .unwrap_or_else(|| message.id.clone());
            
        self.message_history
            .entry(conversation_id)
            .or_insert_with(Vec::new)
            .push(message);
    }
    
    /// Verify an incoming message signature
    pub fn verify_message(&self, message: &FederationMessage) -> Result<bool, CrossFederationError> {
        // Get sender's public key
        let public_key = self.federation_keys
            .get(&message.source_federation_id)
            .ok_or_else(|| CrossFederationError::FederationNotFound(
                message.source_federation_id.clone()
            ))?;
        
        // Check if message has expired
        if let Some(expires_at) = message.expires_at {
            let now = chrono::Utc::now().timestamp() as u64;
            if expires_at < now {
                return Err(CrossFederationError::VerificationFailed(
                    "Message has expired".to_string()
                ));
            }
        }
        
        // Reconstruct message content for verification
        let message_content = format!(
            "{}:{}:{}:{}:{}",
            message.id,
            message.source_federation_id,
            message.destination_federation_id,
            message.content,
            message.created_at
        );
        
        // Decode signature
        let signature = hex::decode(&message.signature)
            .map_err(|_| CrossFederationError::VerificationFailed(
                "Invalid signature format".to_string()
            ))?;
        
        // Create temporary keypair for verification
        let keypair = KeyPair {
            public_key: public_key.clone(),
            private_key: vec![],
            algorithm: Algorithm::Ed25519, // Assuming Ed25519 for signatures
        };
        
        // Verify signature
        match keypair.verify(message_content.as_bytes(), &signature) {
            Ok(true) => Ok(true),
            Ok(false) => Err(CrossFederationError::VerificationFailed(
                "Signature verification failed".to_string()
            )),
            Err(e) => Err(CrossFederationError::ProtocolError(e.to_string())),
        }
    }
    
    /// Receive and process an incoming message
    pub fn receive_message(&mut self, message: FederationMessage) -> Result<(), CrossFederationError> {
        // Verify the message is intended for us
        if message.destination_federation_id != self.federation_id {
            return Err(CrossFederationError::NotAuthorized(
                "Message not intended for this federation".to_string()
            ));
        }
        
        // Verify message signature
        self.verify_message(&message)?;
        
        // Add to incoming queue
        self.incoming_messages.push_back(message.clone());
        
        // Add to message history
        let conversation_id = message.previous_message_id
            .clone()
            .unwrap_or_else(|| message.id.clone());
            
        self.message_history
            .entry(conversation_id)
            .or_insert_with(Vec::new)
            .push(message);
        
        Ok(())
    }
    
    /// Process the next incoming message
    pub async fn process_next_message(&mut self) -> Option<Result<FederationMessage, CrossFederationError>> {
        let message = self.incoming_messages.pop_front()?;
        
        match message.message_type {
            MessageType::ResourceRequest => {
                // Process resource request
                self.handle_resource_request(&message).await
                    .map(|_| message.clone())
                    .map_err(|e| e)
            }
            MessageType::CollaborationProposal => {
                // Process collaboration proposal
                self.handle_collaboration_proposal(&message).await
                    .map(|_| message.clone())
                    .map_err(|e| e)
            }
            MessageType::JointValidation => {
                // Process joint validation
                self.handle_joint_validation(&message).await
                    .map(|_| message.clone())
                    .map_err(|e| e)
            }
            // Handle other message types
            _ => {
                // Default message handling
                Ok(message)
            }
        }.into()
    }
    
    /// Handle resource request message
    async fn handle_resource_request(&mut self, message: &FederationMessage) -> Result<(), CrossFederationError> {
        // Parse request content
        let request: ResourceRequest = serde_json::from_str(&message.content)
            .map_err(|e| CrossFederationError::ProtocolError(format!(
                "Invalid resource request format: {}", e
            )))?;
        
        // Check trust level
        let trust_level = self.trust_levels.get(&message.source_federation_id)
            .cloned()
            .unwrap_or(0);
            
        if trust_level < 50 {
            // Not enough trust to handle this request
            let response = ResourceResponse {
                request_id: request.request_id,
                approved: false,
                reason: Some("Insufficient trust level".to_string()),
                resource_id: None,
                access_conditions: None,
            };
            
            // Create response message
            self.respond_to_message(message, serde_json::to_string(&response).unwrap())?;
            return Ok(());
        }
        
        // TODO: Actual resource handling logic would go here
        let response = ResourceResponse {
            request_id: request.request_id,
            approved: true,
            reason: None,
            resource_id: Some(uuid::Uuid::new_v4().to_string()),
            access_conditions: Some("time_limit:3600".to_string()),
        };
        
        // Create response message
        self.respond_to_message(message, serde_json::to_string(&response).unwrap())?;
        
        Ok(())
    }
    
    /// Handle collaboration proposal
    async fn handle_collaboration_proposal(&mut self, message: &FederationMessage) -> Result<(), CrossFederationError> {
        // Parse proposal content
        let proposal: CollaborationProposal = serde_json::from_str(&message.content)
            .map_err(|e| CrossFederationError::ProtocolError(format!(
                "Invalid collaboration proposal format: {}", e
            )))?;
        
        // Check if we have the federation
        let federation = self.active_federations.get(&self.federation_id)
            .ok_or_else(|| CrossFederationError::FederationNotFound(
                "Local federation not found".to_string()
            ))?;
            
        let fed = federation.read().await;
        
        // Create token for proposal if approved
        let token = format!("collab_{}", uuid::Uuid::new_v4());
        
        let response = CollaborationResponse {
            proposal_id: proposal.proposal_id,
            approved: true,
            token: Some(token),
            start_time: chrono::Utc::now().timestamp() as u64,
            duration: proposal.requested_duration,
            terms: proposal.terms,
        };
        
        // Create response message
        self.respond_to_message(message, serde_json::to_string(&response).unwrap())?;
        
        Ok(())
    }
    
    /// Handle joint validation
    async fn handle_joint_validation(&mut self, message: &FederationMessage) -> Result<(), CrossFederationError> {
        // Parse validation content
        let validation: JointValidation = serde_json::from_str(&message.content)
            .map_err(|e| CrossFederationError::ProtocolError(format!(
                "Invalid joint validation format: {}", e
            )))?;
        
        // Validate hash provided
        let computed_hash = hash(&validation.data);
        let expected_hash = validation.expected_hash;
        
        let response = ValidationResponse {
            validation_id: validation.validation_id,
            confirmed: computed_hash == expected_hash,
            validator_signature: if computed_hash == expected_hash {
                // Sign the hash if it matches
                let signature = self.keypair.sign(computed_hash.as_bytes())
                    .map_err(|e| CrossFederationError::ProtocolError(e.to_string()))?;
                Some(hex::encode(signature))
            } else {
                None
            },
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        // Create response message
        self.respond_to_message(message, serde_json::to_string(&response).unwrap())?;
        
        Ok(())
    }
    
    /// Create and queue a response to a message
    fn respond_to_message(&mut self, original_message: &FederationMessage, content: String) 
        -> Result<FederationMessage, CrossFederationError> 
    {
        let response = self.create_message(
            original_message.source_federation_id.clone(),
            match original_message.message_type {
                MessageType::ResourceRequest => MessageType::ResourceOffer,
                MessageType::CollaborationProposal => MessageType::CollaborationResponse,
                MessageType::JointValidation => MessageType::JointValidation,
                _ => original_message.message_type.clone(),
            },
            content,
            Some(original_message.id.clone()),
            Some(3600), // Default 1 hour expiry
        )?;
        
        self.queue_message(response.clone());
        
        Ok(response)
    }
    
    /// Send all queued outgoing messages
    pub async fn send_pending_messages(&mut self) -> Vec<Result<String, CrossFederationError>> {
        let mut results = Vec::new();
        
        while let Some(message) = self.outgoing_messages.pop_front() {
            // In a real implementation, this would use a network transport layer
            // For now we just simulate successful sending
            results.push(Ok(message.id.clone()));
        }
        
        results
    }
    
    /// Get conversation history by ID
    pub fn get_conversation(&self, conversation_id: &str) -> Option<&Vec<FederationMessage>> {
        self.message_history.get(conversation_id)
    }
    
    /// Update trust level for a federation
    pub fn update_trust_level(&mut self, federation_id: &str, new_level: u8) -> Result<(), CrossFederationError> {
        if !self.federation_keys.contains_key(federation_id) {
            return Err(CrossFederationError::FederationNotFound(federation_id.to_string()));
        }
        
        self.trust_levels.insert(federation_id.to_string(), new_level);
        Ok(())
    }
}

/// Resource request message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub request_id: String,
    pub resource_type: String,
    pub quantity: u64,
    pub duration: u64,
    pub purpose: String,
    pub compensation_offer: Option<String>,
}

/// Resource response message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub resource_id: Option<String>,
    pub access_conditions: Option<String>,
}

/// Collaboration proposal message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationProposal {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub goals: Vec<String>,
    pub requested_duration: u64,
    pub terms: String,
}

/// Collaboration response message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationResponse {
    pub proposal_id: String,
    pub approved: bool,
    pub token: Option<String>,
    pub start_time: u64,
    pub duration: u64,
    pub terms: String,
}

/// Joint validation message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointValidation {
    pub validation_id: String,
    pub data: Vec<u8>,
    pub expected_hash: String,
    pub validation_type: String,
}

/// Validation response message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub validation_id: String,
    pub confirmed: bool,
    pub validator_signature: Option<String>,
    pub timestamp: u64,
}