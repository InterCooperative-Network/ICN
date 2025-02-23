use crate::database::db::Database;
use crate::models::Reputation;
use std::sync::Arc;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use std::time::{SystemTime, Duration, UNIX_EPOCH};

/// A cache for storing reputation scores.
pub struct ReputationCache {
    cache: DashMap<String, i32>,
    max_size: usize,
}

impl ReputationCache {
    /// Creates a new ReputationCache with the specified maximum size.
    fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
        }
    }

    /// Retrieves the reputation score for the given DID from the cache.
    fn get(&self, did: &str) -> Option<i32> {
        self.cache.get(did).map(|v| *v)
    }

    /// Sets the reputation score for the given DID in the cache.
    fn set(&self, did: &str, score: i32) {
        if self.cache.len() >= self.max_size {
            // Implement a simple eviction policy (e.g., remove a random entry)
            if let Some(key) = self.cache.iter().next().map(|entry| entry.key().clone()) {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(did.to_string(), score);
    }
}

/// A service for managing reputation scores.
pub struct ReputationService {
    db: Arc<Database>,
    cache: ReputationCache,
    decay_rate: f64,
}

pub struct ReputationConfig {
    base_decay_rate: f64,
    min_decay_rate: f64,
    max_decay_rate: f64,
    grace_period_days: u64,
    reputation_half_life: f64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            base_decay_rate: 0.1,
            min_decay_rate: 0.02, // Minimum 2% decay for long-standing members
            max_decay_rate: 0.2,  // Maximum 20% decay for new/inactive members
            grace_period_days: 30, // 30 day grace period
            reputation_half_life: 365.0, // 1 year half-life for reputation weight
        }
    }
}

impl ReputationService {
    /// Creates a new ReputationService with the specified database, cache size, and decay rate.
    pub fn new(db: Arc<Database>, max_cache_size: usize, decay_rate: f64) -> Self {
        Self {
            db,
            cache: ReputationCache::new(max_cache_size),
            decay_rate,
        }
    }

    /// Retrieves the reputation score for the given DID and category.
    pub async fn get_reputation(&self, did: &str, category: &str) -> Result<i64, sqlx::Error> {
        if let Some(score) = self.cache.get(did) {
            return Ok(score as i64);
        }

        let reputation = sqlx::query_as!(
            Reputation,
            r#"
            SELECT score FROM reputations WHERE did = $1 AND category = $2
            "#,
            did,
            category
        )
        .fetch_one(&*self.db.pool)
        .await?;

        self.cache.set(did, reputation.score as i32);
        Ok(reputation.score)
    }

    /// Adjusts the reputation score for the given DID and category by the specified adjustment value.
    pub async fn adjust_reputation(&self, did: &str, category: &str, adjustment: i64, zk_snark_proof: Option<&str>) -> Result<(), sqlx::Error> {
        if let Some(proof) = zk_snark_proof {
            if !verify_proof(proof) {
                return Err(sqlx::Error::Protocol("Invalid zk-SNARK proof".to_string()));
            }
        }

        sqlx::query!(
            r#"
            INSERT INTO reputations (did, category, score)
            VALUES ($1, $2, $3)
            ON CONFLICT (did, category) DO UPDATE SET score = reputations.score + $3
            "#,
            did,
            category,
            adjustment
        )
        .execute(&*self.db.pool)
        .await?;

        // Update the cache after adjusting the reputation
        if let Some(mut score) = self.cache.get(did) {
            score += adjustment as i32;
            self.cache.set(did, score);
        } else {
            let reputation = sqlx::query_as!(
                Reputation,
                r#"
                SELECT score FROM reputations WHERE did = $1 AND category = $2
                "#,
                did,
                category
            )
            .fetch_one(&*self.db.pool)
            .await?;
            self.cache.set(did, reputation.score as i32);
        }

        Ok(())
    }

    /// Applies decay to the reputation scores for the given DID.
    pub async fn apply_decay(&self, did: &str) -> Result<(), sqlx::Error> {
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
            let decayed_score = (contribution.score as f64 * (-self.decay_rate * age).exp()) as i64;
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

    /// Handles Sybil resistance mechanisms for DID reputation tracking.
    pub async fn handle_sybil_resistance(&self, did: &str, reputation_score: i64) -> Result<(), sqlx::Error> {
        // Placeholder logic for handling Sybil resistance
        Ok(())
    }

    /// Applies reputation decay for the given DID with a specified decay rate.
    pub async fn apply_reputation_decay(&self, did: &str, decay_rate: f64) -> Result<(), sqlx::Error> {
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

    pub async fn apply_adaptive_decay(&self, did: &str) -> Result<(), sqlx::Error> {
        let config = ReputationConfig::default();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        // Get member history
        let history = sqlx::query!(
            r#"
            SELECT 
                join_date,
                last_contribution,
                total_contributions,
                current_reputation
            FROM member_history 
            WHERE did = $1
            "#,
            did
        )
        .fetch_one(&*self.db.pool)
        .await?;

        // Calculate member tenure in days
        let tenure_days = (now.as_secs() - history.join_date) / 86400;
        
        // Check grace period
        let days_since_contribution = (now.as_secs() - history.last_contribution) / 86400;
        if days_since_contribution <= config.grace_period_days {
            return Ok(()); // No decay during grace period
        }

        // Calculate adaptive decay rate based on tenure and contribution history
        let tenure_factor = (-tenure_days as f64 / config.reputation_half_life).exp();
        let contribution_factor = (history.total_contributions as f64).sqrt() / 10.0;
        
        let adaptive_rate = config.base_decay_rate * 
            (1.0 - tenure_factor) * 
            (1.0 - contribution_factor)
            .clamp(config.min_decay_rate, config.max_decay_rate);

        // Apply the decay
        let new_reputation = (history.current_reputation as f64 * 
            (1.0 - adaptive_rate)).round() as i64;

        sqlx::query!(
            r#"
            UPDATE member_history 
            SET current_reputation = $1
            WHERE did = $2
            "#,
            new_reputation,
            did
        )
        .execute(&*self.db.pool)
        .await?;

        // Update cache
        self.cache.set(did, new_reputation as i32);

        Ok(())
    }

    pub async fn record_contribution(&self, did: &str) -> Result<(), sqlx::Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        sqlx::query!(
            r#"
            UPDATE member_history 
            SET 
                last_contribution = $1,
                total_contributions = total_contributions + 1
            WHERE did = $2
            "#,
            now as i64,
            did
        )
        .execute(&*self.db.pool)
        .await?;

        Ok(())
    }
}

/// Represents a contribution with a score and timestamp.
pub struct Contribution {
    pub score: i64,
    pub timestamp: f64,
}

impl Contribution {
    /// Creates a new Contribution with the specified score and the current timestamp.
    pub fn new(score: i64) -> Self {
        Self {
            score,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;

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

    #[tokio::test]
    async fn test_integration_get_reputation() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let category = "governance";
        let score = service.get_reputation(did, category).await.unwrap();
        assert_eq!(score, 0); // Assuming initial score is 0
    }

    #[tokio::test]
    async fn test_integration_adjust_reputation() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let category = "governance";
        service.adjust_reputation(did, category, 10, None).await.unwrap();
        let score = service.get_reputation(did, category).await.unwrap();
        assert_eq!(score, 10);
    }

    #[tokio::test]
    async fn test_integration_apply_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.apply_decay(did).await.unwrap();
        let score = service.get_reputation(did, "governance").await.unwrap();
        assert!(score < 10); // Assuming initial score was 10 and decay was applied
    }

    #[tokio::test]
    async fn test_integration_handle_sybil_resistance() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let reputation_score = 50;
        service.handle_sybil_resistance(did, reputation_score).await.unwrap();
        // Add assertions based on the expected behavior of handle_sybil_resistance
    }

    #[tokio::test]
    async fn test_integration_apply_reputation_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        let decay_rate = 0.05;
        service.apply_reputation_decay(did, decay_rate).await.unwrap();
        // Add assertions based on the expected behavior of apply_reputation_decay
    }

    #[tokio::test]
    async fn test_integration_apply_adaptive_decay() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.apply_adaptive_decay(did).await.unwrap();
        // Add assertions based on the expected behavior of apply_adaptive_decay
    }

    #[tokio::test]
    async fn test_integration_record_contribution() {
        let db = setup_test_db().await;
        let service = ReputationService::new(db, 100, 0.1);

        let did = "did:icn:test";
        service.record_contribution(did).await.unwrap();
        // Add assertions based on the expected behavior of record_contribution
    }
}
