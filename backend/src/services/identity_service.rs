use async_trait::async_trait;
use std::sync::Arc;
use crate::database::db::Database;
use icn_identity::ledger::{create_identity_in_ledger, get_identity_from_ledger, rotate_key_in_ledger, revoke_key_in_ledger};
use icn_core::verifiable_credentials::{VerifiableCredential, Proof};
use futures::future::join_all; // Import join_all for concurrency

#[async_trait]
pub trait IdentityService: Send + Sync {
    async fn create_identity(&self, identity: &str) -> Result<(), String>;
    async fn get_identity(&self, identity: &str) -> Result<String, String>;
    async fn rotate_key(&self, identity: &str) -> Result<(), String>;
    async fn revoke_key(&self, identity: &str) -> Result<(), String>;
    async fn verify_did(&self, did: &str) -> Result<bool, String>; // Add verify_did method
    async fn verify_credential(&self, credential: &str) -> Result<bool, String>; // Add verify_credential method
    async fn get_public_key(&self, did: &str) -> Result<Option<Vec<u8>>, String>; // Add get_public_key method
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
        let credential = VerifiableCredential {
            credential_type: "IdentityCredential".to_string(),
            issuer_did: "did:icn:issuer".to_string(),
            subject_did: identity.to_string(),
            issuance_date: chrono::Utc::now().to_rfc3339(),
            expiration_date: None,
            credential_status: None,
            credential_schema: None,
            proof: Proof {
                type_: "Ed25519Signature2018".to_string(),
                created: chrono::Utc::now().to_rfc3339(),
                proof_purpose: "assertionMethod".to_string(),
                verification_method: "did:icn:issuer#keys-1".to_string(),
                jws: "example-jws".to_string(),
            },
        };

        create_identity_in_ledger(identity, &credential).await.map_err(|e| e.to_string())
    }

    async fn get_identity(&self, identity: &str) -> Result<String, String> {
        get_identity_from_ledger(identity).await.map_err(|e| e.to_string())
    }

    async fn rotate_key(&self, identity: &str) -> Result<(), String> {
        rotate_key_in_ledger(identity).await.map_err(|e| e.to_string())
    }

    async fn revoke_key(&self, identity: &str) -> Result<(), String> {
        revoke_key_in_ledger(identity).await.map_err(|e| e.to_string())
    }

    async fn verify_did(&self, did: &str) -> Result<bool, String> {
        // Placeholder logic for verifying DID
        Ok(true)
    }

    async fn verify_credential(&self, credential: &str) -> Result<bool, String> {
        // Placeholder logic for verifying credential
        Ok(true)
    }

    async fn get_public_key(&self, did: &str) -> Result<Option<Vec<u8>>, String> {
        // Placeholder logic for retrieving public key
        Ok(Some(vec![]))
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

    #[tokio::test]
    async fn test_verify_did() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.verify_did("did:icn:test").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_verify_credential() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.verify_credential("example-credential").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_get_public_key() {
        let pool = setup_test_db().await;
        let db = Arc::new(Database { pool });
        let service = IdentityServiceImpl::new(db);

        let result = service.get_public_key("did:icn:test").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
