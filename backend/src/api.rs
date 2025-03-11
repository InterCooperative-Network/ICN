use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{User, Node, Edge};
use crate::services::Services;

pub type AppState = Arc<SqlitePool>;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub title: String,
    pub content: String,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_id: i64,
    pub target_id: i64,
    pub relationship_type: String,
    pub user_id: i64,
}

pub async fn create_user(
    State(services): State<Services>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, String> {
    services.create_user(req.username, req.email, req.password)
        .await
        .map(Json)
}

pub async fn get_user(
    State(services): State<Services>,
    Path(id): Path<i64>,
) -> Result<Json<User>, String> {
    services.get_user(id)
        .await
        .map(Json)
}

pub async fn create_node(
    State(services): State<Services>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<Node>, String> {
    services.create_node(req.title, req.content, req.user_id)
        .await
        .map(Json)
}

pub async fn get_node(
    State(services): State<Services>,
    Path(id): Path<i64>,
) -> Result<Json<Node>, String> {
    services.get_node(id)
        .await
        .map(Json)
}

pub async fn create_edge(
    State(services): State<Services>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<Json<Edge>, String> {
    services.create_edge(req.source_id, req.target_id, req.relationship_type, req.user_id)
        .await
        .map(Json)
}

pub async fn get_edge(
    State(services): State<Services>,
    Path(id): Path<i64>,
) -> Result<Json<Edge>, String> {
    services.get_edge(id)
        .await
        .map(Json)
} 