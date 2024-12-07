// src/state/state_migrations.rs

use super::{Migration, StateManager};
use crate::storage::StorageResult;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

/// Trait for implementing state migrations
#[async_trait]
pub trait StateMigration: Send + Sync {
    /// Get the migration version
    fn version(&self) -> u32;
    
    /// Get a description of what the migration does
    fn description(&self) -> String;
    
    /// Apply the migration to the state
    async fn apply(&self, state: &StateManager) -> StorageResult<()>;
    
    /// Verify the migration was applied correctly
    async fn verify(&self, state: &StateManager) -> StorageResult<bool>;
}

/// Example state migration for updating reputation calculation
pub struct ReputationFormulaUpdate {
    version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OldReputationData {
    score: i64,
    history: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewReputationData {
    score: i64,
    weighted_score: f64,
    history: Vec<String>,
    categories: HashMap<String, i64>,
}

#[async_trait]
impl StateMigration for ReputationFormulaUpdate {
    fn version(&self) -> u32 {
        self.version
    }

    fn description(&self) -> String {
        "Update reputation data format to include weighted scores and categories".to_string()
    }

    async fn apply(&self, state: &StateManager) -> StorageResult<()> {
        // Begin batch operation
        state.begin_batch().await?;

        // Get all reputation keys (this would be implemented in the storage backend)
        let reputation_keys = state.list_keys_with_prefix("reputation:").await?;

        for key in reputation_keys {
            // Load old format
            let old_data: OldReputationData = state.storage.retrieve(&key).await?;
            
            // Convert to new format
            let new_data = NewReputationData {
                score: old_data.score,
                weighted_score: calculate_weighted_score(&old_data),
                history: old_data.history,
                categories: HashMap::new(),
            };
            
            // Store updated data
            state.batch_store(&key, &new_data).await?;
        }

        // Record the migration
        let migration = Migration {
            version: self.version(),
            description: self.description(),
            timestamp: chrono::Utc::now(),
        };
        
        state.apply_migration(migration).await?;
        
        // Commit all changes
        state.commit_batch().await?;
        
        Ok(())
    }

    async fn verify(&self, state: &StateManager) -> StorageResult<bool> {
        let reputation_keys = state.list_keys_with_prefix("reputation:").await?;
        
        for key in reputation_keys {
            // Attempt to load data in new format
            let result: StorageResult<NewReputationData> = state.storage.retrieve(&key).await;
            
            if result.is_err() {
                return Ok(false);
            }
            
            let data = result.unwrap();
            if data.weighted_score <= 0.0 {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

fn calculate_weighted_score(old_data: &OldReputationData) -> f64 {
    // Example weight calculation
    let base_score = old_data.score as f64;
    let history_weight = (old_data.history.len() as f64).sqrt();
    base_score * history_weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_reputation_migration() {
        let state = setup_test_state().await;
        
        // Create test data in old format
        let old_data = OldReputationData {
            score: 100,
            history: vec!["contribution".to_string(), "endorsement".to_string()],
        };
        
        state.storage
            .store("reputation:test_user", &old_data)
            .await
            .expect("Failed to store test data");

        // Apply migration
        let migration = ReputationFormulaUpdate { version: 1 };
        migration.apply(&state).await.expect("Failed to apply migration");

        // Verify migration
        assert!(migration.verify(&state).await.unwrap());

        // Check new data format
        let new_data: NewReputationData = state.storage
            .retrieve("reputation:test_user")
            .await
            .expect("Failed to retrieve migrated data");

        assert_eq!(new_data.score, 100);
        assert!(new_data.weighted_score > 0.0);
        assert_eq!(new_data.history.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_batch_rollback() {
        let state = setup_test_state().await;
        
        // Create test data
        let old_data = OldReputationData {
            score: 100,
            history: vec!["contribution".to_string()],
        };
        
        state.storage
            .store("reputation:test_user", &old_data)
            .await
            .expect("Failed to store test data");

        // Start migration but introduce error
        state.begin_batch().await.unwrap();
        
        // Store invalid data to trigger rollback
        state.batch_store("invalid_key", &"invalid_data").await.unwrap();
        
        // Attempt to commit (should fail)
        let result = state.commit_batch().await;
        assert!(result.is_err());

        // Verify original data is unchanged
        let original: OldReputationData = state.storage
            .retrieve("reputation:test_user")
            .await
            .expect("Failed to retrieve original data");
            
        assert_eq!(original.score, 100);
        assert_eq!(original.history.len(), 1);
    }
}