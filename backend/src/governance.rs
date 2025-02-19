use crate::models::{Proposal, Vote};
use sqlx::PgPool;
use std::sync::Arc;
use log::{info, error};
use crate::db::Database;

pub struct GovernanceEngine {
    db: Arc<Database>,
}

impl GovernanceEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db
        }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        self.db.create_proposal(&proposal).await.map_err(|e| {
            error!("Error creating proposal: {}", e);
            e
        })
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        self.db.record_vote(&vote).await.map_err(|e| {
            error!("Error recording vote: {}", e);
            e
        })
    }

    pub async fn list_proposals(&self) -> Result<Vec<Proposal>, sqlx::Error> {
        let proposals = sqlx::query_as!(
            Proposal,
            r#"
            SELECT id, title, description, created_by, ends_at, created_at
            FROM proposals
            "#
        )
        .fetch_all(&*self.db.db_pool)
        .await
        .map_err(|e| {
            error!("Error listing proposals: {}", e);
            e
        })?;
        Ok(proposals)
    }
}
