use crate::models::{Proposal, Vote};
use sqlx::PgPool;
use std::sync::Arc;
use log::{info, error};
use crate::db::Database;
use crate::identity::IdentityManager;
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use futures::future::join_all; // Import join_all for concurrency

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
        // Verify DID using IdentityManager
        if !self.identity_manager.verify_did(&proposal.created_by).await {
            return Err(sqlx::Error::Protocol("Invalid DID".to_string()));
        }

        // Validate verifiable credential
        if !self.identity_manager.verify_credential(&proposal.verifiable_credential).await {
            return Err(sqlx::Error::Protocol("Invalid verifiable credential".to_string()));
        }

        self.db.create_proposal(&proposal).await.map_err(|e| {
            error!("Error creating proposal: {}", e);
            e
        })
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        // Validate verifiable credential
        if !self.identity_manager.verify_credential(&vote.verifiable_credential).await {
            return Err(sqlx::Error::Protocol("Invalid verifiable credential".to_string()));
        }

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
        // Verify DID using IdentityManager
        if !self.identity_manager.verify_did(created_by).await {
            return Err("Invalid DID".to_string());
        }

        let proposal = Proposal {
            id: 0, // Placeholder, will be set by the database
            title: title.to_string(),
            description: description.to_string(),
            created_by: created_by.to_string(),
            ends_at: chrono::NaiveDateTime::parse_from_str(ends_at, "%Y-%m-%d %H:%M:%S").map_err(|e| e.to_string())?,
            created_at: chrono::Utc::now().naive_utc(),
            did: created_by.to_string(), // Add did field for DID-based access control
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

    pub async fn get_proposal_status(&self, proposal_id: &str) -> Result<String, sqlx::Error> {
        let status = sqlx::query!(
            r#"
            SELECT status FROM proposals WHERE id = $1
            "#,
            proposal_id
        )
        .fetch_one(&*self.db.db_pool)
        .await
        .map_err(|e| {
            error!("Error getting proposal status: {}", e);
            e
        })?;
        Ok(status.status)
    }

    pub async fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), sqlx::Error> {
        self.db.apply_reputation_decay(did, decay_rate).await.map_err(|e| {
            error!("Error applying reputation decay: {}", e);
            e
        })
    }

    pub async fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), sqlx::Error> {
        self.db.handle_sybil_resistance(did, reputation_score).await.map_err(|e| {
            error!("Error handling sybil resistance: {}", e);
            e
        })
    }

    pub async fn handle_delegated_governance(&self, federation_id: &str, representative_id: &str) -> Result<(), String> {
        // Placeholder logic for handling delegated governance
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Proposal, Vote};
    use crate::db::Database;
    use crate::identity::IdentityManager;
    use sqlx::{PgPool, Executor};
    use std::sync::Arc;
    use chrono::NaiveDateTime;

    async fn setup_test_db() -> Arc<Database> {
        let pool = PgPool::connect("postgres://icnuser:icnpass@localhost/icndb").await.unwrap();
        pool.execute("TRUNCATE TABLE proposals, votes").await.unwrap();
        Arc::new(Database::new(pool))
    }

    #[tokio::test]
    async fn test_create_proposal() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let proposal = Proposal {
            id: 1,
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: NaiveDateTime::from_timestamp(1_614_000_000, 0),
            created_at: NaiveDateTime::from_timestamp(1_614_000_000, 0),
        };

        let result = governance_engine.create_proposal(proposal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_vote() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let vote = Vote {
            proposal_id: 1,
            voter: "did:icn:test".to_string(),
            approve: true,
        };

        let result = governance_engine.record_vote(vote).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_proposals() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let proposals = governance_engine.list_proposals().await;
        assert!(proposals.is_ok());
    }

    #[tokio::test]
    async fn test_get_proposal_status() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let status = governance_engine.get_proposal_status("1").await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_apply_reputation_decay() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let result = governance_engine.apply_reputation_decay("did:icn:test", 0.1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_sybil_resistance() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let result = governance_engine.handle_sybil_resistance("did:icn:test", 50).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_delegated_governance() {
        let db = setup_test_db().await;
        let identity_manager = Arc::new(IdentityManager::new(db.clone()));
        let governance_engine = GovernanceEngine::new(db.clone(), identity_manager.clone());

        let result = governance_engine.handle_delegated_governance("federation_id", "representative_id").await;
        assert!(result.is_ok());
    }
}
