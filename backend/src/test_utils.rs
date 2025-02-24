use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::database::Database;
use crate::identity::IdentityManager;
use crate::reputation::ReputationManager;
use crate::networking::p2p::P2PManager;

/// Test database configuration
pub struct TestDb {
    pub pool: PgPool,
}

impl TestDb {
    /// Creates a new test database connection
    pub async fn new() -> Self {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb_test".to_string());
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        Self { pool }
    }

    /// Cleans up test database tables
    pub async fn cleanup(&self) {
        // List of tables to truncate
        let tables = vec![
            "proposals",
            "votes",
            "identities",
            "reputation_scores",
            "storage",
            "federation_operations",
        ];

        for table in tables {
            let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool)
                .await;
        }
    }
}

/// Creates mock services for testing
pub struct TestServices {
    pub identity_manager: Arc<IdentityManager>,
    pub reputation_manager: Arc<ReputationManager>,
    pub p2p_manager: Arc<Mutex<P2PManager>>,
    pub database: Arc<Database>,
}

impl TestServices {
    /// Creates new test services with a shared database connection
    pub async fn new() -> Self {
        let test_db = TestDb::new().await;
        let database = Arc::new(Database { pool: test_db.pool });
        
        Self {
            identity_manager: Arc::new(IdentityManager::new(database.clone())),
            reputation_manager: Arc::new(ReputationManager::new(
                database.clone(),
                100, // max_cache_size
                0.1, // decay_rate
            )),
            p2p_manager: Arc::new(Mutex::new(P2PManager::new())),
            database,
        }
    }

    /// Cleans up all test data
    pub async fn cleanup(&self) {
        let test_db = TestDb {
            pool: self.database.pool.clone(),
        };
        test_db.cleanup().await;
    }
}

/// Test helper functions
pub mod helpers {
    use super::*;
    use crate::models::{Proposal, Vote};
    use chrono::{NaiveDateTime, Utc};

    /// Creates a test proposal
    pub fn create_test_proposal(id: i32) -> Proposal {
        Proposal {
            id,
            title: format!("Test Proposal {}", id),
            description: "Test Description".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: NaiveDateTime::from_timestamp(Utc::now().timestamp() + 3600, 0),
            created_at: NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0),
        }
    }

    /// Creates a test vote
    pub fn create_test_vote(proposal_id: i32, voter: &str, approve: bool) -> Vote {
        Vote {
            proposal_id,
            voter: voter.to_string(),
            approve,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let test_db = TestDb::new().await;
        assert!(sqlx::query("SELECT 1")
            .execute(&test_db.pool)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_services_creation() {
        let services = TestServices::new().await;
        assert!(services.database.pool.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let services = TestServices::new().await;
        services.cleanup().await;
        // Verify tables are empty
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM proposals")
            .fetch_one(&services.database.pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }
} 