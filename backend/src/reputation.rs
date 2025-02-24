use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::task;
use tokio::time::{self, Duration};

pub struct ReputationManager {
    reputations: Mutex<HashMap<String, i64>>,
    event_sender: Sender<ReputationEvent>,
}

impl ReputationManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::channel(100);
        let manager = ReputationManager {
            reputations: Mutex::new(HashMap::new()),
            event_sender,
        };
        manager.start_event_listener(event_receiver);
        manager
    }

    pub fn get_reputation(&self, did: &str) -> i64 {
        let reputations = self.reputations.lock().unwrap();
        *reputations.get(did).unwrap_or(&0)
    }

    pub fn adjust_reputation(&self, did: &str, adjustment: i64) {
        let mut reputations = self.reputations.lock().unwrap();
        let entry = reputations.entry(did.to_string()).or_insert(0);
        *entry += adjustment;
    }

    pub fn apply_decay(&self, decay_rate: f64) {
        let mut reputations = self.reputations.lock().unwrap();
        for value in reputations.values_mut() {
            *value = (*value as f64 * (1.0 - decay_rate)).round() as i64;
        }
    }

    pub fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
        let contributions = sqlx::query_as!(
            Contribution,
            r#"
            SELECT score, timestamp FROM contributions WHERE did = $1
            "#,
            did
        )
        .fetch_all(&*self.db.pool)
        .await?;

        for contribution in contributions {
            let age = now - contribution.timestamp;
            let decayed_score = (contribution.score as f64 * (-decay_rate * age).exp()) as i64;
            sqlx::query!(
                r#"
                UPDATE contributions SET score = $1 WHERE did = $2 AND timestamp = $3
                "#,
                decayed_score,
                did,
                contribution.timestamp
            )
            .execute(&*self.db.pool)
            .await?;
        }

        Ok(())
    }

    pub fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), String> {
        // Placeholder logic for handling Sybil resistance
        Ok(())
    }

    pub fn emit_event(&self, event: ReputationEvent) {
        let sender = self.event_sender.clone();
        task::spawn(async move {
            sender.send(event).await.unwrap();
        });
    }

    fn start_event_listener(&self, mut event_receiver: Receiver<ReputationEvent>) {
        task::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    ReputationEvent::ReputationAdjusted { did, adjustment } => {
                        // Handle reputation adjustment event
                    }
                    ReputationEvent::ReputationDecayApplied { did, decay_rate } => {
                        // Handle reputation decay event
                    }
                }
            }
        });
    }

    pub async fn batch_reputation_updates(&self, events: Vec<ReputationEvent>) -> Result<(), String> {
        for event in events {
            self.emit_event(event);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_reputation() {
        let manager = ReputationManager::new();
        assert_eq!(manager.get_reputation("did:example:123"), 0);
    }

    #[test]
    fn test_adjust_reputation() {
        let manager = ReputationManager::new();
        manager.adjust_reputation("did:example:123", 10);
        assert_eq!(manager.get_reputation("did:example:123"), 10);
    }

    #[test]
    fn test_apply_decay() {
        let manager = ReputationManager::new();
        manager.adjust_reputation("did:example:123", 100);
        manager.apply_decay(0.1);
        assert_eq!(manager.get_reputation("did:example:123"), 90);
    }

    #[test]
    fn test_apply_reputation_decay() {
        let manager = ReputationManager::new();
        manager.adjust_reputation("did:example:123", 100);
        manager.apply_reputation_decay("did:example:123", 0.1).unwrap();
        assert_eq!(manager.get_reputation("did:example:123"), 90);
    }

    #[test]
    fn test_handle_sybil_resistance() {
        let manager = ReputationManager::new();
        manager.handle_sybil_resistance("did:example:123", 50).unwrap();
        // Add assertions based on the expected behavior of handle_sybil_resistance
    }

    #[test]
    fn test_emit_event() {
        let manager = ReputationManager::new();
        manager.emit_event(ReputationEvent::ReputationAdjusted {
            did: "did:example:123".to_string(),
            adjustment: 10,
        });
        // Add assertions based on the expected behavior of emit_event
    }

    #[tokio::test]
    async fn test_batch_reputation_updates() {
        let manager = ReputationManager::new();
        let events = vec![
            ReputationEvent::ReputationAdjusted {
                did: "did:example:123".to_string(),
                adjustment: 10,
            },
            ReputationEvent::ReputationDecayApplied {
                did: "did:example:123".to_string(),
                decay_rate: 0.1,
            },
        ];
        manager.batch_reputation_updates(events).await.unwrap();
        // Add assertions based on the expected behavior of batch_reputation_updates
    }
}

pub struct Contribution {
    pub score: i64,
    pub timestamp: f64,
}

impl Contribution {
    pub fn new(score: i64) -> Self {
        Self {
            score,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64,
        }
    }
}

pub enum ReputationEvent {
    ReputationAdjusted { did: String, adjustment: i64 },
    ReputationDecayApplied { did: String, decay_rate: f64 },
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::env;
    use sqlx::PgPool;

    async fn setup_test_db() -> Arc<Database> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://icnuser:icnpass@db:5432/icndb".to_string());
        let pool = PgPool::connect(&database_url).await.unwrap();
        Arc::new(Database { pool })
    }

    #[tokio::test]
    async fn test_get_reputation() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let category = "governance";
        let score = service.get_reputation(did, category).await.unwrap();
        assert_eq!(score, 0); // Assuming initial score is 0
    }

    #[tokio::test]
    async fn test_adjust_reputation() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let category = "governance";
        service.adjust_reputation(did, category, 10, None).await.unwrap();
        let score = service.get_reputation(did, category).await.unwrap();
        assert_eq!(score, 10);
    }

    #[tokio::test]
    async fn test_apply_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.apply_decay(did).await.unwrap();
        let score = service.get_reputation(did, "governance").await.unwrap();
        assert!(score < 10); // Assuming initial score was 10 and decay was applied
    }

    #[tokio::test]
    async fn test_handle_sybil_resistance() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let reputation_score = 50;
        service.handle_sybil_resistance(did, reputation_score).await.unwrap();
        // Add assertions based on the expected behavior of handle_sybil_resistance
    }

    #[tokio::test]
    async fn test_apply_reputation_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let decay_rate = 0.05;
        service.apply_reputation_decay(did, decay_rate).await.unwrap();
        // Add assertions based on the expected behavior of apply_reputation_decay
    }

    #[tokio::test]
    async fn test_apply_adaptive_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.apply_adaptive_decay(did).await.unwrap();
        // Add assertions based on the expected behavior of apply_adaptive_decay
    }

    #[tokio::test]
    async fn test_record_contribution() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.record_contribution(did).await.unwrap();
        // Add assertions based on the expected behavior of record_contribution
    }
}
