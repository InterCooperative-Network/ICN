use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    VoteCast {
        proposal_id: String,
        voter: String,
        vote: bool,
    },
    ReputationChanged {
        member_id: String,
        delta: i32,
    },
    MembershipChanged {
        member_id: String,
        action: String,
    }
}

pub struct EventService {
    event_tx: mpsc::Sender<DomainEvent>,
}

impl EventService {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(100);
        // Spawn event processor
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Process domain events
                println!("Processing domain event: {:?}", event);
            }
        });
        
        Self {
            event_tx: tx,
        }
    }

    pub async fn publish(&self, event: DomainEvent) {
        if let Err(e) = self.event_tx.send(event).await {
            eprintln!("Failed to publish event: {}", e);
        }
    }
}
