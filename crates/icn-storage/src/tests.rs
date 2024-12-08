// src/tests.rs
use super::*;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use icn_types::{Block, Transaction, TransactionType};

/// Helper function to create a test database and run migrations
async fn setup_test_db() -> StorageResult<String> {
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
async fn cleanup_test_db(db_url: &str) -> StorageResult<()> {
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
async fn create_test_storage() -> StorageResult<(StorageManager, String)> {
    let db_url = setup_test_db().await?;
    
    let config = StorageConfig {
        database_url: db_url.clone(),
        max_pool_size: 2,
        timeout_seconds: 5,
    };

    let storage = StorageManager::new(config).await?;
    Ok((storage, db_url))
}

#[tokio::test]
#[serial]
async fn test_block_storage_and_retrieval() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    // Create test block
    let block = Block {
        height: 1,
        hash: "test_hash".to_string(),
        previous_hash: "prev_hash".to_string(),
        timestamp: 12345,
        proposer: "test_proposer".to_string(),
        data: serde_json::json!({"test": "data"}),
    };

    // Store block
    storage.store_block(&block).await?;

    // Retrieve block
    let retrieved = storage.get_block(&block.hash).await?;
    assert_eq!(block.hash, retrieved.hash);
    assert_eq!(block.height, retrieved.height);
    assert_eq!(block.previous_hash, retrieved.previous_hash);
    assert_eq!(block.timestamp, retrieved.timestamp);
    assert_eq!(block.proposer, retrieved.proposer);

    cleanup_test_db(&db_url).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_block_cache() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    let block = Block {
        height: 1,
        hash: "cache_test_hash".to_string(),
        previous_hash: "prev_hash".to_string(),
        timestamp: 12345,
        proposer: "test_proposer".to_string(),
        data: serde_json::json!({"test": "data"}),
    };

    // Store block
    storage.store_block(&block).await?;

    // First retrieval (from database)
    let _retrieved = storage.get_block(&block.hash).await?;

    // Second retrieval (should be from cache)
    let start = std::time::Instant::now();
    let _cached = storage.get_block(&block.hash).await?;
    let cache_duration = start.elapsed();

    // Third retrieval with new connection (from database)
    let start = std::time::Instant::now();
    let config = StorageConfig {
        database_url: db_url.clone(),
        max_pool_size: 2,
        timeout_seconds: 5,
    };
    let new_storage = StorageManager::new(config).await?;
    let _db = new_storage.get_block(&block.hash).await?;
    let db_duration = start.elapsed();

    // Cache should be significantly faster
    assert!(cache_duration < db_duration);

    cleanup_test_db(&db_url).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_transaction_storage_and_retrieval() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    let tx = Transaction {
        hash: "tx_hash".to_string(),
        block_height: 1,
        sender: "test_sender".to_string(),
        transaction_type: TransactionType::Transfer,
        data: serde_json::json!({"amount": 100}),
        timestamp: 12345,
    };

    // Store transaction
    storage.store_transactions(&[tx.clone()]).await?;

    // Retrieve transactions
    let transactions = storage.get_transactions_by_sender(&tx.sender).await?;
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].hash, tx.hash);
    assert_eq!(transactions[0].sender, tx.sender);

    cleanup_test_db(&db_url).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_relationship_management() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    let source_did = "did:icn:alice";
    let target_did = "did:icn:bob";
    let rel_type = "COOPERATES_WITH";
    let metadata = Some(serde_json::json!({
        "trust_level": "high",
        "start_date": "2024-01-01"
    }));

    // Create relationship
    storage.upsert_relationship(source_did, target_did, rel_type, metadata.clone()).await?;

    // Get relationships for source
    let relationships = storage.get_relationships_for_did(source_did).await?;
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].source_did, source_did);
    assert_eq!(relationships[0].target_did, target_did);
    assert_eq!(relationships[0].relationship_type, rel_type);
    assert_eq!(relationships[0].metadata, metadata);

    // Update relationship
    let new_metadata = Some(serde_json::json!({
        "trust_level": "medium",
        "start_date": "2024-01-01"
    }));
    storage.upsert_relationship(source_did, target_did, rel_type, new_metadata.clone()).await?;

    // Verify update
    let updated = storage.get_relationships_for_did(source_did).await?;
    assert_eq!(updated[0].metadata, new_metadata);

    cleanup_test_db(&db_url).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_latest_block_height() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    // Initially should be 0
    let height = storage.get_latest_block_height().await?;
    assert_eq!(height, 0);

    // Add some blocks
    for i in 1..=3 {
        let block = Block {
            height: i,
            hash: format!("test_hash_{}", i),
            previous_hash: format!("prev_hash_{}", i-1),
            timestamp: 12345,
            proposer: "test_proposer".to_string(),
            data: serde_json::json!({"test": "data"}),
        };
        storage.store_block(&block).await?;
    }

    // Should return highest height
    let height = storage.get_latest_block_height().await?;
    assert_eq!(height, 3);

    cleanup_test_db(&db_url).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_cleanup_old_data() -> StorageResult<()> {
    let (storage, db_url) = create_test_storage().await?;

    // Add transactions with different timestamps
    let transactions = vec![
        Transaction {
            hash: "tx1".to_string(),
            block_height: 1,
            sender: "sender".to_string(),
            transaction_type: TransactionType::Transfer,
            data: serde_json::json!({}),
            timestamp: 1000,
        },
        Transaction {
            hash: "tx2".to_string(),
            block_height: 1,
            sender: "sender".to_string(),
            transaction_type: TransactionType::Transfer,
            data: serde_json::json!({}),
            timestamp: 2000,
        },
    ];

    storage.store_transactions(&transactions).await?;

    // Cleanup old transactions
    storage.cleanup_old_data(1500).await?;

    // Should only have newer transaction
    let remaining = storage.get_transactions_by_sender("sender").await?;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].hash, "tx2");

    cleanup_test_db(&db_url).await?;
    Ok(())
}