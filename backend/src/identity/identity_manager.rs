use crate::identity::{DID, IdentitySystem};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct IdentityManager {
    identity_system: Arc<Mutex<IdentitySystem>>,
    dids: HashMap<String, DID>,
}

impl IdentityManager {
    pub fn new(identity_system: Arc<Mutex<IdentitySystem>>) -> Self {
        IdentityManager {
            identity_system,
            dids: HashMap::new(),
        }
    }

    pub fn register_did(&mut self, did: DID) {
        let mut identity_system = self.identity_system.lock().unwrap();
        identity_system.register_did(did.clone(), vec![]);
        self.dids.insert(did.id.clone(), did);
    }

    pub fn get_did(&self, did: &str) -> Option<&DID> {
        self.dids.get(did)
    }

    pub fn is_registered(&self, did: &str) -> bool {
        self.dids.contains_key(did)
    }

    pub fn add_permission(&mut self, did: &str, permission: String) {
        let mut identity_system = self.identity_system.lock().unwrap();
        identity_system.add_permission(did, permission);
    }

    pub fn remove_permission(&mut self, did: &str, permission: &str) {
        let mut identity_system = self.identity_system.lock().unwrap();
        identity_system.remove_permission(did, permission);
    }

    pub fn get_permissions(&self, did: &str) -> Vec<String> {
        let identity_system = self.identity_system.lock().unwrap();
        identity_system.get_permissions(did)
    }
}
