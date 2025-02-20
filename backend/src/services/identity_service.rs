use async_trait::async_trait;
use std::sync::Arc;
use crate::database::db::Database;

#[async_trait]
pub trait IdentityService: Send + Sync {
    async fn create_identity(&self, identity: &str) -> Result<(), String>;
    async fn get_identity(&self, identity: &str) -> Result<String, String>;
}

pub struct IdentityServiceImpl {
    db: Arc<Database>,
}

impl IdentityServiceImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
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
}
