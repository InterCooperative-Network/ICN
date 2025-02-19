use async_trait::async_trait;
use sqlx::PgPool;

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
    pool: PgPool,
}

impl DatabaseStorageBackend {
    pub fn new(pool: PgPool) -> Self {
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
        .execute(&self.pool)
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
        .fetch_one(&self.pool)
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
        .execute(&self.pool)
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
        .fetch_one(&self.pool)
        .await?;
        Ok(result.exists.unwrap_or(false))
    }
}
