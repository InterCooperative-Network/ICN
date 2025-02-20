use crate::database::queries::{create_proposal_in_db, record_vote_in_db};
use crate::database::models::{Proposal, Vote};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct GovernanceService {
    db: Arc<Mutex<dyn Database>>,
}

impl GovernanceService {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        Self { db }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        let db = self.db.lock().await;
        create_proposal_in_db(&*db, &proposal).await
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        let db = self.db.lock().await;
        record_vote_in_db(&*db, &vote).await
    }
}
