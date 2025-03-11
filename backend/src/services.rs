use sqlx::SqlitePool;
use crate::models::{User, Node, Edge};
use bcrypt::{hash, DEFAULT_COST};

#[derive(Clone)]
pub struct Services {
    pool: SqlitePool,
}

impl Services {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, username: String, email: String, password: String) -> Result<User, String> {
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES (?, ?, ?)
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))
    }

    pub async fn get_user(&self, id: i64) -> Result<User, String> {
        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get user: {}", e))
    }

    pub async fn create_node(&self, title: String, content: String, user_id: i64) -> Result<Node, String> {
        sqlx::query_as!(
            Node,
            r#"
            INSERT INTO nodes (title, content, user_id)
            VALUES (?, ?, ?)
            RETURNING id, title, content, user_id, created_at, updated_at
            "#,
            title,
            content,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create node: {}", e))
    }

    pub async fn get_node(&self, id: i64) -> Result<Node, String> {
        sqlx::query_as!(
            Node,
            r#"
            SELECT * FROM nodes WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get node: {}", e))
    }

    pub async fn create_edge(&self, source_id: i64, target_id: i64, relationship_type: String, user_id: i64) -> Result<Edge, String> {
        sqlx::query_as!(
            Edge,
            r#"
            INSERT INTO edges (source_id, target_id, relationship_type, user_id)
            VALUES (?, ?, ?, ?)
            RETURNING id, source_id, target_id, relationship_type, user_id, created_at, updated_at
            "#,
            source_id,
            target_id,
            relationship_type,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create edge: {}", e))
    }

    pub async fn get_edge(&self, id: i64) -> Result<Edge, String> {
        sqlx::query_as!(
            Edge,
            r#"
            SELECT * FROM edges WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get edge: {}", e))
    }
}
