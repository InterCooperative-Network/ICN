// src/storage.rs
use deadpool_postgres::{Pool, Manager, ManagerConfig, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls, Config as PgConfig};
use serde::{Serialize, Deserialize};
use crate::error::{StorageError, StorageResult};

pub struct StorageManager {
    pool: Pool,
}

impl StorageManager {
    pub async fn new(db_url: Option<String>) -> StorageResult<Self> {
        let mut pg_config = PgConfig::new();
        
        if let Some(_) = db_url {
            pg_config.host("localhost")
                    .port(5432)
                    .dbname("icn")
                    .user("icn")
                    .password("icn_password");
        } else {
            pg_config.host("localhost")
                    .port(5432)
                    .dbname("icn")
                    .user("icn")
                    .password("icn_password");
        }

        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };

        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        
        let pool = Pool::builder(mgr)
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| StorageError::PoolError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub(crate) async fn get_client(&self) -> StorageResult<deadpool_postgres::Client> {
        self.pool.get()
            .await
            .map_err(|e| StorageError::PoolError(e.to_string()))
    }

    pub async fn store<T: Serialize>(&self, key: &str, value: &T) -> StorageResult<()> {
        let client = self.get_client().await?;
            
        // Serialize to a JSON string
        let json_string = serde_json::to_string(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
        // Use PostgreSQL's cast to convert the string to JSONB
        client.execute(
            "INSERT INTO key_value (key, value) VALUES ($1, cast($2 as jsonb))
             ON CONFLICT (key) DO UPDATE SET value = cast($2 as jsonb), updated_at = NOW()",
            &[&key, &json_string]
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> StorageResult<T> {
        let client = self.get_client().await?;
            
        let row = client.query_opt(
            "SELECT value::text FROM key_value WHERE key = $1",
            &[&key]
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?
        .ok_or_else(|| StorageError::KeyNotFound(key.to_string()))?;
            
        // Get the JSON string and parse it
        let json_str: String = row.get(0);
        serde_json::from_str(&json_str)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    pub async fn list_keys(&self) -> StorageResult<Vec<String>> {
        let client = self.get_client().await?;
            
        let rows = client.query("SELECT key FROM key_value", &[])
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        Ok(rows.iter().map(|row| row.get("key")).collect())
    }

    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        let client = self.get_client().await?;
            
        let rows_affected = client.execute(
            "DELETE FROM key_value WHERE key = $1",
            &[&key]
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        if rows_affected == 0 {
            return Err(StorageError::KeyNotFound(key.to_string()));
        }
            
        Ok(())
    }
}