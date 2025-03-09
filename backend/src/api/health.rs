use warp::Filter;
use sqlx::PgPool;
use std::sync::Arc;

pub fn health_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api_health = warp::path!("api" / "v1" / "health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({ "status": "ok" })));
        
    api_health
}