// src/storage/postgres.rs

use super::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use tokio_postgres::{Client, Config, NoTls};
use std::str::FromStr;

/// PostgreSQL implementation of the storage backend
pub struct PostgresStorage {
    client: Client,
}

impl PostgresStorage {
    /// Create a new PostgreSQL storage instance
    pub async fn new(connection_str: &str) -> Result<Self, StorageError> {
        // Parse the connection string
        let config = Config::from_str(connection_str)
            .map_err(|e| StorageError::DatabaseError(format!("Invalid connection string: {}", e)))?;
            
        // Connect to the database
        let (client, connection) = config
            .connect(NoTls)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Connection failed: {}", e)))?;
            
        // Spawn the connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });
        
        // Create the storage table if it doesn't exist
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS storage (
                    key TEXT PRIMARY KEY,
                    value BYTEA NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                )",
                &[],
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Table creation failed: {}", e)))?;

        // Create an updated_at trigger
        client
            .execute(
                "CREATE OR REPLACE FUNCTION update_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = CURRENT_TIMESTAMP;
                    RETURN NEW;
                END;
                $$ language 'plpgsql'",
                &[],
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Trigger function creation failed: {}", e)))?;

        client
            .execute(
                "DROP TRIGGER IF EXISTS update_storage_updated_at ON storage;
                CREATE TRIGGER update_storage_updated_at
                    BEFORE UPDATE ON storage
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at()",
                &[],
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Trigger creation failed: {}", e)))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl StorageBackend for PostgresStorage {
    async fn set(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        self.client
            .execute(
                "INSERT INTO storage (key, value) VALUES ($1, $2)
                 ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
                &[&key, &value],
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Insert failed: {}", e)))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        let row = self.client
            .query_one("SELECT value FROM storage WHERE key = $1", &[&key])
            .await
            .map_err(|e| match e {
                tokio_postgres::Error::RowNotFound => StorageError::NotFound(key.to_string()),
                _ => StorageError::DatabaseError(format!("Query failed: {}", e)),
            })?;
            
        let value: Vec<u8> = row.get(0);
        Ok(value)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let rows_affected = self.client
            .execute("DELETE FROM storage WHERE key = $1", &[&key])
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Delete failed: {}", e)))?;
            
        if rows_affected == 0 {
            return Err(StorageError::NotFound(key.to_string()));
        }
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let result = self.client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM storage WHERE key = $1)",
                &[&key],
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("Query failed: {}", e)))?;
            
        let exists: bool = result.get(0);
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // Helper function to create test database connection
    async fn create_test_db() -> PostgresStorage {
        let connection_str = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test".to_string());
            
        PostgresStorage::new(&connection_str)
            .await
            .expect("Failed to create test database connection")
    }

    #[tokio::test]
    #[serial]
    async fn test_basic_operations() {
        let storage = create_test_db().await;
        let test_key = "test_key";
        let test_value = b"test_value".to_vec();

        // Test set and get
        storage.set(test_key, &test_value).await.unwrap();
        let retrieved = storage.get(test_key).await.unwrap();
        assert_eq!(retrieved, test_value);

        // Test exists
        assert!(storage.exists(test_key).await.unwrap());

        // Test delete
        storage.delete(test_key).await.unwrap();
        assert!(!storage.exists(test_key).await.unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_not_found() {
        let storage = create_test_db().await;
        let result = storage.get("nonexistent_key").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
}