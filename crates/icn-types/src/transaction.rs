use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    ContributionRecord,
    RelationshipUpdate,
    ProposalSubmission,
    VoteCast,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub sender: String,
    pub receiver: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        sender: String,
        receiver: Option<String>,
    ) -> Self {
        let timestamp = chrono::Utc::now();
        let mut tx = Self {
            id: String::new(),
            transaction_type,
            sender,
            receiver,
            timestamp,
            metadata: HashMap::new(),
        };
        
        tx.id = tx.calculate_id();
        tx
    }

    pub fn calculate_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}",
            self.sender,
            self.receiver.as_deref().unwrap_or(""),
            self.timestamp.timestamp(),
            serde_json::to_string(&self.transaction_type).unwrap(),
        ));
        hex::encode(hasher.finalize())
    }
}
