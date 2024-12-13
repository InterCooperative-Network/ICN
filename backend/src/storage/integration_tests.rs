// src/storage/integration_tests.rs

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::storage::postgres::PostgresStorage;
    use serde::{Serialize, Deserialize};
    use serial_test::serial;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        value: f64,
    }

    async fn setup_test_storage() -> StorageManager {
        let connection_str = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/icn_test".to_string());
            
        let backend = PostgresStorage::new(&connection_str)
            .await
            .expect("Failed to create PostgreSQL connection");
            
        StorageManager::new(Box::new(backend))
    }

    #[tokio::test]
    #[serial]
    async fn test_complex_data_storage() {
        let storage = setup_test_storage().await;
        
        // Create test data
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
            value: 42.5,
        };
        
        // Store the data
        storage
            .store("test_complex", &test_data)
            .await
            .expect("Failed to store test data");
            
        // Retrieve and verify the data
        let retrieved: TestData = storage
            .retrieve("test_complex")
            .await
            .expect("Failed to retrieve test data");
            
        assert_eq!(retrieved, test_data);
        
        // Clean up
        storage
            .remove("test_complex")
            .await
            .expect("Failed to remove test data");
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_operations() {
        let storage = setup_test_storage().await;
        
        // Test data
        let items = vec![
            TestData { id: 1, name: "Item 1".to_string(), value: 10.0 },
            TestData { id: 2, name: "Item 2".to_string(), value: 20.0 },
            TestData { id: 3, name: "Item 3".to_string(), value: 30.0 },
        ];
        
        // Store multiple items
        for (i, item) in items.iter().enumerate() {
            let key = format!("multi_test_{}", i);
            storage.store(&key, item).await.expect("Failed to store item");
        }
        
        // Verify all items
        for (i, original_item) in items.iter().enumerate() {
            let key = format!("multi_test_{}", i);
            let retrieved: TestData = storage
                .retrieve(&key)
                .await
                .expect("Failed to retrieve item");
            assert_eq!(&retrieved, original_item);
        }
        
        // Test exists
        assert!(storage.has_key("multi_test_0").await.unwrap());
        assert!(!storage.has_key("nonexistent").await.unwrap());
        
        // Clean up
        for i in 0..items.len() {
            let key = format!("multi_test_{}", i);
            storage.remove(&key).await.expect("Failed to remove item");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_error_handling() {
        let storage = setup_test_storage().await;
        
        // Test retrieving non-existent key
        let result: Result<TestData, _> = storage.retrieve("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
        
        // Test removing non-existent key
        let result = storage.remove("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
        
        // Test storing and retrieving incompatible types
        storage.store("type_mismatch", &42i32).await.unwrap();
        let result: Result<String, _> = storage.retrieve("type_mismatch").await;
        assert!(matches!(result, Err(StorageError::SerializationError(_))));
        
        // Clean up
        storage.remove("type_mismatch").await.unwrap();
    }
}