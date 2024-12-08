// crates/icn-storage/tests/common/mod.rs
use sqlx::postgres::PgPoolOptions;
use icn_storage::{StorageManager, StorageConfig, StorageResult, StorageError};

/// Helper function to create a test database and run migrations
pub async fn setup_test_db() -> StorageResult<String> {
    let db_name = format!("icn_test_{}", std::process::id());
    let admin_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost".to_string());

    // Connect to postgres to create test database
    let pool = PgPoolOptions::new()
        .connect(&admin_url)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    // Create test database
    sqlx::query(&format!(r#"CREATE DATABASE "{}""#, db_name))
        .execute(&pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    let test_url = format!("{}/{}", admin_url, db_name);

    // Run migrations on test database
    let pool = PgPoolOptions::new()
        .connect(&test_url)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    Ok(test_url)
}

/// Helper function to cleanup test database
pub async fn cleanup_test_db(db_url: &str) -> StorageResult<()> {
    let admin_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost".to_string());

    let pool = PgPoolOptions::new()
        .connect(&admin_url)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    let db_name = db_url.split('/').last().unwrap();
    
    // Terminate all connections to the test database
    sqlx::query(&format!(
        r#"
        SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{}'
        AND pid <> pg_backend_pid()
        "#,
        db_name
    ))
    .execute(&pool)
    .await
    .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    // Drop the test database
    sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE)"#, db_name))
        .execute(&pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Creates a test storage manager instance
pub async fn create_test_storage() -> StorageResult<(StorageManager, String)> {
    let db_url = setup_test_db().await?;
    
    let config = StorageConfig {
        database_url: db_url.clone(),
        max_pool_size: 2,
        timeout_seconds: 5,
    };

    let storage = StorageManager::new(config).await?;
    Ok((storage, db_url))
}