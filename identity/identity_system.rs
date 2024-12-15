use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use crate::Algorithm;

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

    pub fn register_did(&mut self, did: String, permissions: Vec<String>, initial_reputation: i64) {
        self.permissions.insert(did.clone(), permissions);
        self.reputation_scores.insert(did.clone(), initial_reputation);
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

    pub fn register_public_key(&mut self, did: String, public_key: Vec<u8>, algorithm: Algorithm) {
        self.public_keys.insert(did, (public_key, algorithm));
    }

    pub fn verify_did(&self, did: &str, message: &[u8], signature: &[u8]) -> bool {
        if let Some((public_key, algorithm)) = self.public_keys.get(did) {
            let key_pair = crate::KeyPair {
                public_key: public_key.clone(),
                private_key: vec![], // Not used for verification
                algorithm: algorithm.clone(),
            };
            key_pair.verify(message, signature)
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

    pub fn rotate_key(&mut self, did: &str, new_public_key: Vec<u8>, algorithm: Algorithm) {
        if let Some(version) = self.key_versions.get_mut(did) {
            *version += 1;
            self.public_keys.insert(did.to_string(), (new_public_key, algorithm));
        }
    }

    pub fn get_key_version(&self, did: &str) -> Option<u32> {
        self.key_versions.get(did).cloned()
    }
}
