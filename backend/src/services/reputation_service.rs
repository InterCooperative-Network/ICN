use crate::database::db::Database;
use crate::models::Reputation;
use std::sync::Arc;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ReputationCache {
    cache: DashMap<String, i32>,
    max_size: usize,
}

impl ReputationCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
        }
    }

    fn get(&self, did: &str) -> Option<i32> {
        self.cache.get(did).map(|v| *v)
    }

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

pub struct ReputationService {
    db: Arc<Database>,
    cache: ReputationCache,
    decay_rate: f64,
}

impl ReputationService {
    pub fn new(db: Arc<Database>, max_cache_size: usize, decay_rate: f64) -> Self {
        Self {
            db,
            cache: ReputationCache::new(max_cache_size),
            decay_rate,
        }
    }

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

    pub async fn adjust_reputation(&self, did: &str, category: &str, adjustment: i64) -> Result<(), sqlx::Error> {
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
