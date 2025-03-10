use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use icn_crypto::KeyPair;
use icn_types::FederationId;
use sodiumoxide::crypto::box_;
use hex;

/// Error types for federation messaging
#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Message not found: {0}")]
    MessageNotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

/// Message visibility settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageVisibility {
    /// Visible only to direct recipient
    Private,
    
    /// Visible to specific roles in the federation
    Role(Vec<String>),
    
    /// Visible to all federation members
    Federation,
    
    /// Visible to member and linked federations
    FederationNetwork,
    
    /// Visible to all ICN participants (broadcast)
    Public,
}

/// Message priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Lowest priority, non-urgent
    Low,
    
    /// Normal priority for most communication
    Normal,
    
    /// Higher priority for important messages
    High,
    
    /// Highest priority for critical messages
    Critical,
    
    /// System messages (highest priority)
    System,
}

/// Message status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message has been composed but not sent
    Draft,
    
    /// Message has been sent but not delivered
    Sent,
    
    /// Message has been delivered to recipient
    Delivered,
    
    /// Message has been read by recipient
    Read,
    
    /// Message has been acknowledged by recipient
    Acknowledged,
    
    /// Message failed to deliver
    Failed,
    
    /// Message expired before delivery
    Expired,
}

/// Types of federation messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Standard text message
    Text,
    
    /// Governance proposal
    Proposal,
    
    /// Vote on a proposal
    Vote,
    
    /// Resource request
    ResourceRequest,
    
    /// Resource offer
    ResourceOffer,
    
    /// System notification
    SystemNotification,
    
    /// Custom message type
    Custom(String),
}

/// A secure message between federation members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Unique message ID
    pub id: String,
    
    /// Sender's DID
    pub sender: String,
    
    /// Recipient's DID or federation ID
    pub recipient: String,
    
    /// Message visibility
    pub visibility: MessageVisibility,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// Message subject
    pub subject: String,
    
    /// Encrypted message content
    pub encrypted_content: Vec<u8>,
    
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Current message status
    pub status: MessageStatus,
    
    /// Digital signature of the message
    pub signature: String,
    
    /// References to other messages (e.g., for threading)
    pub references: Vec<String>,
    
    /// Custom message attributes
    pub attributes: HashMap<String, String>,
}

/// Manages the messaging system for federation members
pub struct FederationMessenger {
    /// The federation ID this messenger belongs to
    federation_id: FederationId,
    
    /// Key pair for encryption/decryption
    key_pair: KeyPair,
    
    /// Public keys of known participants
    public_keys: RwLock<HashMap<String, Vec<u8>>>,
    
    /// Inbox of received messages
    inbox: RwLock<Vec<FederationMessage>>,
    
    /// Outbox of sent messages
    outbox: RwLock<Vec<FederationMessage>>,
    
    /// Draft messages
    drafts: RwLock<HashMap<String, FederationMessage>>,
    
    /// Message delivery status
    delivery_status: RwLock<HashMap<String, MessageStatus>>,
    
    /// Message channels by topic
    channels: RwLock<HashMap<String, HashSet<String>>>,
    
    /// Message handlers by type
    message_handlers: RwLock<HashMap<MessageType, Box<dyn MessageHandler + Send + Sync>>>,
    
    /// Members of this federation
    federation_members: RwLock<HashSet<String>>,
}

/// Trait for handling different message types
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: &FederationMessage) -> Result<(), MessagingError>;
}

impl FederationMessenger {
    /// Create a new FederationMessenger
    pub fn new(federation_id: FederationId, key_pair: KeyPair) -> Self {
        Self {
            federation_id,
            key_pair,
            public_keys: RwLock::new(HashMap::new()),
            inbox: RwLock::new(Vec::new()),
            outbox: RwLock::new(Vec::new()),
            drafts: RwLock::new(HashMap::new()),
            delivery_status: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
            message_handlers: RwLock::new(HashMap::new()),
            federation_members: RwLock::new(HashSet::new()),
        }
    }

    /// Register public key for a participant
    pub async fn register_public_key(&self, did: &str, public_key: Vec<u8>) {
        let mut keys = self.public_keys.write().await;
        keys.insert(did.to_string(), public_key);
    }

    /// Register a member of the federation
    pub async fn register_member(&self, member_did: &str) {
        let mut members = self.federation_members.write().await;
        members.insert(member_did.to_string());
    }

    /// Register a message handler for a specific message type
    pub async fn register_handler(&self, message_type: MessageType, handler: Box<dyn MessageHandler + Send + Sync>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.insert(message_type, handler);
    }

    /// Create a new message
    pub async fn create_message(
        &self,
        recipient: &str,
        message_type: MessageType,
        subject: &str,
        content: &[u8],
        visibility: MessageVisibility,
        priority: MessagePriority,
        references: Vec<String>,
        expires_in_hours: Option<u64>,
    ) -> Result<FederationMessage, MessagingError> {
        // Encrypt content for recipient
        let encrypted_content = self.encrypt_for_recipient(recipient, content).await?;
        
        // Generate message ID
        let message_id = Uuid::new_v4().to_string();
        
        // Set expiration time if provided
        let expires_at = expires_in_hours.map(|hours| {
            Utc::now() + chrono::Duration::hours(hours as i64)
        });
        
        // Create signature for message
        let signature_data = format!(
            "{}:{}:{}:{}:{}",
            message_id, self.federation_id, recipient, Utc::now(), hex::encode(&encrypted_content)
        );
        
        let signature = match self.key_pair.sign(signature_data.as_bytes()) {
            Ok(sig) => hex::encode(sig),
            Err(_) => return Err(MessagingError::EncryptionFailed("Failed to sign message".to_string())),
        };
        
        // Create the message
        let message = FederationMessage {
            id: message_id,
            sender: self.federation_id.clone(),
            recipient: recipient.to_string(),
            visibility,
            message_type,
            priority,
            subject: subject.to_string(),
            encrypted_content,
            timestamp: Utc::now(),
            expires_at,
            status: MessageStatus::Draft,
            signature,
            references,
            attributes: HashMap::new(),
        };
        
        // Store in drafts
        let mut drafts = self.drafts.write().await;
        drafts.insert(message.id.clone(), message.clone());
        
        Ok(message)
    }

    /// Encrypt content for a specific recipient
    async fn encrypt_for_recipient(&self, recipient: &str, content: &[u8]) -> Result<Vec<u8>, MessagingError> {
        // Get recipient's public key
        let keys = self.public_keys.read().await;
        let public_key = keys.get(recipient)
            .ok_or_else(|| MessagingError::InvalidRecipient(format!("No public key for {}", recipient)))?;
        
        // Encrypt the content
        encrypt(content, public_key).map_err(|e| MessagingError::EncryptionFailed(e.to_string()))
    }

    /// Decrypt content of a message
    async fn decrypt_message(&self, message: &FederationMessage) -> Result<Vec<u8>, MessagingError> {
        // Verify the message is for us
        if message.recipient != self.federation_id {
            return Err(MessagingError::Unauthorized("Message not intended for this recipient".to_string()));
        }
        
        // Decrypt using our private key
        decrypt(&message.encrypted_content, &self.key_pair.private_key)
            .map_err(|e| MessagingError::DecryptionFailed(e.to_string()))
    }

    /// Send a message that was previously created
    pub async fn send_message(&self, message_id: &str) -> Result<(), MessagingError> {
        // Get the message from drafts
        let mut drafts = self.drafts.write().await;
        let message = drafts.remove(message_id)
            .ok_or_else(|| MessagingError::MessageNotFound(message_id.to_string()))?;
        
        // Update status
        let mut message = message;
        message.status = MessageStatus::Sent;
        
        // Store in outbox
        let mut outbox = self.outbox.write().await;
        outbox.push(message.clone());
        
        // Update delivery status
        let mut status = self.delivery_status.write().await;
        status.insert(message.id.clone(), MessageStatus::Sent);
        
        // In a real implementation, we would now send the message over the network
        
        Ok(())
    }

    /// Create and send a message in one step
    pub async fn send_new_message(
        &self,
        recipient: &str,
        message_type: MessageType,
        subject: &str,
        content: &[u8],
        visibility: MessageVisibility,
        priority: MessagePriority,
        references: Vec<String>,
        expires_in_hours: Option<u64>,
    ) -> Result<String, MessagingError> {
        let message = self.create_message(
            recipient,
            message_type.clone(),
            subject,
            content,
            visibility,
            priority.clone(),
            references,
            expires_in_hours,
        ).await?;
        
        let message_id = message.id.clone();
        self.send_message(&message_id).await?;
        
        Ok(message_id)
    }

    /// Process a received message
    pub async fn process_received_message(&self, message: FederationMessage) -> Result<(), MessagingError> {
        // Verify this message is intended for us
        if message.recipient != self.federation_id && !self.is_broadcast(&message).await {
            return Err(MessagingError::Unauthorized("Message not intended for this recipient".to_string()));
        }
        
        // Verify the signature
        self.verify_message(&message).await?;
        
        // Update status
        let mut updated_message = message.clone();
        updated_message.status = MessageStatus::Delivered;
        
        // Store in inbox
        let mut inbox = self.inbox.write().await;
        inbox.push(updated_message.clone());
        
        // Update delivery status
        let mut status = self.delivery_status.write().await;
        status.insert(updated_message.id.clone(), MessageStatus::Delivered);
        
        // Check if we have a handler for this message type
        let handlers = self.message_handlers.read().await;
        if let Some(handler) = handlers.get(&message.message_type) {
            handler.handle_message(&updated_message).await?;
        }
        
        Ok(())
    }

    /// Mark a message as read
    pub async fn mark_as_read(&self, message_id: &str) -> Result<(), MessagingError> {
        let mut inbox = self.inbox.write().await;
        let msg_index = inbox.iter().position(|m| m.id == message_id)
            .ok_or_else(|| MessagingError::MessageNotFound(message_id.to_string()))?;
        
        inbox[msg_index].status = MessageStatus::Read;
        
        // Update delivery status
        let mut status = self.delivery_status.write().await;
        status.insert(message_id.to_string(), MessageStatus::Read);
        
        Ok(())
    }

    /// Mark a message as acknowledged
    pub async fn acknowledge_message(&self, message_id: &str) -> Result<(), MessagingError> {
        let mut inbox = self.inbox.write().await;
        let msg_index = inbox.iter().position(|m| m.id == message_id)
            .ok_or_else(|| MessagingError::MessageNotFound(message_id.to_string()))?;
        
        inbox[msg_index].status = MessageStatus::Acknowledged;
        
        // Update delivery status
        let mut status = self.delivery_status.write().await;
        status.insert(message_id.to_string(), MessageStatus::Acknowledged);
        
        Ok(())
    }

    /// Check if a message is a broadcast message
    async fn is_broadcast(&self, message: &FederationMessage) -> bool {
        matches!(message.visibility, MessageVisibility::Public | MessageVisibility::FederationNetwork)
    }

    /// Verify a message signature
    async fn verify_message(&self, message: &FederationMessage) -> Result<bool, MessagingError> {
        // Get sender's public key
        let keys = self.public_keys.read().await;
        let sender_public_key = keys.get(&message.sender)
            .ok_or_else(|| MessagingError::InvalidRecipient(format!("No public key for {}", message.sender)))?;
        
        // Recreate signature data
        let signature_data = format!(
            "{}:{}:{}:{}:{}",
            message.id, message.sender, message.recipient, message.timestamp, hex::encode(&message.encrypted_content)
        );
        
        // Verify the signature
        // In a real implementation, we'd use proper signature verification
        // For now, just check if it's not empty
        if message.signature.is_empty() {
            return Err(MessagingError::Unauthorized("Invalid signature".to_string()));
        }
        
        Ok(true)
    }

    /// Subscribe to a message channel
    pub async fn subscribe_to_channel(&self, channel: &str) {
        let member_id = self.federation_id.clone();
        let mut channels = self.channels.write().await;
        
        let subscribers = channels.entry(channel.to_string()).or_insert_with(HashSet::new);
        subscribers.insert(member_id);
    }

    /// Unsubscribe from a message channel
    pub async fn unsubscribe_from_channel(&self, channel: &str) {
        let member_id = self.federation_id.clone();
        let mut channels = self.channels.write().await;
        
        if let Some(subscribers) = channels.get_mut(channel) {
            subscribers.remove(&member_id);
        }
    }

    /// Send a message to a channel
    pub async fn send_to_channel(
        &self,
        channel: &str,
        subject: &str,
        content: &[u8],
        message_type: MessageType,
        priority: MessagePriority,
        expires_in_hours: Option<u64>,
    ) -> Result<Vec<String>, MessagingError> {
        let mut message_ids = Vec::new();
        
        // Get all subscribers
        let channels = self.channels.read().await;
        let subscribers = channels.get(channel)
            .ok_or_else(|| MessagingError::InvalidRecipient(format!("Channel not found: {}", channel)))?;
        
        // Send to each subscriber
        for subscriber in subscribers {
            if subscriber != &self.federation_id {
                let msg_id = self.send_new_message(
                    subscriber,
                    message_type.clone(),
                    subject,
                    content,
                    MessageVisibility::Private,
                    priority.clone(),
                    vec![],
                    expires_in_hours,
                ).await?;
                
                message_ids.push(msg_id);
            }
        }
        
        Ok(message_ids)
    }

    /// Get all messages in the inbox
    pub async fn get_inbox_messages(&self) -> Vec<FederationMessage> {
        let inbox = self.inbox.read().await;
        inbox.clone()
    }

    /// Get all sent messages
    pub async fn get_sent_messages(&self) -> Vec<FederationMessage> {
        let outbox = self.outbox.read().await;
        outbox.clone()
    }

    /// Get message by ID
    pub async fn get_message(&self, message_id: &str) -> Option<FederationMessage> {
        // Check inbox
        let inbox = self.inbox.read().await;
        if let Some(msg) = inbox.iter().find(|m| m.id == message_id) {
            return Some(msg.clone());
        }
        
        // Check outbox
        let outbox = self.outbox.read().await;
        if let Some(msg) = outbox.iter().find(|m| m.id == message_id) {
            return Some(msg.clone());
        }
        
        // Check drafts
        let drafts = self.drafts.read().await;
        drafts.get(message_id).cloned()
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: &str) -> Result<(), MessagingError> {
        // Check and remove from inbox
        {
            let mut inbox = self.inbox.write().await;
            if let Some(pos) = inbox.iter().position(|m| m.id == message_id) {
                inbox.remove(pos);
                return Ok(());
            }
        }
        
        // Check and remove from outbox
        {
            let mut outbox = self.outbox.write().await;
            if let Some(pos) = outbox.iter().position(|m| m.id == message_id) {
                outbox.remove(pos);
                return Ok(());
            }
        }
        
        // Check and remove from drafts
        {
            let mut drafts = self.drafts.write().await;
            if drafts.remove(message_id).is_some() {
                return Ok(());
            }
        }
        
        Err(MessagingError::MessageNotFound(message_id.to_string()))
    }

    /// Clean up expired messages
    pub async fn cleanup_expired_messages(&self) -> usize {
        let now = Utc::now();
        let mut expired_count = 0;
        
        // Clean inbox
        {
            let mut inbox = self.inbox.write().await;
            let before_len = inbox.len();
            inbox.retain(|msg| {
                msg.expires_at.map(|exp| exp > now).unwrap_or(true)
            });
            expired_count += before_len - inbox.len();
        }
        
        // Clean outbox
        {
            let mut outbox = self.outbox.write().await;
            let before_len = outbox.len();
            outbox.retain(|msg| {
                msg.expires_at.map(|exp| exp > now).unwrap_or(true)
            });
            expired_count += before_len - outbox.len();
        }
        
        // Clean drafts
        {
            let mut drafts = self.drafts.write().await;
            let before_len = drafts.len();
            drafts.retain(|_, msg| {
                msg.expires_at.map(|exp| exp > now).unwrap_or(true)
            });
            expired_count += before_len - drafts.len();
        }
        
        expired_count
    }

    /// Start background message processing
    pub async fn start_background_processor(messenger: Arc<FederationMessenger>) {
        tokio::spawn(async move {
            loop {
                // Clean up expired messages
                let expired_count = messenger.cleanup_expired_messages().await;
                if expired_count > 0 {
                    println!("Cleaned up {} expired messages", expired_count);
                }
                
                // Sleep for a bit
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }
}

// Simple encryption/decryption functions
fn encrypt(data: &[u8], _key: &KeyPair) -> Result<Vec<u8>, MessagingError> {
    // This is a placeholder implementation
    // In a real implementation, we would use the key to encrypt the data
    Ok(data.to_vec())
}

fn decrypt(data: &[u8], _key: &KeyPair) -> Result<Vec<u8>, MessagingError> {
    // This is a placeholder implementation
    // In a real implementation, we would use the key to decrypt the data
    Ok(data.to_vec())
}
