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
            
        // Create indexes on the resources table
        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_resource_owner ON resources(owner);")
            .execute(&pool)
            .await?;
        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_resource_type ON resources(resource_type);")
            .execute(&pool)
            .await?;
        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_resource_created_at ON resources(created_at);")
            .execute(&pool)
            .await?;
        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_resource_updated_at ON resources(updated_at);")
            .execute(&pool)
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
}
