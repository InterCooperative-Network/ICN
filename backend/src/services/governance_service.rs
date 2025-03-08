use crate::database::queries::{create_proposal_in_db, record_vote_in_db};
use crate::database::models::{Proposal, Vote};
use std::sync::Arc;
use tokio::sync::Mutex;
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use crate::services::identity_service::IdentityService; // Import IdentityService
use icn_crypto::KeyPair; // Import KeyPair for signature verification
use crate::reputation::ReputationManager; // Import ReputationManager
use futures::future::join_all; // Import join_all for concurrency

pub struct GovernanceService {
    db: Arc<Mutex<dyn Database>>,
    identity_service: Arc<dyn IdentityService>, // Add IdentityService to GovernanceService
    reputation_manager: Arc<ReputationManager>, // Add ReputationManager to GovernanceService
}

impl GovernanceService {
    pub fn new(db: Arc<Mutex<dyn Database>>, identity_service: Arc<dyn IdentityService>, reputation_manager: Arc<ReputationManager>) -> Self {
        Self { db, identity_service, reputation_manager }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> Result<i64, sqlx::Error> {
        // Verify DID using IdentityService
        if !self.identity_service.verify_did(&proposal.created_by).await {
            return Err(sqlx::Error::Protocol("Invalid DID".to_string()));
        }

        // Validate verifiable credential
        if !self.identity_service.verify_credential(&proposal.verifiable_credential).await {
            return Err(sqlx::Error::Protocol("Invalid verifiable credential".to_string()));
        }

        // Check reputation using ReputationManager
        if !self.verify_member_eligibility(&proposal.created_by).await {
            return Err(sqlx::Error::Protocol("Insufficient reputation".to_string()));
        }

        let db = self.db.lock().await;
        create_proposal_in_db(&*db, &proposal).await
    }

    pub async fn record_vote(&self, vote: Vote) -> Result<(), sqlx::Error> {
        // Validate verifiable credential
        if !self.identity_service.verify_credential(&vote.verifiable_credential).await {
            return Err(sqlx::Error::Protocol("Invalid verifiable credential".to_string()));
        }

        // Verify signature using icn-crypto
        if !self.verify_signature(&vote.voter, &vote.signature, &vote.proposal_id).await {
            return Err(sqlx::Error::Protocol("Invalid signature".to_string()));
        }

        if let Some(proof) = &vote.zk_snark_proof {
            if !verify_proof(proof) {
                return Err(sqlx::Error::Protocol("Invalid zk-SNARK proof".to_string()));
            }
        }
        let db = self.db.lock().await;
        record_vote_in_db(&*db, &vote).await
    }

    async fn verify_signature(&self, did: &str, signature: &str, message: &str) -> bool {
        // Retrieve public key from IdentityService
        if let Some(public_key) = self.identity_service.get_public_key(did).await {
            let key_pair = KeyPair {
                public_key,
                private_key: vec![], // Not needed for verification
                algorithm: icn_crypto::Algorithm::Secp256k1, // Assuming Secp256k1 for this example
            };
            return key_pair.verify(message.as_bytes(), signature.as_bytes());
        }
        false
    }

    async fn verify_member_eligibility(&self, did: &str) -> bool {
        self.reputation_manager.get_reputation(did, "governance") >= 50 // Example threshold
    }

    // New handler for creating proposals
    pub async fn handle_create_proposal(
        &self,
        proposal: Proposal,
        notification_manager: crate::notification::NotificationManager,
        websocket_clients: crate::websocket::WebSocketClients,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match self.create_proposal(proposal.clone()).await {
            Ok(id) => {
                let subject = format!("New Proposal Created: {}", proposal.title);
                let body = format!("A new proposal has been created by {}. Description: {}",
                    proposal.created_by, proposal.description);
                notification_manager.send_notification(&subject, &body).await;
                let message = warp::ws::Message::text(serde_json::to_string(&proposal).unwrap());
                crate::websocket::broadcast_message(&message, websocket_clients).await;
                Ok(warp::reply::json(&id))
            },
            Err(e) => Err(warp::reject::custom(e)),
        }
    }

    // New handler for voting on proposals
    pub async fn handle_vote_on_proposal(
        &self,
        vote: Vote,
        notification_manager: crate::notification::NotificationManager,
        websocket_clients: crate::websocket::WebSocketClients,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match self.record_vote(vote.clone()).await {
            Ok(_) => {
                let subject = format!("New Vote on Proposal: {}", vote.proposal_id);
                let body = format!("A new vote has been cast by {}. Approve: {}", vote.voter, vote.approve);
                notification_manager.send_notification(&subject, &body).await;
                let message = warp::ws::Message::text(serde_json::to_string(&vote).unwrap());
                crate::websocket::broadcast_message(&message, websocket_clients).await;
                Ok(warp::reply::json(&"Vote recorded"))
            },
            Err(e) => Err(warp::reject::custom(e)),
        }
    }

    pub async fn submit_proposal(&self, title: &str, description: &str, created_by: &str, ends_at: &str) -> Result<i64, String> {
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

    pub async fn vote(&self, _proposal_id: i64, voter: &str, approve: bool) -> Result<(), String> {
        let vote = Vote {
            proposal_id: _proposal_id,
            voter: voter.to_string(),
            approve,
        };

        self.record_vote(vote).await.map_err(|e| e.to_string())
    }

    pub async fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), String> {
        // Placeholder logic for handling Sybil resistance
        Ok(())
    }

    pub async fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), String> {
        let db = self.db.lock().await;
        db.apply_reputation_decay(did, decay_rate).await.map_err(|e| e.to_string())
    }

    pub async fn handle_delegated_governance(&self, federation_id: &str, representative_id: &str) -> Result<(), String> {
        // Placeholder logic for handling delegated governance
        Ok(())
    }

    pub async fn verify_signatures_concurrently(&self, dids: Vec<&str>, signatures: Vec<&str>, messages: Vec<&str>) -> Result<Vec<bool>, String> {
        let verification_futures: Vec<_> = dids.iter().zip(signatures.iter()).zip(messages.iter())
            .map(|((&did, &signature), &message)| {
                self.verify_signature(did, signature, message)
            })
            .collect();

        let results = join_all(verification_futures).await;
        Ok(results)
    }
}
