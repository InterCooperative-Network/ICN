use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use secp256k1::{Secp256k1, PublicKey as Secp256k1PublicKey, Signature as Secp256k1Signature};
use rsa::{RSAPublicKey, PaddingScheme};
use ecdsa::{VerifyingKey, signature::Verifier};
use sha2::{Sha256, Digest};
use crate::did::creation::Algorithm;
use crate::did::creation::DID;
use crate::did::creation::DIDError;
use tokio::time::sleep;

pub struct IdentitySystem {
    permissions: HashMap<String, Vec<String>>,
    roles: HashMap<String, Vec<String>>,
    public_keys: HashMap<String, (Vec<u8>, Algorithm)>,
    reputation_scores: HashMap<String, i64>,
    last_activity: HashMap<String, SystemTime>,
    key_versions: HashMap<String, u32>,
}

impl IdentitySystem {
    pub fn new() -> Self {
        IdentitySystem {
            permissions: HashMap::new(),
            roles: HashMap::new(),
            public_keys: HashMap::new(),
            reputation_scores: HashMap::new(),
            last_activity: HashMap::new(),
            key_versions: HashMap::new(),
        }
    }

    pub fn register_did(&mut self, did: String, permissions: Vec<String>, initial_reputation: i64, public_key: Vec<u8>, algorithm: Algorithm) {
        self.permissions.insert(did.clone(), permissions);
        self.reputation_scores.insert(did.clone(), initial_reputation);
        self.public_keys.insert(did.clone(), (public_key, algorithm));
        self.last_activity.insert(did.clone(), SystemTime::now());
        self.key_versions.insert(did, 1);
    }

    pub fn has_permission(&self, did: &str, permission: &str) -> bool {
        if let Some(perms) = self.permissions.get(did) {
            perms.contains(&permission.to_string())
        } else {
            false
        }
    }

    pub fn assign_role(&mut self, did: String, role: String) {
        self.roles.entry(did).or_insert_with(Vec::new).push(role);
    }

    pub fn get_roles(&self, did: &str) -> Vec<String> {
        self.roles.get(did).cloned().unwrap_or_default()
    }

    pub fn verify_did(&self, did: &str, message: &[u8], signature: &[u8]) -> bool {
        if let Some((public_key, algorithm)) = self.public_keys.get(did) {
            match algorithm {
                Algorithm::Secp256k1 => {
                    let secp = Secp256k1::new();
                    let public_key = Secp256k1PublicKey::from_slice(public_key).expect("invalid public key");
                    let msg = secp256k1::Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
                    let signature = Secp256k1Signature::from_compact(signature).expect("invalid signature");
                    secp.verify(&msg, &signature, &public_key).is_ok()
                },
                Algorithm::RSA => {
                    let public_key = RSAPublicKey::from_pkcs1(public_key).expect("failed to decode public key");
                    let padding = PaddingScheme::new_pkcs1v15_sign(None);
                    public_key.verify(padding, &Sha256::digest(message), signature).is_ok()
                },
                Algorithm::ECDSA => {
                    let verifying_key = VerifyingKey::from_bytes(public_key).expect("failed to decode public key");
                    verifying_key.verify(message, signature).is_ok()
                },
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_reputation(&self, did: &str) -> i64 {
        *self.reputation_scores.get(did).unwrap_or(&0)
    }

    pub fn adjust_reputation(&mut self, did: &str, change: i64) {
        if let Some(score) = self.reputation_scores.get_mut(did) {
            *score += change;
        }
    }

    pub fn is_eligible(&self, did: &str, min_reputation: i64) -> bool {
        self.get_reputation(did) >= min_reputation
    }

    pub fn dynamic_recalibration(&mut self) {
        let now = SystemTime::now();
        for (did, last_active) in &self.last_activity {
            if let Ok(duration) = now.duration_since(*last_active) {
                if duration > Duration::from_secs(30 * 24 * 60 * 60) { // 30 days
                    self.adjust_reputation(did, -1); // Decay reputation
                }
            }
        }
    }

    pub fn update_last_activity(&mut self, did: &str) {
        self.last_activity.insert(did.to_string(), SystemTime::now());
    }

    pub fn rotate_key(&mut self, did: &str) -> Result<(), DIDError> {
        if let Some((public_key, algorithm)) = self.public_keys.get_mut(did) {
            let mut did_instance = DID::new(did.to_string(), algorithm.clone());
            did_instance.rotate_key()?;
            *public_key = did_instance.public_key.clone();
            if let Some(version) = self.key_versions.get_mut(did) {
                *version += 1;
            }
            Ok(())
        } else {
            Err(DIDError::KeyRotation)
        }
    }

    pub async fn start_real_time_recalibration(&self) {
        let reputation_system = self.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut reputation = reputation_system.lock().unwrap();
                    reputation.dynamic_recalibration();
                }
                sleep(Duration::from_secs(10)).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Signature};
    use rsa::{RSAPrivateKey, RSAPublicKey};
    use ecdsa::{SigningKey, VerifyingKey, signature::Signer};

    #[test]
    fn test_register_and_verify_did_secp256k1() {
        let mut identity_system = IdentitySystem::new();
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 10, public_key.serialize().to_vec(), Algorithm::Secp256k1);

        let message = b"test message";
        let msg = secp256k1::Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
        let signature = secp.sign(&msg, &secret_key).serialize_compact().to_vec();

        assert!(identity_system.verify_did(&did, message, &signature));
    }

    #[test]
    fn test_register_and_verify_did_rsa() {
        let mut identity_system = IdentitySystem::new();
        let private_key = RSAPrivateKey::new(&mut rand::thread_rng(), 2048).expect("failed to generate a key");
        let public_key = RSAPublicKey::from(&private_key);
        let did = "did:example:rsa".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 10, public_key.to_pkcs1().expect("failed to encode public key"), Algorithm::RSA);

        let message = b"test message";
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        let signature = private_key.sign(padding, &Sha256::digest(message)).expect("failed to sign message");

        assert!(identity_system.verify_did(&did, message, &signature));
    }

    #[test]
    fn test_register_and_verify_did_ecdsa() {
        let mut identity_system = IdentitySystem::new();
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let verifying_key = VerifyingKey::from(&signing_key);
        let did = "did:example:ecdsa".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 10, verifying_key.to_bytes().to_vec(), Algorithm::ECDSA);

        let message = b"test message";
        let signature = signing_key.sign(message).to_bytes().to_vec();

        assert!(identity_system.verify_did(&did, message, &signature));
    }

    #[test]
    fn test_key_rotation() {
        let mut identity_system = IdentitySystem::new();
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 10, public_key.serialize().to_vec(), Algorithm::Secp256k1);

        let old_public_key = identity_system.public_keys.get(&did).unwrap().0.clone();
        identity_system.rotate_key(&did).unwrap();
        let new_public_key = identity_system.public_keys.get(&did).unwrap().0.clone();

        assert_ne!(old_public_key, new_public_key);
    }
}
