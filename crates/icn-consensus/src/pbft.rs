use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    PrePrepare,
    Prepare,
    Commit,
    ViewChange,
    NewView,
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
    validators: Vec<String>,
    primary: usize,
    prepared_messages: HashMap<String, HashSet<String>>, // block_hash -> set of validator IDs
    committed_messages: HashMap<String, HashSet<String>>,
    view_change_messages: HashMap<u64, HashSet<String>>, // view_number -> set of validator IDs
    timeout: Duration,
    last_activity: Instant,
}

impl PbftConsensus {
    pub fn new(validators: Vec<String>) -> Self {
        Self {
            view_number: 0,
            sequence_number: 0,
            validators: validators.clone(),
            primary: 0,
            prepared_messages: HashMap::new(),
            committed_messages: HashMap::new(),
            view_change_messages: HashMap::new(),
            timeout: Duration::from_secs(30),
            last_activity: Instant::now(),
        }
    }

    pub fn is_primary(&self, validator_id: &str) -> bool {
        self.validators.get(self.primary) == Some(&validator_id.to_string())
    }

    pub fn handle_message(&mut self, message: ConsensusMessage) -> Result<(), String> {
        self.last_activity = Instant::now();

        match message.message_type {
            MessageType::PrePrepare => self.handle_pre_prepare(message),
            MessageType::Prepare => self.handle_prepare(message),
            MessageType::Commit => self.handle_commit(message),
            MessageType::ViewChange => self.handle_view_change(message),
            MessageType::NewView => self.handle_new_view(message),
        }
    }

    fn handle_pre_prepare(&mut self, message: ConsensusMessage) -> Result<(), String> {
        // Verify the message is from the current primary
        if !self.is_primary(&message.sender) {
            return Err("Pre-prepare from non-primary node".to_string());
        }

        // Verify sequence number
        if message.sequence_number != self.sequence_number + 1 {
            return Err("Invalid sequence number".to_string());
        }

        // Initialize prepared set for this block
        self.prepared_messages.entry(message.block_hash.clone())
            .or_insert_with(HashSet::new);

        self.sequence_number = message.sequence_number;
        Ok(())
    }

    fn handle_prepare(&mut self, message: ConsensusMessage) -> Result<(), String> {
        // Add prepare message to prepared set
        if let Some(prepared_set) = self.prepared_messages.get_mut(&message.block_hash) {
            prepared_set.insert(message.sender);

            // Check if we have enough prepares (2f + 1)
            if self.has_quorum(prepared_set.len()) {
                debug!("Block {} has reached prepare quorum", message.block_hash);
            }
        }

        Ok(())
    }

    fn handle_commit(&mut self, message: ConsensusMessage) -> Result<(), String> {
        // Verify we have enough prepares before accepting commits
        if let Some(prepared_set) = self.prepared_messages.get(&message.block_hash) {
            if !self.has_quorum(prepared_set.len()) {
                return Err("Cannot commit before prepare quorum".to_string());
            }
        } else {
            return Err("No prepare phase for this block".to_string());
        }

        // Add commit message
        let committed_set = self.committed_messages.entry(message.block_hash.clone())
            .or_insert_with(HashSet::new);
        committed_set.insert(message.sender);

        Ok(())
    }

    fn handle_view_change(&mut self, message: ConsensusMessage) -> Result<(), String> {
        // Add view change message
        let view_changes = self.view_change_messages.entry(message.view_number)
            .or_insert_with(HashSet::new);
        view_changes.insert(message.sender);

        // Check if we have enough view changes to proceed
        if self.has_quorum(view_changes.len()) {
            self.start_new_view(message.view_number)?;
        }

        Ok(())
    }

    fn handle_new_view(&mut self, message: ConsensusMessage) -> Result<(), String> {
        // Verify message is from the new primary
        let new_primary = (message.view_number as usize) % self.validators.len();
        if self.validators[new_primary] != message.sender {
            return Err("New view message from invalid primary".to_string());
        }

        self.view_number = message.view_number;
        self.primary = new_primary;
        self.sequence_number = message.sequence_number;

        Ok(())
    }

    pub fn is_committed(&self, block_hash: &str) -> bool {
        if let Some(committed_set) = self.committed_messages.get(block_hash) {
            self.has_quorum(committed_set.len())
        } else {
            false
        }
    }

    fn has_quorum(&self, count: usize) -> bool {
        // Need 2f + 1 messages where f = (n-1)/3
        let f = (self.validators.len() - 1) / 3;
        count >= 2 * f + 1
    }

    fn start_new_view(&mut self, new_view: u64) -> Result<(), String> {
        if new_view <= self.view_number {
            return Err("Invalid new view number".to_string());
        }

        self.view_number = new_view;
        self.primary = (new_view as usize) % self.validators.len();
        self.prepared_messages.clear();
        self.committed_messages.clear();

        Ok(())
    }

    pub fn check_timeout(&mut self) -> bool {
        if self.last_activity.elapsed() > self.timeout {
            // Initiate view change
            self.start_new_view(self.view_number + 1).unwrap_or_else(|e| {
                error!("Failed to start new view: {}", e);
            });
            true
        } else {
            false
        }
    }
}
