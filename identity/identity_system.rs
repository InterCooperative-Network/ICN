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
    reputation_scores: HashMap<String, HashMap<String, i64>>, // Multi-dimensional reputation scores
    last_activity: HashMap<String, SystemTime>,
    key_versions: HashMap<String, u32>,
    federation_roles: HashMap<String, HashMap<String, Vec<String>>>, // Federation-specific roles
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
            federation_roles: HashMap::new(),
        }
    }

    pub fn register_did(&mut self, did: String, permissions: Vec<String>, initial_reputation: i64, public_key: Vec<u8>, algorithm: Algorithm) {
        self.permissions.insert(did.clone(), permissions);
        self.reputation_scores.insert(did.clone(), {
            let mut categories = HashMap::new();
            categories.insert("general".to_string(), initial_reputation);
            categories.insert("governance".to_string(), initial_reputation);
            categories.insert("resource_sharing".to_string(), initial_reputation);
            categories.insert("technical_contributions".to_string(), initial_reputation);
            categories
        });
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

    pub fn assign_federation_role(&mut self, federation_id: String, did: String, role: String) {
        self.federation_roles.entry(federation_id).or_insert_with(HashMap::new).entry(did).or_insert_with(Vec::new).push(role);
    }

    pub fn get_federation_roles(&self, federation_id: &str, did: &str) -> Vec<String> {
        self.federation_roles.get(federation_id).and_then(|roles| roles.get(did)).cloned().unwrap_or_default()
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

    pub fn get_reputation(&self, did: &str, category: &str) -> i64 {
        self.reputation_scores.get(did).and_then(|categories| categories.get(category)).cloned().unwrap_or(0)
    }

    pub fn adjust_reputation(&mut self, did: &str, change: i64, category: &str) {
        if let Some(categories) = self.reputation_scores.get_mut(did) {
            if let Some(score) = categories.get_mut(category) {
                *score += change;
            }
        }
    }

    pub fn is_eligible(&self, did: &str, min_reputation: i64, category: &str) -> bool {
        self.get_reputation(did, category) >= min_reputation
    }

    pub fn dynamic_recalibration(&mut self) {
        let now = SystemTime::now();
        for (did, last_active) in &self.last_activity {
            if let Ok(duration) = now.duration_since(*last_active) {
                if duration > Duration::from_secs(30 * 24 * 60 * 60) { // 30 days
                    self.adjust_reputation(did, -1, "general"); // Decay general reputation
                    self.adjust_reputation(did, -1, "governance"); // Decay governance reputation
                    self.adjust_reputation(did, -1, "resource_sharing"); // Decay resource sharing reputation
                    self.adjust_reputation(did, -1, "technical_contributions"); // Decay technical contributions reputation
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

    #[test]
    fn test_reputation_decay() {
        let mut identity_system = IdentitySystem::new();
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 100, vec![], Algorithm::Secp256k1);

        identity_system.dynamic_recalibration();
        assert_eq!(identity_system.get_reputation(&did, "general"), 99);
        assert_eq!(identity_system.get_reputation(&did, "governance"), 99);
        assert_eq!(identity_system.get_reputation(&did, "resource_sharing"), 99);
        assert_eq!(identity_system.get_reputation(&did, "technical_contributions"), 99);
    }

    #[test]
    fn test_multi_dimensional_reputation_tracking() {
        let mut identity_system = IdentitySystem::new();
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 50, vec![], Algorithm::Secp256k1);

        identity_system.adjust_reputation(&did, 20, "governance");
        identity_system.adjust_reputation(&did, 30, "resource_sharing");

        assert_eq!(identity_system.get_reputation(&did, "governance"), 70);
        assert_eq!(identity_system.get_reputation(&did, "resource_sharing"), 80);
    }

    #[test]
    fn test_category_specific_eligibility_checks() {
        let mut identity_system = IdentitySystem::new();
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 40, vec![], Algorithm::Secp256k1);

        identity_system.adjust_reputation(&did, 10, "governance");

        assert!(identity_system.is_eligible(&did, 30, "governance"));
        assert!(!identity_system.is_eligible(&did, 50, "governance"));
    }

    #[test]
    fn test_assign_and_get_federation_roles() {
        let mut identity_system = IdentitySystem::new();
        let federation_id = "federation123".to_string();
        let did = "did:example:secp256k1".to_string();
        identity_system.register_did(did.clone(), vec!["read".to_string()], 10, vec![], Algorithm::Secp256k1);

        identity_system.assign_federation_role(federation_id.clone(), did.clone(), "admin".to_string());
        identity_system.assign_federation_role(federation_id.clone(), did.clone(), "member".to_string());

        let roles = identity_system.get_federation_roles(&federation_id, &did);
        assert_eq!(roles, vec!["admin".to_string(), "member".to_string()]);
    }
}
