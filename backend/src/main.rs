use std::net::SocketAddr;
use warp::{Filter, Reply};
use log::info;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Define routes
    let health_route = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("health"))
        .and(warp::get())
        .map(|| {
            info!("Health check requested");
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION")
            }))
        });

    let routes = health_route
        .with(warp::cors().allow_any_origin())
        .with(warp::log("icn_backend"));

    // Configure server address and start
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    info!("Server starting on http://{}", addr);
    
    warp::serve(routes)
        .run(addr)
        .await;
}

fn with_db(pool: sqlx::PgPool) -> impl Filter<Extract = (sqlx::PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn health_handler(pool: sqlx::PgPool) -> Result<impl Reply, warp::Rejection> {
    // Test database connectivity
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .is_ok();

    let status = if db_healthy {
        warp::http::StatusCode::OK
    } else {
        warp::http::StatusCode::SERVICE_UNAVAILABLE
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "status": if db_healthy { "healthy" } else { "unhealthy" },
            "version": env!("CARGO_PKG_VERSION"),
            "database": db_healthy
        })),
        status
    ))
}
