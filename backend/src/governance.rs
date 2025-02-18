use crate::models::{Proposal, Vote};
use sqlx::PgPool;
use std::sync::Arc;
use log::{info, error};

pub struct GovernanceEngine {
    db_pool: Arc<PgPool>,
}

impl GovernanceEngine {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            db_pool: pool
        }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        match sqlx::query!(
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
        .fetch_one(&*self.db_pool)
        .await {
            Ok(row) => {
                info!("Proposal created with ID: {}", row.id);
                Ok(row.id)
            },
            Err(e) => {
                error!("Failed to create proposal: {}", e);
                Err(e)
            }
        }
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        match sqlx::query!(
            r#"
            INSERT INTO votes (proposal_id, voter, approve)
            VALUES ($1, $2, $3)
            "#,
            vote.proposal_id,
            vote.voter,
            vote.approve
        )
        .execute(&*self.db_pool)
        .await {
            Ok(_) => {
                info!("Vote recorded for proposal ID: {}", vote.proposal_id);
                Ok(())
            },
            Err(e) => {
                error!("Failed to record vote: {}", e);
                Err(e)
            }
        }
    }
}
