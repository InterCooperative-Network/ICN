use serde::{Serialize, Deserialize};
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::thread_rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    pub public_key: String,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl DID {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Self {
            id: format!("did:icn:{}", hex::encode(public_key.serialize())),
            public_key: hex::encode(public_key.serialize()),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
