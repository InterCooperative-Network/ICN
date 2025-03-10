use warp::{Rejection, Reply};
use std::sync::Arc;
use serde_json::{json, Value};
use crate::core::Core;

pub mod identity_handlers;
pub mod federation_handlers;
pub mod governance_handlers;
pub mod resource_handlers;

/// Health check handler
pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

// Helper function to create standardized error responses
pub fn api_error(status_code: u16, message: &str) -> impl Reply {
    warp::reply::with_status(
        warp::reply::json(&json!({
            "error": message
        })),
        warp::http::StatusCode::from_u16(status_code).unwrap()
    )
}

// Common type definitions for API handlers
pub type ApiResult<T> = Result<T, Rejection>;
pub type HandlerResult = ApiResult<impl Reply>;