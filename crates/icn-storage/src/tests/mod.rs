// src/tests/mod.rs
use crate::StorageManager;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::types::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
}

#[tokio::test]
async fn test_storage_crud_operations() {
    // Initialize storage
    let storage = StorageManager::new(None).await.expect("Failed to create storage");
    
    // Create test data
    let test_data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    
    // Store data
    storage.store("test_key", &test_data).await.expect("Failed to store data");
    
    // Retrieve and verify data
    let retrieved: TestData = storage.retrieve("test_key").await.expect("Failed to retrieve data");
    assert_eq!(retrieved, test_data);
    
    // Update data
    let updated_data = TestData {
        name: "test_updated".to_string(),
        value: 43,
    };
    
    // Wait a bit to ensure timestamp changes
    sleep(Duration::from_secs(1)).await;
    
    storage.store("test_key", &updated_data).await.expect("Failed to update data");
    
    // Verify update
    let retrieved: TestData = storage.retrieve("test_key").await.expect("Failed to retrieve updated data");
    assert_eq!(retrieved, updated_data);

    // Delete data
    storage.delete("test_key").await.expect("Failed to delete data");
    
    // Verify deletion
    let result = storage.retrieve::<TestData>("test_key").await;
    assert!(result.is_err(), "Data should be deleted");
}

#[tokio::test]
async fn test_list_keys() {
    let storage = StorageManager::new(None).await.expect("Failed to create storage");
    
    // Store multiple items
    let test_data = TestData { name: "test1".to_string(), value: 1 };
    storage.store("key1", &test_data).await.expect("Failed to store data 1");
    
    let test_data = TestData { name: "test2".to_string(), value: 2 };
    storage.store("key2", &test_data).await.expect("Failed to store data 2");
    
    // List keys
    let keys = storage.list_keys().await.expect("Failed to list keys");
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
    
    // Clean up
    storage.delete("key1").await.expect("Failed to delete key1");
    storage.delete("key2").await.expect("Failed to delete key2");
}

#[tokio::test]
async fn test_metadata() {
    let storage = StorageManager::new(None).await.expect("Failed to create storage");
    
    // Store some data
    let test_data = TestData { name: "test".to_string(), value: 42 };
    let key = "metadata_test_key";
    
    storage.store(key, &test_data).await.expect("Failed to store data");
    
    // Get metadata about stored data using raw query
    let client = storage.get_client().await.expect("Failed to get client");
    let row = client.query_one(
        "SELECT created_at::timestamp with time zone, updated_at::timestamp with time zone FROM key_value WHERE key = $1",
        &[&key]
    ).await.expect("Failed to get metadata");
    
    let created_at: chrono::DateTime<chrono::Utc> = row.get(0);
    let updated_at: chrono::DateTime<chrono::Utc> = row.get(1);
    
    // Initially, created_at and updated_at should be very close
    assert!((updated_at - created_at).num_milliseconds() < 1000);
    
    // Wait a second and update
    sleep(Duration::from_secs(1)).await;
    
    let updated_data = TestData { name: "test_updated".to_string(), value: 43 };
    storage.store(key, &updated_data).await.expect("Failed to update data");
    
    // Check timestamps again
    let row = client.query_one(
        "SELECT created_at::timestamp with time zone, updated_at::timestamp with time zone FROM key_value WHERE key = $1",
        &[&key]
    ).await.expect("Failed to get updated metadata");
    
    let created_at_after: chrono::DateTime<chrono::Utc> = row.get(0);
    let updated_at_after: chrono::DateTime<chrono::Utc> = row.get(1);
    
    // created_at should not change
    assert_eq!(created_at, created_at_after);
    // updated_at should be newer
    assert!(updated_at_after > updated_at);
    
    // Clean up
    storage.delete(key).await.expect("Failed to delete test data");
}