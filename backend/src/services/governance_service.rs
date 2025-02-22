use crate::database::queries::{create_proposal_in_db, record_vote_in_db};
use crate::database::models::{Proposal, Vote};
use std::sync::Arc;
use tokio::sync::Mutex;
use zk_snarks::verify_proof; // Import zk-SNARK verification function

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
        if let Some(proof) = &vote.zk_snark_proof {
            if !verify_proof(proof) {
                return Err(sqlx::Error::Protocol("Invalid zk-SNARK proof".to_string()));
            }
        }
        let db = self.db.lock().await;
        record_vote_in_db(&*db, &vote).await
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
}
