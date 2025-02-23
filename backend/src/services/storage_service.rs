use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::storage::{StorageBackend, StorageResult, StorageError};

pub struct StorageService {
    pool: Arc<PgPool>,
}

impl StorageService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn store_on_chain(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        // Implement on-chain storage logic here
        Ok(())
    }

    pub async fn retrieve_on_chain(&self, key: &str) -> StorageResult<Vec<u8>> {
        // Implement on-chain retrieval logic here
        Ok(vec![])
    }

    pub async fn delete_on_chain(&self, key: &str) -> StorageResult<()> {
        // Implement on-chain deletion logic here
        Ok(())
    }

    pub async fn store_off_chain(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        self.set(key, value).await
    }

    pub async fn retrieve_off_chain(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.get(key).await
    }

    pub async fn delete_off_chain(&self, key: &str) -> StorageResult<()> {
        self.delete(key).await
    }
}

#[async_trait]
impl StorageBackend for StorageService {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO storage (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key) DO UPDATE SET value = $2
            "#,
            key,
            value
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        let result = sqlx::query!(
            r#"
            SELECT value FROM storage WHERE key = $1
            "#,
            key
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result.value)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM storage WHERE key = $1
            "#,
            key
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM storage WHERE key = $1)
            "#,
            key
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result.exists.unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;
    use std::sync::Arc;

    async fn setup_test_db() -> Arc<PgPool> {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb_test".to_string());
        Arc::new(PgPool::connect(&database_url).await.unwrap())
    }

    #[tokio::test]
    async fn test_store_on_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.store_on_chain("test_key", b"test_value").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retrieve_on_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.retrieve_on_chain("test_key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"test_value");
    }

    #[tokio::test]
    async fn test_delete_on_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.delete_on_chain("test_key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_off_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.store_off_chain("test_key", b"test_value").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retrieve_off_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.retrieve_off_chain("test_key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"test_value");
    }

    #[tokio::test]
    async fn test_delete_off_chain() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let result = storage_service.delete_off_chain("test_key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exists() {
        let pool = setup_test_db().await;
        let storage_service = StorageService::new(pool);

        let key = "test_key";
        let value = b"test_value";

        storage_service.store_off_chain(key, value).await.unwrap();
        let exists = storage_service.exists(key).await.unwrap();
        assert!(exists);

        storage_service.delete_off_chain(key).await.unwrap();
        let exists = storage_service.exists(key).await.unwrap();
        assert!(!exists);
    }
}
