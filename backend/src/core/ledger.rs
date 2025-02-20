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
}
