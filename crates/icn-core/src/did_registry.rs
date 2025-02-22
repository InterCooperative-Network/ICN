use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidDocument {
    pub id: String,
    pub public_key: String,
    pub active: bool,
    pub is_verified: bool,
    pub verification_proof: Option<VerificationProof>,
    pub last_verification: Option<u64>,
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
            is_verified: false,
            verification_proof: None,
            last_verification: None,
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

    pub fn verify_did(&mut self, did: &str, proof: VerificationProof) -> Result<bool, String> {
        if let Some(doc) = self.ledger.get_mut(did) {
            let mut did_instance = DID::new(doc.id.clone(), Algorithm::Secp256k1);
            did_instance.verify_sybil_resistance(proof)
                .map_err(|e| e.to_string())
        } else {
            Err("DID not found".to_string())
        }
    }
}