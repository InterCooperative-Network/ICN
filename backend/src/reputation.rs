use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ReputationManager {
    reputations: Mutex<HashMap<String, i64>>,
}

impl ReputationManager {
    pub fn new() -> Self {
        ReputationManager {
            reputations: Mutex::new(HashMap::new()),
        }
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
