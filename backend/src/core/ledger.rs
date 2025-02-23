use crate::database::models::{Proposal, Vote};
use crate::database::queries::{create_proposal, record_vote};
use sqlx::PgPool;
use std::sync::Arc;

pub struct Ledger {
    db_pool: Arc<PgPool>,
}

impl Ledger {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        create_proposal(&self.db_pool, &proposal).await
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        record_vote(&self.db_pool, &vote).await
    }

    pub async fn record_federation_transaction(&self, federation_id: i64, transaction: &str) -> Result<(), sqlx::Error> {
        // Placeholder logic for recording federation transaction
        Ok(())
    }

    pub async fn record_proposal_transaction(&self, proposal_id: i64, transaction: &str) -> Result<(), sqlx::Error> {
        // Placeholder logic for recording proposal transaction
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{Proposal, Vote};
    use sqlx::{PgPool, Executor};
    use std::sync::Arc;
    use chrono::NaiveDateTime;

    async fn setup_test_db() -> Arc<PgPool> {
        let pool = PgPool::connect("postgres://icnuser:icnpass@localhost/icndb").await.unwrap();
        pool.execute("TRUNCATE TABLE proposals, votes").await.unwrap();
        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_create_proposal() {
        let db_pool = setup_test_db().await;
        let ledger = Ledger::new(db_pool.clone());

        let proposal = Proposal {
            id: 1,
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: NaiveDateTime::from_timestamp(1_614_000_000, 0),
            created_at: NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let result = ledger.create_proposal(proposal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_vote() {
        let db_pool = setup_test_db().await;
        let ledger = Ledger::new(db_pool.clone());

        let vote = Vote {
            proposal_id: 1,
            voter: "did:icn:test".to_string(),
            approve: true,
        };

        let result = ledger.record_vote(vote).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_federation_transaction() {
        let db_pool = setup_test_db().await;
        let ledger = Ledger::new(db_pool.clone());

        let result = ledger.record_federation_transaction(1, "Test Federation Transaction").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_proposal_transaction() {
        let db_pool = setup_test_db().await;
        let ledger = Ledger::new(db_pool.clone());

        let result = ledger.record_proposal_transaction(1, "Test Proposal Transaction").await;
        assert!(result.is_ok());
    }
}
