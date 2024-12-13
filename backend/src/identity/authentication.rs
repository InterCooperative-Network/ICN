use crate::identity::DID;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, Signature};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Authentication {
    dids: Arc<Mutex<HashMap<String, DID>>>,
    secp: Secp256k1<secp256k1::All>,
}

impl Authentication {
    pub fn new() -> Self {
        Authentication {
            dids: Arc::new(Mutex::new(HashMap::new())),
            secp: Secp256k1::new(),
        }
    }

    pub fn register_did(&self, did: DID, secret_key: SecretKey) {
        let mut dids = self.dids.lock().unwrap();
        dids.insert(did.id.clone(), did);
    }

    pub fn authenticate(&self, did: &str, message: &str, signature: &str) -> bool {
        let dids = self.dids.lock().unwrap();
        if let Some(did) = dids.get(did) {
            let message_hash = Sha256::digest(message.as_bytes());
            let message = Message::from_slice(&message_hash).expect("32 bytes");
            let signature = Signature::from_str(signature).expect("valid signature");
            self.secp.verify(&message, &signature, &did.public_key).is_ok()
        } else {
            false
        }
    }
}
