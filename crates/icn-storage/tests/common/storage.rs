// crates/icn-storage/tests/storage.rs
use serial_test::serial;
use icn_types::{Block, Transaction, TransactionType};
use icn_storage::StorageResult;

mod common;
use common::{create_test_storage, cleanup_test_db};

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

    // Store and retrieve to populate cache
    storage.store_block(&block).await?;
    let _first = storage.get_block(&block.hash).await?;

    // Measure cached retrieval time
    let start = std::time::Instant::now();
    let _cached = storage.get_block(&block.hash).await?;
    let cache_duration = start.elapsed();

    // Measure database retrieval time with new connection
    let start = std::time::Instant::now();
    let (new_storage, _) = create_test_storage().await?;
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