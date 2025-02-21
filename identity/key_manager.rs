use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use ring::signature::{self, KeyPair, Ed25519KeyPair};
use ring::rand::SystemRandom;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use crate::did::Algorithm;

#[derive(Error, Debug)]
pub enum KeyManagerError {
    #[error("Key generation failed")]
    KeyGenerationError,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Key rotation failed")]
    RotationFailed,
    #[error("Key revocation failed")]
    RevocationFailed,
    #[error("Key backup failed")]
    BackupFailed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub created_at: SystemTime,
    pub rotated_at: Option<SystemTime>,
    pub version: u32,
    pub status: KeyStatus,
    pub algorithm: Algorithm,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyStatus {
    Active,
    Revoked,
    Expired,
    Rotating,
}

pub struct KeyManager {
    keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    metadata: Arc<Mutex<HashMap<String, KeyMetadata>>>,
    backup_keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    rng: SystemRandom,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
            metadata: Arc::new(Mutex::new(HashMap::new())),
            backup_keys: Arc::new(Mutex::new(HashMap::new())),
            rng: SystemRandom::new(),
        }
    }

    pub fn generate_key(&self, did: &str, algorithm: Algorithm) -> Result<Vec<u8>, KeyManagerError> {
        let key_pair = Ed25519KeyPair::generate_pkcs8(&self.rng)
            .map_err(|_| KeyManagerError::KeyGenerationError)?;
        
        let public_key = key_pair.public_key().as_ref().to_vec();
        
        // Store the key and metadata
        {
            let mut keys = self.keys.lock().unwrap();
            let mut metadata = self.metadata.lock().unwrap();
            
            keys.insert(did.to_string(), key_pair.to_pkcs8_der().as_ref().to_vec());
            metadata.insert(did.to_string(), KeyMetadata {
                created_at: SystemTime::now(),
                rotated_at: None,
                version: 1,
                status: KeyStatus::Active,
                algorithm,
            });
        }
        
        // Create backup key
        self.create_backup(did)?;
        
        Ok(public_key)
    }

    pub fn rotate_key(&self, did: &str) -> Result<Vec<u8>, KeyManagerError> {
        let mut keys = self.keys.lock().unwrap();
        let mut metadata = self.metadata.lock().unwrap();
        
        let meta = metadata.get_mut(did).ok_or(KeyManagerError::KeyNotFound)?;
        if meta.status == KeyStatus::Revoked {
            return Err(KeyManagerError::RotationFailed);
        }

        // Generate new key pair
        let new_key_pair = Ed25519KeyPair::generate_pkcs8(&self.rng)
            .map_err(|_| KeyManagerError::KeyGenerationError)?;
        
        // Update metadata
        meta.rotated_at = Some(SystemTime::now());
        meta.version += 1;
        meta.status = KeyStatus::Active;

        // Store new key
        keys.insert(did.to_string(), new_key_pair.to_pkcs8_der().as_ref().to_vec());
        
        // Create new backup
        self.create_backup(did)?;

        Ok(new_key_pair.public_key().as_ref().to_vec())
    }

    pub fn revoke_key(&self, did: &str) -> Result<(), KeyManagerError> {
        let mut metadata = self.metadata.lock().unwrap();
        
        let meta = metadata.get_mut(did).ok_or(KeyManagerError::KeyNotFound)?;
        meta.status = KeyStatus::Revoked;
        
        Ok(())
    }

    fn create_backup(&self, did: &str) -> Result<(), KeyManagerError> {
        let keys = self.keys.lock().unwrap();
        let mut backup_keys = self.backup_keys.lock().unwrap();
        
        if let Some(key) = keys.get(did) {
            backup_keys.insert(did.to_string(), key.clone());
            Ok(())
        } else {
            Err(KeyManagerError::BackupFailed)
        }
    }

    pub fn get_key_metadata(&self, did: &str) -> Result<KeyMetadata, KeyManagerError> {
        let metadata = self.metadata.lock().unwrap();
        metadata.get(did)
            .cloned()
            .ok_or(KeyManagerError::KeyNotFound)
    }

    pub fn restore_from_backup(&self, did: &str) -> Result<Vec<u8>, KeyManagerError> {
        let backup_keys = self.backup_keys.lock().unwrap();
        let mut keys = self.keys.lock().unwrap();
        let mut metadata = self.metadata.lock().unwrap();
        
        if let Some(backup_key) = backup_keys.get(did) {
            keys.insert(did.to_string(), backup_key.clone());
            if let Some(meta) = metadata.get_mut(did) {
                meta.status = KeyStatus::Active;
            }
            Ok(backup_key.clone())
        } else {
            Err(KeyManagerError::KeyNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_key_generation() {
        let key_manager = KeyManager::new();
        let did = "did:example:123";
        
        let public_key = key_manager.generate_key(did, Algorithm::Secp256k1).unwrap();
        assert!(!public_key.is_empty());
        
        let metadata = key_manager.get_key_metadata(did).unwrap();
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.status, KeyStatus::Active);
    }

    #[test]
    fn test_key_rotation() {
        let key_manager = KeyManager::new();
        let did = "did:example:123";
        
        let original_key = key_manager.generate_key(did, Algorithm::Secp256k1).unwrap();
        let rotated_key = key_manager.rotate_key(did).unwrap();
        
        assert_ne!(original_key, rotated_key);
        
        let metadata = key_manager.get_key_metadata(did).unwrap();
        assert_eq!(metadata.version, 2);
        assert_eq!(metadata.status, KeyStatus::Active);
        assert!(metadata.rotated_at.is_some());
    }

    #[test]
    fn test_key_revocation() {
        let key_manager = KeyManager::new();
        let did = "did:example:123";
        
        key_manager.generate_key(did, Algorithm::Secp256k1).unwrap();
        key_manager.revoke_key(did).unwrap();
        
        let metadata = key_manager.get_key_metadata(did).unwrap();
        assert_eq!(metadata.status, KeyStatus::Revoked);
    }

    #[test]
    fn test_backup_restore() {
        let key_manager = KeyManager::new();
        let did = "did:example:123";
        
        key_manager.generate_key(did, Algorithm::Secp256k1).unwrap();
        key_manager.revoke_key(did).unwrap();
        
        let restored_key = key_manager.restore_from_backup(did).unwrap();
        assert!(!restored_key.is_empty());
        
        let metadata = key_manager.get_key_metadata(did).unwrap();
        assert_eq!(metadata.status, KeyStatus::Active);
    }
}
