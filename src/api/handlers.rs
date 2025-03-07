use icn_types::IcnResult;
use crate::network::communication::NetworkLayer;
use crate::services::event_service::EventService;
use crate::services::event_service::DomainEvent;
use crate::storage::event_store::EventStore;
use crate::network::communication::NetworkEvent;
use warp::Reply;

pub async fn health_check() -> IcnResult<&'static str> {
    Ok("OK")
}

pub struct ApiHandlers {
    network: NetworkLayer,
    event_service: EventService,
    event_store: EventStore,
}

impl ApiHandlers {
    pub async fn new() -> Self {
        Self {
            network: NetworkLayer::new().await,
            event_service: EventService::new(),
            event_store: EventStore::new(),
        }
    }

    pub async fn handle_vote(&self, proposal_id: String, vote: bool) -> impl Reply {
        // Handle vote through all layers
        self.event_service.publish(DomainEvent::VoteCast {
            proposal_id: proposal_id.clone(),
            voter: "user".to_string(),
            vote,
        }).await;

        self.network.broadcast_event(NetworkEvent::VoteUpdate {
            proposal_id,
            votes: 1,
        }).await;

        warp::reply::json(&"Vote recorded")
    }

    // Add more handlers for other API endpoints
}
