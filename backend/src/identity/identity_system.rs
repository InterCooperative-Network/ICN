use std::collections::HashMap;
use crate::identity::DID;

#[derive(Clone)]
pub struct IdentitySystem {
    permissions: HashMap<String, Vec<String>>,
    registered_dids: HashMap<String, DID>,
}

impl IdentitySystem {
    pub fn new() -> Self {
        IdentitySystem {
            permissions: HashMap::new(),
            registered_dids: HashMap::new(),
        }
    }

    pub fn get_permissions(&self, did: &str) -> Vec<String> {
        self.permissions.get(did)
            .cloned()
            .unwrap_or_default()
    }

    pub fn register_did(&mut self, did: DID, permissions: Vec<String>) {
        self.permissions.insert(did.id.clone(), permissions);
        self.registered_dids.insert(did.id.clone(), did);
    }

    pub fn is_registered(&self, did: &str) -> bool {
        self.registered_dids.contains_key(did)
    }

    pub fn add_permission(&mut self, did: &str, permission: String) {
        if let Some(perms) = self.permissions.get_mut(did) {
            if !perms.contains(&permission) {
                perms.push(permission);
            }
        }
    }

    pub fn remove_permission(&mut self, did: &str, permission: &str) {
        if let Some(perms) = self.permissions.get_mut(did) {
            perms.retain(|p| p != permission);
        }
    }

    pub fn get_did(&self, did: &str) -> Option<&DID> {
        self.registered_dids.get(did)
    }
}