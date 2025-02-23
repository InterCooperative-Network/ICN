use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[async_trait]
pub trait StorageBackend {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()>;
    async fn get(&self, key: &str) -> StorageResult<Vec<u8>>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

pub struct StorageManager {
    backend: Box<dyn StorageBackend + Send + Sync>,
}

impl StorageManager {
    pub fn new(backend: Box<dyn StorageBackend + Send + Sync>) -> Self {
        Self { backend }
    }
}

pub struct DatabaseStorageBackend {
    pool: Arc<PgPool>,
}

impl DatabaseStorageBackend {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StorageBackend for DatabaseStorageBackend {
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

pub struct OnChainStorage {
    // Add fields for on-chain storage management
}

impl OnChainStorage {
    pub async fn store(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        // Implement on-chain storage logic here
        Ok(())
    }

    pub async fn retrieve(&self, key: &str) -> StorageResult<Vec<u8>> {
        // Implement on-chain retrieval logic here
        Ok(vec![])
    }

    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        // Implement on-chain deletion logic here
        Ok(())
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
    async fn test_set_and_get() {
        let pool = setup_test_db().await;
        let backend = DatabaseStorageBackend::new(pool);

        let key = "test_key";
        let value = b"test_value";

        backend.set(key, value).await.unwrap();
        let retrieved_value = backend.get(key).await.unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = setup_test_db().await;
        let backend = DatabaseStorageBackend::new(pool);

        let key = "test_key";
        let value = b"test_value";

        backend.set(key, value).await.unwrap();
        backend.delete(key).await.unwrap();
        let result = backend.get(key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exists() {
        let pool = setup_test_db().await;
        let backend = DatabaseStorageBackend::new(pool);

        let key = "test_key";
        let value = b"test_value";

        backend.set(key, value).await.unwrap();
        let exists = backend.exists(key).await.unwrap();
        assert!(exists);

        backend.delete(key).await.unwrap();
        let exists = backend.exists(key).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_on_chain_store() {
        let on_chain_storage = OnChainStorage {
            // Initialize fields for on-chain storage management
        };

        let key = "on_chain_key";
        let value = b"on_chain_value";

        let result = on_chain_storage.store(key, value).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_on_chain_retrieve() {
        let on_chain_storage = OnChainStorage {
            // Initialize fields for on-chain storage management
        };

        let key = "on_chain_key";

        let result = on_chain_storage.retrieve(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"on_chain_value");
    }

    #[tokio::test]
    async fn test_on_chain_delete() {
        let on_chain_storage = OnChainStorage {
            // Initialize fields for on-chain storage management
        };

        let key = "on_chain_key";

        let result = on_chain_storage.delete(key).await;
        assert!(result.is_ok());
    }
}
