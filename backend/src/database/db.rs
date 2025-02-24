use sqlx::PgPool;
use crate::models::{Proposal, Vote};
use std::env;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Load from environment variable with fallback
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb".to_string());
            
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
            
        Ok(Self { pool })
    }

    pub async fn create_proposal(&self, proposal: &Proposal) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO proposals (title, description, created_by, ends_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            proposal.title,
            proposal.description,
            proposal.created_by,
            proposal.ends_at
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.id)
        .map_err(|e| {
            eprintln!("Error creating proposal: {}", e);
            e
        })
    }

    pub async fn record_vote(&self, vote: &Vote) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO votes (proposal_id, voter, approve)
            VALUES ($1, $2, $3)
            "#,
            vote.proposal_id,
            vote.voter,
            vote.approve
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("Error recording vote: {}", e);
            e
        })?;
        
        Ok(())
    }

    pub async fn store_identity(&self, identity: &str, data: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO identities (identity, data)
            VALUES ($1, $2)
            ON CONFLICT (identity) DO UPDATE SET data = $2
            "#,
            identity,
            data
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("Error storing identity: {}", e);
            e
        })?;
        
        Ok(())
    }

    pub async fn retrieve_identity(&self, identity: &str) -> Result<String, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT data FROM identities WHERE identity = $1
            "#,
            identity
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving identity: {}", e);
            e
        })?;
        
        Ok(result.data)
    }

    pub async fn execute_query(&self, query: &str) -> Result<sqlx::Row, sqlx::Error> {
        sqlx::query(query)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn run_migration(&self, migration: &str) -> Result<(), sqlx::Error> {
        sqlx::query(migration)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as f64;
        let contributions = sqlx::query_as!(
            Contribution,
            r#"
            SELECT score, timestamp FROM contributions WHERE did = $1
            "#,
            did
        )
        .fetch_all(&self.pool)
        .await?;

        for contribution in contributions {
            let age = now - contribution.timestamp;
            let decayed_score = (contribution.score as f64 * (-decay_rate * age).exp()) as i64;
            sqlx::query!(
                r#"
                UPDATE contributions SET score = $1 WHERE did = $2 AND timestamp = $3
                "#,
                decayed_score,
                did,
                contribution.timestamp
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), sqlx::Error> {
        // Placeholder logic for handling Sybil resistance
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;

    async fn setup_test_db() -> PgPool {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_proposal() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let proposal = Proposal {
            id: 0,
            title: "Test Proposal".to_string(),
            description: "Test Description".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
            created_at: chrono::NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let result = db.create_proposal(&proposal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_vote() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let vote = Vote {
            proposal_id: 1,
            voter: "did:icn:test".to_string(),
            approve: true,
        };

        let result = db.record_vote(&vote).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_identity() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let identity = "did:icn:test";
        let data = "test data";

        let store_result = db.store_identity(identity, data).await;
        assert!(store_result.is_ok());

        let retrieve_result = db.retrieve_identity(identity).await;
        assert!(retrieve_result.is_ok());
        assert_eq!(retrieve_result.unwrap(), data);
    }

    #[tokio::test]
    async fn test_execute_query() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let query = "SELECT 1";
        let result = db.execute_query(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_migration() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let migration = "CREATE TABLE test_table (id SERIAL PRIMARY KEY)";
        let result = db.run_migration(migration).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_reputation_decay() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let did = "did:icn:test";
        let decay_rate = 0.05;

        let result = db.apply_reputation_decay(did, decay_rate).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_sybil_resistance() {
        let pool = setup_test_db().await;
        let db = Database { pool };

        let did = "did:icn:test";
        let reputation_score = 50;

        let result = db.handle_sybil_resistance(did, reputation_score).await;
        assert!(result.is_ok());
    }
}
