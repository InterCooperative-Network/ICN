use crate::network::communication::NetworkLayer;
use crate::services::event_service::EventService;
use crate::services::event_service::DomainEvent;
use crate::storage::event_store::EventStore;
use crate::network::communication::NetworkEvent;
use warp::Reply;

pub async fn health_check() -> &'static str {
    "OK"
}

pub struct ApiHandlers {
    network: NetworkLayer,
    event_service: EventService,
    event_store: EventStore,
}

impl ApiHandlers {
    pub async fn new() -> Self {
        Self {
            network: NetworkLayer::new(Some("127.0.0.1:8080")).await.expect("Failed to create NetworkLayer"),
            event_service: EventService::new(),
            event_store: EventStore::new(),
        }
    }

    pub async fn handle_vote(&self, proposal_id: String, vote: bool) -> impl Reply {
        // Handle vote through all layers
        let event = DomainEvent::VoteCast {
            proposal_id: proposal_id.clone(),
            voter: "user".to_string(),
            vote,
        };

        // Publish event
        self.event_service.publish(event.clone()).await;

        // Store the event
        if let Err(e) = self.event_store.append(&proposal_id, event) {
            eprintln!("Failed to store vote event: {}", e);
            return warp::reply::json(&format!("Failed to store vote: {}", e));
        }

        // Broadcast with error handling
        if let Err(e) = self.network.broadcast_event(NetworkEvent::VoteUpdate {
            proposal_id,
            votes: 1,
        }).await {
            eprintln!("Failed to broadcast vote update: {}", e);
            return warp::reply::json(&format!("Vote recorded but broadcast failed: {}", e));
        }

        warp::reply::json(&"Vote recorded and broadcast successfully")
    }

    // Add more handlers for other API endpoints
}
