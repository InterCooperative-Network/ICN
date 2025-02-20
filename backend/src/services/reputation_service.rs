use crate::database::db::Database;
use crate::models::Reputation;
use std::sync::Arc;

pub struct ReputationService {
    db: Arc<Database>,
}

impl ReputationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_reputation(&self, did: &str, category: &str) -> Result<i64, sqlx::Error> {
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

        Ok(())
    }
}
