use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidDocument {
    pub id: String,
    pub public_key: String,
    pub active: bool,
}

pub struct DidRegistry {
    ledger: HashMap<String, DidDocument>,
}

impl DidRegistry {
    pub fn new() -> Self {
        DidRegistry {
            ledger: HashMap::new(),
        }
    }
    
    // Create (register) a new DID document
    pub fn register_did(&mut self, did: &str, pub_key: &str) {
        let doc = DidDocument {
            id: did.to_string(),
            public_key: pub_key.to_string(),
            active: true,
        };
        self.ledger.insert(did.to_string(), doc);
    }
    
    // Read (get) a DID document
    pub fn get_did(&self, did: &str) -> Option<&DidDocument> {
        self.ledger.get(did)
    }
    
    // Update a DID document's public key
    pub fn update_did(&mut self, did: &str, new_pub_key: &str) {
        if let Some(doc) = self.ledger.get_mut(did) {
            doc.public_key = new_pub_key.to_string();
        }
    }
    
    // Deactivate (mark inactive) a DID document
    pub fn deactivate_did(&mut self, did: &str) {
        if let Some(doc) = self.ledger.get_mut(did) {
            doc.active = false;
        }
    }
    
    // Implement proof verification logic using cryptographic libraries.
    // For simplicity, this placeholder verifies that the document exists and is active.
    pub fn verify_proofs(&self, did: &str) -> bool {
        if let Some(doc) = self.get_did(did) {
            // ...insert cryptographic verification logic here...
            doc.active
        } else {
            false
        }
    }
}