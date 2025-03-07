use sqlx::PgPool;
use std::sync::Arc;
use std::env;
use crate::models::{Proposal, Vote};

pub struct Database {
    pub db_pool: Arc<PgPool>,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self {
            db_pool: Arc::new(pool),
        }
    }

    pub async fn create_proposal(&self, proposal: &Proposal) -> Result<i64, sqlx::Error> {
        // This is a mock implementation for testing
        // In a real system, this would insert into a database
        Ok(1)
    }

    pub async fn record_vote(&self, vote: &Vote) -> Result<(), sqlx::Error> {
        // This is a mock implementation for testing
        Ok(())
    }

    pub async fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), sqlx::Error> {
        // This is a mock implementation for testing
        Ok(())
    }

    pub async fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), sqlx::Error> {
        // This is a mock implementation for testing
        Ok(())
    }
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb".to_string());
        
    let pool = PgPool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
        
    Ok(pool)
}
