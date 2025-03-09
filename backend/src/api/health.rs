use warp::Filter;
use sqlx::PgPool;
use std::sync::Arc;

pub fn health_routes(db: Arc<PgPool>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db = Arc::clone(&db);
    warp::path!("api" / "v1" / "health")
        .and(warp::get())
        .and(with_db(db))
        .and_then(health_handler)
}

fn with_db(db: Arc<PgPool>) -> impl Filter<Extract = (Arc<PgPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn health_handler(db: Arc<PgPool>) -> Result<impl warp::Reply, warp::Rejection> {
    match sqlx::query("SELECT 1").execute(db.as_ref()).await {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "status": "ok",
            "database": "connected",
            "version": env!("CARGO_PKG_VERSION")
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "status": "degraded",
            "database": "error",
            "error": e.to_string(),
            "version": env!("CARGO_PKG_VERSION")
        })))
    }
}