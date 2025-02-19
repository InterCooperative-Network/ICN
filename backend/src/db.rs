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
}
