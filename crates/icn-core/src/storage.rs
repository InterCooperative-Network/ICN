use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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
