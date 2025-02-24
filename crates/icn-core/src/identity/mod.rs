use std::collections::HashMap;
use tokio::sync::RwLock;
use icn_types::{MemberId, CooperativeId};
use async_trait::async_trait;

pub struct IdentityManager {
    members: RwLock<HashMap<String, MemberId>>,
    cooperatives: RwLock<HashMap<String, CooperativeId>>,
}

#[async_trait]
pub trait IdentityInterface: Send + Sync {
    async fn register_member(&self, did: String, cooperative_id: CooperativeId) -> Result<MemberId, String>;
    async fn verify_member(&self, did: &str) -> bool;
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
}

#[async_trait]
impl IdentityInterface for IdentityManager {
    async fn register_member(&self, did: String, cooperative_id: CooperativeId) -> Result<MemberId, String> {
        let member_id = MemberId {
            did: did.clone(),
            cooperative_id: cooperative_id.clone(),
        };
        
        self.members.write().await.insert(did, member_id.clone());
        Ok(member_id)
    }

    async fn verify_member(&self, did: &str) -> bool {
        self.members.read().await.contains_key(did)
    }

    async fn start(&self) -> Result<(), String> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        Ok(())
    }
}

impl IdentityManager {
    pub fn new() -> Self {
        Self {
            members: RwLock::new(HashMap::new()),
            cooperatives: RwLock::new(HashMap::new()),
        }
    }
}
