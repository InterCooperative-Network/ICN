// src/state/migrations.rs
use crate::error::{StorageError, StorageResult};
use crate::storage::StorageManager;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

impl Migration {
    pub fn initial_schema() -> Self {
        Self {
            name: String::from("001_initial_schema"),
            up_sql: String::from(r#"
                CREATE TABLE IF NOT EXISTS key_value (
                    key TEXT PRIMARY KEY,
                    value JSONB NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );

                CREATE INDEX IF NOT EXISTS key_value_key_idx ON key_value(key);
                
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ language 'plpgsql';

                CREATE TRIGGER update_key_value_updated_at
                    BEFORE UPDATE ON key_value
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                
                CREATE TABLE IF NOT EXISTS migrations (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
            "#),
            down_sql: String::from(r#"
                DROP TRIGGER IF EXISTS update_key_value_updated_at ON key_value;
                DROP FUNCTION IF EXISTS update_updated_at_column();
                DROP TABLE IF EXISTS key_value;
                DROP TABLE IF EXISTS migrations;
            "#),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    applied_migrations: Vec<String>,
    version: i64,
}

pub struct Migrator {
    storage: Arc<StorageManager>,
}

impl Migrator {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }

    pub async fn check_migration_applied(&self, name: &str) -> Result<bool, StorageError> {
        // Get client directly since we can't use key_value table yet
        let client = self.storage.get_client().await?;
        
        let result = client.query_one(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_name = 'migrations'
            )",
            &[],
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let migrations_exists: bool = result.get(0);
        if !migrations_exists {
            return Ok(false);
        }

        let result = client.query_one(
            "SELECT EXISTS (
                SELECT 1 FROM migrations WHERE name = $1
            )",
            &[&name],
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(result.get(0))
    }

    pub async fn record_migration(&self, name: &str) -> Result<(), StorageError> {
        let client = self.storage.get_client().await?;
        
        client.execute(
            "INSERT INTO migrations (name) VALUES ($1)",
            &[&name],
        ).await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn apply_migration(&self, migration: &Migration) -> StorageResult<()> {
        // Check if migration was already applied
        if self.check_migration_applied(&migration.name).await? {
            println!("Migration {} already applied, skipping", migration.name);
            return Ok(());
        }

        println!("Applying migration: {}", migration.name);
        
        // Get database connection and apply migration
        let client = self.storage.get_client().await?;
        client.batch_execute(&migration.up_sql)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Record migration as applied
        self.record_migration(&migration.name).await?;

        println!("Migration {} completed successfully", migration.name);
        Ok(())
    }
}