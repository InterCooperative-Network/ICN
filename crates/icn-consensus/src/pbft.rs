use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    PrePrepare,
    Prepare,
    Commit,
    ViewChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    pub message_type: MessageType,
    pub view_number: u64,
    pub sequence_number: u64,
    pub block_hash: String,
    pub sender: String,
    pub signature: String,
}

pub struct PbftConsensus {
    pub view_number: u64,
    pub sequence_number: u64,
    pub validators: Vec<String>,
    pub primary: usize, // Index of primary validator in validators list
    pub messages: HashMap<(MessageType, u64, u64), HashMap<String, ConsensusMessage>>, // (type, view, seq) -> (validator -> message)
    pub prepared_blocks: HashSet<String>, // Block hashes that have been prepared
    pub committed_blocks: HashSet<String>, // Block hashes that have been committed
}

impl PbftConsensus {
    pub fn new(validators: Vec<String>) -> Self {
        let primary = 0; // Start with first validator as primary
        Self {
            view_number: 0,
            sequence_number: 0,
            validators,
            primary,
            messages: HashMap::new(),
            prepared_blocks: HashSet::new(),
            committed_blocks: HashSet::new(),
        }
    }
    
    pub fn get_primary(&self) -> &String {
        &self.validators[self.primary]
    }
    
    pub fn is_primary(&self, validator: &str) -> bool {
        self.validators.get(self.primary).map_or(false, |primary| primary == validator)
    }
    
    pub fn process_message(&mut self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>, String> {
        match message.message_type {
            MessageType::PrePrepare => self.process_pre_prepare(message),
            MessageType::Prepare => self.process_prepare(message),
            MessageType::Commit => self.process_commit(message),
            MessageType::ViewChange => self.process_view_change(message),
        }
    }
    
    pub fn process_pre_prepare(&mut self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>, String> {
        // Ensure message is from the primary
        if !self.is_primary(&message.sender) {
            return Err("Pre-prepare message not from primary".into());
        }
        
        // Ensure view number matches current view
        if message.view_number != self.view_number {
            return Err("Pre-prepare message for wrong view".into());
        }
        
        // Store the message
        self.messages
            .entry((MessageType::PrePrepare, message.view_number, message.sequence_number))
            .or_insert_with(HashMap::new)
            .insert(message.sender.clone(), message.clone());
            
        // Respond with Prepare message
        let prepare_message = ConsensusMessage {
            message_type: MessageType::Prepare,
            view_number: self.view_number,
            sequence_number: message.sequence_number,
            block_hash: message.block_hash.clone(),
            sender: "self_id".to_string(), // This should be the node's ID
            signature: "signature".to_string(), // This should be a proper signature
        };
        
        Ok(vec![prepare_message])
    }
    
    pub fn process_prepare(&mut self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>, String> {
        // Ensure view number matches current view
        if message.view_number != self.view_number {
            return Err("Prepare message for wrong view".into());
        }
        
        // Store the message
        self.messages
            .entry((MessageType::Prepare, message.view_number, message.sequence_number))
            .or_insert_with(HashMap::new)
            .insert(message.sender.clone(), message.clone());
            
        // Check if we have 2f+1 prepare messages (including our own)
        let prepare_count = self.messages
            .get(&(MessageType::Prepare, message.view_number, message.sequence_number))
            .map(|m| m.len())
            .unwrap_or(0);
            
        // Calculate required quorum (2f+1 out of 3f+1)
        let f = (self.validators.len() - 1) / 3;
        let quorum = 2 * f + 1;
        
        if prepare_count >= quorum {
            // Mark as prepared
            self.prepared_blocks.insert(message.block_hash.clone());
            
            // Send commit message
            let commit_message = ConsensusMessage {
                message_type: MessageType::Commit,
                view_number: self.view_number,
                sequence_number: message.sequence_number,
                block_hash: message.block_hash.clone(),
                sender: "self_id".to_string(), // This should be the node's ID
                signature: "signature".to_string(), // This should be a proper signature
            };
            
            Ok(vec![commit_message])
        } else {
            Ok(vec![])
        }
    }
    
    pub fn process_commit(&mut self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>, String> {
        // Ensure view number matches current view
        if message.view_number != self.view_number {
            return Err("Commit message for wrong view".into());
        }
        
        // Store the message
        self.messages
            .entry((MessageType::Commit, message.view_number, message.sequence_number))
            .or_insert_with(HashMap::new)
            .insert(message.sender.clone(), message.clone());
            
        // Check if we have 2f+1 commit messages
        let commit_count = self.messages
            .get(&(MessageType::Commit, message.view_number, message.sequence_number))
            .map(|m| m.len())
            .unwrap_or(0);
            
        // Calculate required quorum (2f+1 out of 3f+1)
        let f = (self.validators.len() - 1) / 3;
        let quorum = 2 * f + 1;
        
        if commit_count >= quorum {
            // Mark as committed
            self.committed_blocks.insert(message.block_hash.clone());
            
            // Move to next sequence number
            self.sequence_number += 1;
            
            // No new messages to send
            Ok(vec![])
        } else {
            Ok(vec![])
        }
    }
    
    pub fn process_view_change(&mut self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>, String> {
        // Implementation for view change omitted for brevity
        Ok(vec![])
    }
    
    pub fn is_committed(&self, block_hash: &str) -> bool {
        self.committed_blocks.contains(block_hash)
    }
}
