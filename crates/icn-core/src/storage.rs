use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct StorageManager {
    references: HashMap<String, StorageReference>,
    pending_uploads: HashMap<String, StorageRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageRequest {
    pub resource_id: String,
    pub storage_type: StorageType,
    pub size: u64,
    pub chunk_size: u32,
    pub status: StorageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoragePermission {
    ReadOnly,
    ReadWrite,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAccess {
    pub permission: StoragePermission,
    pub expires_at: Option<DateTime<Utc>>,
    pub storage_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffChainStorageProof {
    pub storage_location: String,
    pub content_hash: String,
    pub last_verified_at: DateTime<Utc>,
    pub access_control: HashMap<String, StorageAccess>, // DID -> Access
}

impl OffChainStorageProof {
    pub fn verify_access(&self, did: &str, requested_permission: &StoragePermission) -> bool {
        if let Some(access) = self.access_control.get(did) {
            if let Some(expiry) = access.expires_at {
                if expiry < Utc::now() {
                    return false;
                }
            }
            
            matches!(
                (&access.permission, requested_permission),
                (StoragePermission::Admin, _) |
                (StoragePermission::ReadWrite, StoragePermission::ReadOnly) |
                (StoragePermission::ReadWrite, StoragePermission::ReadWrite) |
                (StoragePermission::ReadOnly, StoragePermission::ReadOnly)
            )
        } else {
            false
        }
    }
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
            pending_uploads: HashMap::new(),
        }
    }

    pub fn add_storage_reference(&mut self, reference: StorageReference) -> Result<(), StorageError> {
        if self.references.contains_key(&reference.metadata.content_hash) {
            return Err(StorageError::ReferenceAlreadyExists);
        }
        self.references.insert(reference.metadata.content_hash.clone(), reference);
        Ok(())
    }

    pub fn get_storage_location(&self, content_hash: &str) -> Option<&StorageReference> {
        self.references.get(content_hash)
    }
}
