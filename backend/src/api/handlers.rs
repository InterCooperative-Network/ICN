use axum::{
    extract::{Path, Extension},
    Json,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::models::Node;

// Health check response
#[derive(Serialize)]
pub struct HealthCheck {
    status: String,
    version: String,
}

// Health check handler
pub async fn health_check() -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// List nodes handler
pub async fn list_nodes(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Node>>, crate::api::ApiError> {
    let nodes = sqlx::query_as!(
        Node,
        "SELECT id, name, status, last_seen, version FROM nodes"
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::api::ApiError::Database)?;

    Ok(Json(nodes))
}

// Get single node handler
pub async fn get_node(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Node>, crate::api::ApiError> {
    let node = sqlx::query_as!(
        Node,
        "SELECT id, name, status, last_seen, version FROM nodes WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::api::ApiError::Database)?
    .ok_or_else(|| crate::api::ApiError::NotFound(format!("Node {} not found", id)))?;

    Ok(Json(node))
}

// Join federation request
#[derive(Deserialize)]
pub struct JoinFederationRequest {
    node_name: String,
    node_version: String,
    federation_id: String,
}

// Join federation handler
pub async fn join_federation(
    Extension(pool): Extension<PgPool>,
    Json(request): Json<JoinFederationRequest>,
) -> Result<Json<Node>, crate::api::ApiError> {
    let node = sqlx::query_as!(
        Node,
        r#"
        INSERT INTO nodes (name, status, version, federation_id)
        VALUES ($1, 'online', $2, $3)
        RETURNING id, name, status, last_seen, version
        "#,
        request.node_name,
        request.node_version,
        request.federation_id
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::api::ApiError::Database)?;

    Ok(Json(node))
}
