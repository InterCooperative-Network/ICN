use async_trait::async_trait;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    VoteCast { proposal_id: String, voter: String, vote: bool },
    ReputationUpdated { member_id: String, new_score: i32 },
    ProposalCreated { id: String, title: String, description: String },
    StateChanged { entity_id: String, change_type: String },
}

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: DomainEvent);
}

pub struct EventService {
    tx: broadcast::Sender<DomainEvent>,
    handlers: Vec<Box<dyn EventHandler + Send + Sync>>,
}

impl EventService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            handlers: Vec::new(),
        }
    }

    pub async fn publish(&self, event: DomainEvent) {
        // Publish to event bus and notify handlers
        for handler in &self.handlers {
            handler.handle(event.clone()).await;
        }
        self.tx.send(event).unwrap();
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.tx.subscribe()
    }
}
