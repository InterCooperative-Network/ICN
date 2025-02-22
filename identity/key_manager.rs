use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey, ED25519};
use threshold_crypto::{SecretKeySet, SecretKeyShare, PublicKeySet, SignatureShare};
use rsa::{RSAPrivateKey, RSAPublicKey, PaddingScheme};
use bls_signatures::{PrivateKey as BlsPrivateKey, PublicKey as BlsPublicKey, Signature as BlsSignature, Serialize as BlsSerialize, AggregatePublicKey, AggregateSignature};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

pub struct KeyManager {
    key_pairs: HashMap<String, Ed25519KeyPair>,
    threshold_keys: HashMap<String, (SecretKeySet, PublicKeySet)>,
    rsa_keys: HashMap<String, (RSAPrivateKey, RSAPublicKey)>,
    bls_keys: HashMap<String, (BlsPrivateKey, BlsPublicKey)>,
    key_rotation_schedule: HashMap<String, SystemTime>,
    key_revocation_schedule: HashMap<String, SystemTime>,
}

impl KeyManager {
    pub fn new() -> Self {
        KeyManager {
            key_pairs: HashMap::new(),
            threshold_keys: HashMap::new(),
            rsa_keys: HashMap::new(),
            bls_keys: HashMap::new(),
            key_rotation_schedule: HashMap::new(),
            key_revocation_schedule: HashMap::new(),
        }
    }

    pub fn generate_ed25519_key_pair(&mut self, identity: &str) {
        let rng = SystemRandom::new();
        let key_pair = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).unwrap();
        self.key_pairs.insert(identity.to_string(), key_pair);
    }

    pub fn generate_threshold_keys(&mut self, identity: &str, threshold: usize, total: usize) {
        let sks = SecretKeySet::random(threshold, &mut rand::thread_rng());
        let pks = sks.public_keys();
        self.threshold_keys.insert(identity.to_string(), (sks, pks));
    }

    pub fn generate_rsa_keys(&mut self, identity: &str, bits: usize) {
        let private_key = RSAPrivateKey::new(&mut rand::thread_rng(), bits).unwrap();
        let public_key = RSAPublicKey::from(&private_key);
        self.rsa_keys.insert(identity.to_string(), (private_key, public_key));
    }

    pub fn generate_bls_keys(&mut self, identity: &str) {
        let private_key = BlsPrivateKey::generate(&mut rand::thread_rng());
        let public_key = BlsPublicKey::from(&private_key);
        self.bls_keys.insert(identity.to_string(), (private_key, public_key));
    }

    pub fn rotate_key(&mut self, identity: &str) {
        self.generate_ed25519_key_pair(identity);
        self.key_rotation_schedule.insert(identity.to_string(), SystemTime::now());
    }

    pub fn revoke_key(&mut self, identity: &str) {
        self.key_pairs.remove(identity);
        self.threshold_keys.remove(identity);
        self.rsa_keys.remove(identity);
        self.bls_keys.remove(identity);
        self.key_revocation_schedule.insert(identity.to_string(), SystemTime::now());
    }

    pub async fn start_key_rotation(&self) {
        let key_manager = self.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut key_manager = key_manager.lock().unwrap();
                    key_manager.check_key_rotation_schedule();
                }
                sleep(Duration::from_secs(10)).await;
            }
        });
    }

    pub async fn start_key_revocation(&self) {
        let key_manager = self.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut key_manager = key_manager.lock().unwrap();
                    key_manager.check_key_revocation_schedule();
                }
                sleep(Duration::from_secs(10)).await;
            }
        });
    }

    fn check_key_rotation_schedule(&mut self) {
        let now = SystemTime::now();
        for (identity, &rotation_time) in &self.key_rotation_schedule {
            if let Ok(duration) = now.duration_since(rotation_time) {
                if duration > Duration::from_secs(30 * 24 * 60 * 60) { // 30 days
                    self.rotate_key(identity);
                }
            }
        }
    }

    fn check_key_revocation_schedule(&mut self) {
        let now = SystemTime::now();
        for (identity, &revocation_time) in &self.key_revocation_schedule {
            if let Ok(duration) = now.duration_since(revocation_time) {
                if duration > Duration::from_secs(30 * 24 * 60 * 60) { // 30 days
                    self.revoke_key(identity);
                }
            }
        }
    }
}
