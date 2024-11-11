// src/blockchain/transaction.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    // Resource transfer between members
    Transfer {
        receiver: String,
        amount: u64,
    },
    
    // Smart contract execution
    ContractExecution {
        contract_id: String,
        input_data: HashMap<String, i64>,
    },
    
    // Relationship management
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    
    RecordMutualAid {
        receiver: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
    
    UpdateRelationship {
        member_two: String,
        relationship_type: String,
        story: String,
        interaction: Option<String>,
    },
    
    AddEndorsement {
        to_did: String,
        content: String,
        context: String,
        skills: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub transaction_type: TransactionType,
    pub timestamp: u128,
    pub hash: String,
    pub resource_cost: u64,      // Resource points required for this transaction
    pub resource_priority: u8,    // Priority level for resource allocation (1-10)
}

#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub max_resources: u64,      // Maximum resource points available
    pub current_resources: u64,  // Current resource points
    pub recovery_rate: u64,      // Points recovered per hour
    pub last_update: u128,       // Last resource update timestamp
}

impl Transaction {
    pub fn new(sender: String, transaction_type: TransactionType) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let hash = Self::calculate_transaction_hash(&sender, &transaction_type, timestamp);
        let resource_cost = Self::calculate_resource_cost(&transaction_type);
        
        Transaction {
            sender,
            transaction_type,
            timestamp,
            hash,
            resource_cost,
            resource_priority: 5, // Default priority level
        }
    }

    fn calculate_transaction_hash(sender: &str, transaction_type: &TransactionType, timestamp: u128) -> String {
        let mut hasher = Sha256::new();
        let transaction_data = match transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                format!("Transfer:{}:{}:{}", sender, receiver, amount)
            },
            TransactionType::ContractExecution { contract_id, input_data } => {
                format!("ContractExecution:{}:{:?}", contract_id, input_data)
            },
            TransactionType::RecordContribution { description, impact_story, context, tags } => {
                format!("Contribution:{}:{}:{}:{:?}", description, impact_story, context, tags)
            },
            TransactionType::RecordMutualAid { receiver, description, impact_story, reciprocity_notes, tags } => {
                format!("MutualAid:{}:{}:{:?}:{:?}:{:?}", receiver, description, impact_story, reciprocity_notes, tags)
            },
            TransactionType::UpdateRelationship { member_two, relationship_type, story, interaction } => {
                format!("Relationship:{}:{}:{}:{:?}", member_two, relationship_type, story, interaction)
            },
            TransactionType::AddEndorsement { to_did, content, context, skills } => {
                format!("Endorsement:{}:{}:{}:{:?}", to_did, content, context, skills)
            },
        };
        
        hasher.update(format!("{}{}{}", sender, transaction_data, timestamp));
        format!("{:x}", hasher.finalize())
    }

    fn calculate_resource_cost(transaction_type: &TransactionType) -> u64 {
        match transaction_type {
            TransactionType::Transfer { amount, .. } => {
                // Base cost plus percentage of transfer amount
                100 + (amount / 100)
            },
            TransactionType::ContractExecution { input_data, .. } => {
                // Base cost plus data size cost
                200 + (input_data.len() as u64 * 10)
            },
            TransactionType::RecordContribution { description, impact_story, tags, .. } => {
                // Cost based on content size and complexity
                let content_length = (description.len() + impact_story.len()) as u64;
                50 + (content_length / 100) + (tags.len() as u64 * 5)
            },
            TransactionType::RecordMutualAid { description, tags, .. } => {
                // Base cost plus content size
                75 + (description.len() as u64 / 100) + (tags.len() as u64 * 5)
            },
            TransactionType::UpdateRelationship { story, .. } => {
                // Base cost plus story length
                100 + (story.len() as u64 / 100)
            },
            TransactionType::AddEndorsement { content, skills, .. } => {
                // Base cost plus content and skills
                60 + (content.len() as u64 / 100) + (skills.len() as u64 * 10)
            },
        }
    }

    pub fn validate(&self) -> bool {
        // Ensure sender is not empty
        if self.sender.is_empty() {
            return false;
        }

        // Validate based on transaction type
        match &self.transaction_type {
            TransactionType::Transfer { receiver, amount } => {
                !receiver.is_empty() && *amount > 0
            },
            TransactionType::ContractExecution { contract_id, input_data } => {
                !contract_id.is_empty() && !input_data.is_empty()
            },
            TransactionType::RecordContribution { description, impact_story, context, tags } => {
                !description.is_empty() && 
                !impact_story.is_empty() && 
                !context.is_empty() && 
                !tags.is_empty()
            },
            TransactionType::RecordMutualAid { receiver, description, tags, .. } => {
                !receiver.is_empty() && 
                !description.is_empty() && 
                !tags.is_empty()
            },
            TransactionType::UpdateRelationship { member_two, relationship_type, story, .. } => {
                !member_two.is_empty() && 
                !relationship_type.is_empty() && 
                !story.is_empty()
            },
            TransactionType::AddEndorsement { to_did, content, context, skills } => {
                !to_did.is_empty() && 
                !content.is_empty() && 
                !context.is_empty() && 
                !skills.is_empty()
            },
        }
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.resource_priority = priority.min(10);
    }

    pub fn get_timestamp_ms(&self) -> u128 {
        self.timestamp
    }

    pub fn get_sender(&self) -> &str {
        &self.sender
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.sender.as_bytes());
        bytes.extend(self.hash.as_bytes());
        bytes.extend(&self.timestamp.to_be_bytes());
        bytes
    }
}

impl ResourceAllocation {
    pub fn new(reputation: i64) -> Self {
        // Calculate resource limits based on reputation
        let max_resources = 1000 + (reputation.max(0) as u64 * 100);
        let recovery_rate = 10 + (reputation.max(0) as u64 / 10);
        
        ResourceAllocation {
            max_resources,
            current_resources: max_resources,
            recovery_rate,
            last_update: Utc::now().timestamp_millis() as u128,
        }
    }

    pub fn update_resources(&mut self) {
        let now = Utc::now().timestamp_millis() as u128;
        let hours_elapsed = ((now - self.last_update) / (1000 * 60 * 60)) as u64;
        
        if hours_elapsed > 0 {
            let recovery = self.recovery_rate * hours_elapsed;
            self.current_resources = (self.current_resources + recovery).min(self.max_resources);
            self.last_update = now;
        }
    }

    pub fn can_afford(&self, cost: u64) -> bool {
        self.current_resources >= cost
    }

    pub fn consume_resources(&mut self, cost: u64) -> Result<(), String> {
        if !self.can_afford(cost) {
            return Err("Insufficient resources".to_string());
        }
        
        self.current_resources -= cost;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_transaction_creation() {
        let sender = "did:icn:sender".to_string();
        let transaction = Transaction::new(
            sender.clone(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        );

        assert_eq!(transaction.sender, sender);
        assert!(!transaction.hash.is_empty());
        assert!(transaction.validate());
    }

    #[test]
    fn test_contribution_transaction() {
        let transaction = Transaction::new(
            "did:icn:sender".to_string(),
            TransactionType::RecordContribution {
                description: "Test contribution".to_string(),
                impact_story: "Made a difference".to_string(),
                context: "Testing".to_string(),
                tags: vec!["test".to_string()],
            },
        );

        assert!(transaction.validate());
        assert!(transaction.resource_cost > 0);
    }

    #[test]
    fn test_invalid_transaction() {
        let transaction = Transaction::new(
            "".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 0,
            },
        );

        assert!(!transaction.validate());
    }

    #[test]
    fn test_resource_allocation() {
        let mut resources = ResourceAllocation::new(1000);
        assert!(resources.can_afford(500));
        assert!(resources.consume_resources(500).is_ok());
        assert_eq!(resources.current_resources, resources.max_resources - 500);
    }

    #[test]
    fn test_resource_recovery() {
        let mut resources = ResourceAllocation::new(1000);
        resources.consume_resources(500).unwrap();
        resources.last_update -= 3600 * 1000; // Subtract one hour in milliseconds
        resources.update_resources();
        assert!(resources.current_resources > resources.max_resources - 500);
    }

    #[test]
    fn test_priority_setting() {
        let mut transaction = Transaction::new(
            "did:icn:sender".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        );

        transaction.set_priority(15); // Should be capped at 10
        assert_eq!(transaction.resource_priority, 10);
    }
}