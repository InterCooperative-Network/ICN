use async_trait::async_trait;
use std::sync::Arc;
use crate::database::db::Database;

#[async_trait]
pub trait IdentityService: Send + Sync {
    async fn create_identity(&self, identity: &str) -> Result<(), String>;
    async fn get_identity(&self, identity: &str) -> Result<String, String>;
    async fn rotate_key(&self, identity: &str) -> Result<(), String>;
    async fn revoke_key(&self, identity: &str) -> Result<(), String>;
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

    async fn rotate_key(&self, identity: &str) -> Result<(), String> {
        // Implement key rotation logic here
        Ok(())
    }

    async fn revoke_key(&self, identity: &str) -> Result<(), String> {
        // Implement key revocation logic here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;

    async fn setup_test_db() -> PgPool {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_identity() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.create_identity("did:icn:test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_identity() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let _ = service.create_identity("did:icn:test").await;
        let result = service.get_identity("did:icn:test").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_rotate_key() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.rotate_key("did:icn:test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.revoke_key("did:icn:test").await;
        assert!(result.is_ok());
    }
}
