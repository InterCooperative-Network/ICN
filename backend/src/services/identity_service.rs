use async_trait::async_trait;
use std::sync::Arc;
use crate::database::db::Database;
use crate::identity::identity_system::IdentitySystem;

#[async_trait]
pub trait IdentityService: Send + Sync {
    async fn create_identity(&self, identity: &str) -> Result<(), String>;
    async fn get_identity(&self, identity: &str) -> Result<String, String>;
    async fn rotate_key(&self, identity: &str) -> Result<(), String>;
    async fn revoke_key(&self, identity: &str) -> Result<(), String>;
}

pub struct IdentityServiceImpl {
    db: Arc<Database>,
    identity_system: Arc<IdentitySystem>,
}

impl IdentityServiceImpl {
    pub fn new(db: Arc<Database>, identity_system: Arc<IdentitySystem>) -> Self {
        Self { db, identity_system }
    }
}

#[async_trait]
impl IdentityService for IdentityServiceImpl {
    async fn create_identity(&self, identity: &str) -> Result<(), String> {
        self.db.store_identity(identity, "").await.map_err(|e| e.to_string())
    }

    async fn get_identity(&self, identity: &str) -> Result<String, String> {
        self.db.retrieve_identity(identity).await.map_err(|e| e.to_string())
    }

    async fn rotate_key(&self, identity: &str) -> Result<(), String> {
        self.identity_system.rotate_key(identity).map_err(|e| e.to_string())
    }

    async fn revoke_key(&self, identity: &str) -> Result<(), String> {
        self.identity_system.revoke_key(identity).map_err(|e| e.to_string())
    }
}
