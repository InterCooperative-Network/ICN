use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Types of PBFT consensus messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    PrePrepare,
    Prepare,
    Commit,
    ViewChange,
    NewView,
}

/// PBFT consensus message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    pub message_type: MessageType,
    pub view_number: u64,
    pub sequence_number: u64,
    pub block_hash: String,
    pub sender: String,
    pub signature: String, // Cryptographic signature of message content
}

/// PBFT consensus state
pub struct PbftConsensus {
    pub view_number: u64,
    pub sequence_number: u64,
    pub validators: Vec<String>,
    pub primary_idx: usize,
    pub prepared_msgs: HashMap<String, HashSet<String>>, // block_hash -> set of validators that prepared
    pub committed_msgs: HashMap<String, HashSet<String>>, // block_hash -> set of validators that committed
    pub committed_blocks: HashSet<String>,                // set of committed block hashes
}

impl PbftConsensus {
    pub fn new(validators: Vec<String>) -> Self {
        Self {
            view_number: 0,
            sequence_number: 0,
            validators,
            primary_idx: 0,
            prepared_msgs: HashMap::new(),
            committed_msgs: HashMap::new(),
            committed_blocks: HashSet::new(),
        }
    }

    /// Check if the current node is the primary/leader
    pub fn is_primary(&self, node_id: &str) -> bool {
        if self.validators.is_empty() {
            return false;
        }
        
        let primary_id = &self.validators[self.primary_idx % self.validators.len()];
        primary_id == node_id
    }

    /// Process a consensus message
    pub fn process_message(&mut self, message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // Verify the sender is a validator
        if !self.validators.contains(&message.sender) {
            return Err("Sender is not a validator".to_string());
        }

        // Verify message view number
        if message.view_number != self.view_number {
            return Err(format!(
                "Message view number mismatch: expected {}, got {}",
                self.view_number, message.view_number
            ));
        }

        match message.message_type {
            MessageType::PrePrepare => self.handle_pre_prepare(message),
            MessageType::Prepare => self.handle_prepare(message),
            MessageType::Commit => self.handle_commit(message),
            MessageType::ViewChange => self.handle_view_change(message),
            MessageType::NewView => self.handle_new_view(message),
        }
    }

    /// Handle pre-prepare message from the primary
    fn handle_pre_prepare(&mut self, message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // Verify the sender is the primary
        let expected_primary = &self.validators[self.primary_idx % self.validators.len()];
        if message.sender != *expected_primary {
            return Err("Pre-prepare message not from primary".to_string());
        }

        // Verify sequence number is correct
        if message.sequence_number != self.sequence_number {
            return Err("Incorrect sequence number".to_string());
        }

        // Create a prepare response
        Ok(Some(MessageType::Prepare))
    }

    /// Handle prepare message from validators
    fn handle_prepare(&mut self, message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // Track prepare message
        let prepared = self.prepared_msgs
            .entry(message.block_hash.clone())
            .or_insert_with(HashSet::new());

        prepared.insert(message.sender);

        // Check if we have enough prepare messages
        let prepared_count = prepared.len();
        let needed_count = self.get_quorum_size();

        if prepared_count >= needed_count && !self.is_committed(&message.block_hash) {
            // We can move to commit phase
            return Ok(Some(MessageType::Commit));
        }

        // Not enough prepare messages yet
        Ok(None)
    }

    /// Handle commit message from validators
    fn handle_commit(&mut self, message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // Track commit message
        let committed = self.committed_msgs
            .entry(message.block_hash.clone())
            .or_insert_with(HashSet::new);

        committed.insert(message.sender);

        // Check if we have enough commit messages
        let committed_count = committed.len();
        let needed_count = self.get_quorum_size();

        if committed_count >= needed_count && !self.committed_blocks.contains(&message.block_hash) {
            // Add to committed blocks
            self.committed_blocks.insert(message.block_hash.clone());

            // Move to next sequence
            self.sequence_number += 1;

            // No need for additional message types
            return Ok(None);
        }

        // Not enough commit messages yet
        Ok(None)
    }

    /// Handle view change message
    fn handle_view_change(&mut self, message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // View change logic (simplified)
        // In a real system, this would involve collecting view change messages
        // and transitioning to a new primary when enough messages are received
        self.view_number += 1;
        self.primary_idx = (self.primary_idx + 1) % self.validators.len();

        Ok(Some(MessageType::NewView))
    }

    /// Handle new view message
    fn handle_new_view(&mut self, _message: ConsensusMessage) -> Result<Option<MessageType>, String> {
        // New view confirmation logic
        // In a real system, this would involve more complex confirmation of the new view
        
        // Reset consensus state for new view
        self.prepared_msgs.clear();
        self.committed_msgs.clear();

        Ok(None)
    }

    /// Check if a block is already committed
    pub fn is_committed(&self, block_hash: &str) -> bool {
        self.committed_blocks.contains(block_hash)
    }

    /// Get the appropriate quorum size for BFT
    fn get_quorum_size(&self) -> usize {
        let f = (self.validators.len() - 1) / 3;  // max faulty nodes
        2 * f + 1 // BFT quorum = 2f+1 from total 3f+1
    }

    /// Force a view change (e.g., on timeout)
    pub fn force_view_change(&mut self) {
        self.view_number += 1;
        self.primary_idx = (self.primary_idx + 1) % self.validators.len();
        self.prepared_msgs.clear();
        self.committed_msgs.clear();
    }

    /// Reset sequence counter (for new consensus rounds)
    pub fn reset_sequence(&mut self) {
        self.sequence_number = 0;
        self.prepared_msgs.clear();
        self.committed_msgs.clear();
    }
    
    /// Get the number of validators
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
    
    /// Get max faulty nodes this system can tolerate
    pub fn max_faulty_nodes(&self) -> usize {
        (self.validators.len() - 1) / 3
    }
}
