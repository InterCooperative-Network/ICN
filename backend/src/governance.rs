use crate::models::{Proposal, Vote};
use sqlx::PgPool;
use std::sync::Arc;
use log::{info, error};
use crate::db::Database;
use crate::identity::IdentityManager;
use zk_snarks::verify_proof; // Import zk-SNARK verification function

pub struct GovernanceEngine {
    db: Arc<Database>,
    identity_manager: Arc<IdentityManager>,
}

impl GovernanceEngine {
    pub fn new(db: Arc<Database>, identity_manager: Arc<IdentityManager>) -> Self {
        Self {
            db,
            identity_manager,
        }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        self.db.create_proposal(&proposal).await.map_err(|e| {
            error!("Error creating proposal: {}", e);
            e
        })
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        if let Some(proof) = &vote.zk_snark_proof {
            if !verify_proof(proof) {
                return Err(sqlx::Error::Protocol("Invalid zk-SNARK proof".to_string()));
            }
        }
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

    pub async fn create_identity(&self, identity: &str) -> Result<(), String> {
        self.identity_manager.create_identity(identity).await
    }

    pub async fn get_identity(&self, identity: &str) -> Result<String, String> {
        self.identity_manager.get_identity(identity).await
    }

    pub async fn update_identity(&self, identity: &str, new_data: &str) -> Result<(), String> {
        self.identity_manager.update_identity(identity, new_data).await
    }

    pub async fn delete_identity(&self, identity: &str) -> Result<(), String> {
        self.identity_manager.delete_identity(identity).await
    }

    pub async fn submit_proposal(&self, title: &str, description: &str, created_by: &str, ends_at: &str) -> Result<i64, String> {
        let proposal = Proposal {
            id: 0, // Placeholder, will be set by the database
            title: title.to_string(),
            description: description.to_string(),
            created_by: created_by.to_string(),
            ends_at: chrono::NaiveDateTime::parse_from_str(ends_at, "%Y-%m-%d %H:%M:%S").map_err(|e| e.to_string())?,
            created_at: chrono::Utc::now().naive_utc(),
        };

        self.create_proposal(proposal).await.map_err(|e| e.to_string())
    }

    pub async fn vote(&self, proposal_id: i64, voter: &str, approve: bool) -> Result<(), String> {
        let vote = Vote {
            proposal_id,
            voter: voter.to_string(),
            approve,
        };

        self.record_vote(vote).await.map_err(|e| e.to_string())
    }
}
