// src/state/migrations.rs

use super::{Migration, StateManager};
use crate::storage::StorageResult;
use tokio_postgres::Client;
use std::collections::HashMap;

/// Represents a database schema migration
pub struct SchemaMigration {
    pub version: u32,
    pub description: String,
    pub up_sql: String,
    pub down_sql: String,
}

/// Manages database schema migrations
pub struct MigrationManager {
    migrations: HashMap<u32, SchemaMigration>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Register a new migration
    pub fn register_migration(&mut self, migration: SchemaMigration) {
        self.migrations.insert(migration.version, migration);
    }

    /// Apply all pending migrations
    pub async fn apply_migrations(
        &self,
        state: &StateManager,
        client: &Client,
    ) -> StorageResult<()> {
        // Get current state version
        let metadata = state.get_metadata().await;
        let current_version = metadata.current_version;

        // Find all migrations that need to be applied
        let pending: Vec<_> = self
            .migrations
            .iter()
            .filter(|(&version, _)| version > current_version)
            .collect();

        // Sort by version
        let mut pending = pending;
        pending.sort_by_key(|(&version, _)| version);

        // Apply each migration in a transaction
        for (version, migration) in pending {
            let transaction = client
                .transaction()
                .await
                .map_err(|e| crate::storage::StorageError::DatabaseError(e.to_string()))?;

            // Execute the migration
            transaction
                .execute(&migration.up_sql, &[])
                .await
                .map_err(|e| crate::storage::StorageError::DatabaseError(e.to_string()))?;

            // Commit the transaction
            transaction
                .commit()
                .await
                .map_err(|e| crate::storage::StorageError::DatabaseError(e.to_string()))?;

            // Record the migration in state
            let migration_record = Migration {
                version: *version,
                description: migration.description.clone(),
                timestamp: chrono::Utc::now(),
            };

            state.apply_migration(migration_record).await?;
        }

        Ok(())
    }
}

// Example migrations
pub fn get_base_migrations() -> Vec<SchemaMigration> {
    vec![
        SchemaMigration {
            version: 1,
            description: "Create blocks table".to_string(),
            up_sql: r#"
                CREATE TABLE IF NOT EXISTS blocks (
                    height BIGINT PRIMARY KEY,
                    hash TEXT NOT NULL,
                    previous_hash TEXT NOT NULL,
                    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                    transactions JSONB NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash);
                CREATE INDEX IF NOT EXISTS idx_blocks_previous_hash ON blocks(previous_hash);
            "#.to_string(),
            down_sql: "DROP TABLE IF EXISTS blocks;".to_string(),
        },
        SchemaMigration {
            version: 2,
            description: "Create relationships table".to_string(),
            up_sql: r#"
                CREATE TABLE IF NOT EXISTS relationships (
                    id SERIAL PRIMARY KEY,
                    member_one TEXT NOT NULL,
                    member_two TEXT NOT NULL,
                    relationship_type TEXT NOT NULL,
                    metadata JSONB NOT NULL DEFAULT '{}',
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(member_one, member_two)
                );
                
                CREATE INDEX IF NOT EXISTS idx_relationships_members 
                ON relationships(member_one, member_two);
            "#.to_string(),
            down_sql: "DROP TABLE IF EXISTS relationships;".to_string(),
        },
        SchemaMigration {
            version: 3,
            description: "Create reputation table".to_string(),
            up_sql: r#"
                CREATE TABLE IF NOT EXISTS reputation (
                    did TEXT PRIMARY KEY,
                    score BIGINT NOT NULL DEFAULT 0,
                    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    history JSONB NOT NULL DEFAULT '[]'
                );
                
                CREATE INDEX IF NOT EXISTS idx_reputation_score ON reputation(score DESC);
            "#.to_string(),
            down_sql: "DROP TABLE IF EXISTS reputation;".to_string(),
        }
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::postgres::PostgresStorage;
    use std::sync::Arc;
    use serial_test::serial;

    async fn setup_test_env() -> (StateManager, Client, MigrationManager) {
        let connection_str = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test".to_string());
            
        let backend = PostgresStorage::new(&connection_str)
            .await
            .expect("Failed to create PostgreSQL connection");
            
        let storage = Arc::new(crate::storage::StorageManager::new(Box::new(backend)));
        let state = StateManager::new(storage)
            .await
            .expect("Failed to create state manager");

        let (client, connection) = tokio_postgres::connect(
            &connection_str,
            tokio_postgres::NoTls,
        )
        .await
        .expect("Failed to connect to database");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        let mut migration_manager = MigrationManager::new();
        for migration in get_base_migrations() {
            migration_manager.register_migration(migration);
        }

        (state, client, migration_manager)
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_application() {
        let (state, client, migration_manager) = setup_test_env().await;

        // Apply migrations
        migration_manager
            .apply_migrations(&state, &client)
            .await
            .expect("Failed to apply migrations");

        // Verify tables were created
        let tables = client
            .query(
                "SELECT table_name FROM information_schema.tables 
                 WHERE table_schema = 'public'",
                &[],
            )
            .await
            .expect("Failed to query tables");

        let table_names: Vec<String> = tables.iter().map(|row| row.get(0)).collect();
        
        assert!(table_names.contains(&"blocks".to_string()));
        assert!(table_names.contains(&"relationships".to_string()));
        assert!(table_names.contains(&"reputation".to_string()));

        // Check state metadata was updated
        let metadata = state.get_metadata().await;
        assert_eq!(metadata.current_version, 3);
        assert_eq!(metadata.applied_migrations.len(), 3);

        // Verify migration order
        let versions: Vec<u32> = metadata
            .applied_migrations
            .iter()
            .map(|m| m.version)
            .collect();
        assert_eq!(versions, vec![1, 2, 3]);
    }

    #[tokio::test]
    #[serial]
    async fn test_idempotent_migrations() {
        let (state, client, migration_manager) = setup_test_env().await;

        // Apply migrations twice
        migration_manager
            .apply_migrations(&state, &client)
            .await
            .expect("Failed to apply migrations first time");
            
        migration_manager
            .apply_migrations(&state, &client)
            .await
            .expect("Failed to apply migrations second time");

        // Verify no duplicate migrations were recorded
        let metadata = state.get_metadata().await;
        assert_eq!(metadata.current_version, 3);
        assert_eq!(metadata.applied_migrations.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_error_handling() {
        let (state, client, mut migration_manager) = setup_test_env().await;

        // Add an invalid migration
        migration_manager.register_migration(SchemaMigration {
            version: 4,
            description: "Invalid migration".to_string(),
            up_sql: "INVALID SQL STATEMENT;".to_string(),
            down_sql: "".to_string(),
        });

        // Apply migrations and verify error handling
        let result = migration_manager.apply_migrations(&state, &client).await;
        assert!(result.is_err());

        // Verify state wasn't updated with failed migration
        let metadata = state.get_metadata().await;
        assert!(metadata.current_version < 4);
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_state_consistency() {
        let (state, client, migration_manager) = setup_test_env().await;

        // Apply initial migrations
        migration_manager
            .apply_migrations(&state, &client)
            .await
            .expect("Failed to apply migrations");

        // Verify database state matches recorded migrations
        let metadata = state.get_metadata().await;
        
        for migration in metadata.applied_migrations.iter() {
            // Try to create a table that should already exist from each migration
            let result = match migration.version {
                1 => client.execute("CREATE TABLE blocks (height BIGINT PRIMARY KEY);", &[]).await,
                2 => client.execute("CREATE TABLE relationships (id SERIAL PRIMARY KEY);", &[]).await,
                3 => client.execute("CREATE TABLE reputation (did TEXT PRIMARY KEY);", &[]).await,
                _ => panic!("Unexpected migration version"),
            };
            
            // Should fail because tables already exist
            assert!(result.is_err());
        }
    }
}