use sqlx::PgPool;
use uuid::Uuid;
use crate::models::Node;
use crate::api::ApiError;

pub struct NodeService {
    pool: PgPool,
}

impl NodeService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_nodes(&self) -> Result<Vec<Node>, ApiError> {
        sqlx::query_as!(
            Node,
            "SELECT id, name, status, last_seen, version FROM nodes"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(ApiError::Database)
    }

    pub async fn get_node(&self, id: Uuid) -> Result<Option<Node>, ApiError> {
        sqlx::query_as!(
            Node,
            "SELECT id, name, status, last_seen, version FROM nodes WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(ApiError::Database)
    }

    pub async fn create_node(
        &self,
        name: String,
        version: String,
    ) -> Result<Node, ApiError> {
        sqlx::query_as!(
            Node,
            r#"
            INSERT INTO nodes (name, status, version)
            VALUES ($1, 'online', $2)
            RETURNING id, name, status, last_seen, version
            "#,
            name,
            version
        )
        .fetch_one(&self.pool)
        .await
        .map_err(ApiError::Database)
    }

    pub async fn update_node_status(
        &self,
        id: Uuid,
        status: String,
    ) -> Result<Node, ApiError> {
        sqlx::query_as!(
            Node,
            r#"
            UPDATE nodes
            SET status = $1, last_seen = CURRENT_TIMESTAMP
            WHERE id = $2
            RETURNING id, name, status, last_seen, version
            "#,
            status,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(ApiError::Database)
    }
} 