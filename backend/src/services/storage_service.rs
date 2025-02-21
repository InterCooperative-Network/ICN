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
