use std::net::SocketAddr;
use warp::{Filter, Reply};
use log::{info, error};
use warp::ws::WebSocket;
use futures::{StreamExt, SinkExt};  // Combined StreamExt and added SinkExt
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN backend server...");

    // Database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");

    // WebSocket handler
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_websocket_connection)
        });

    // Health check route
    let health_route = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("health"))
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(health_handler);

    // Combine routes and add CORS
    let routes = health_route
        .or(ws_route)
        .with(warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        )
        .with(warp::log("icn_backend"));

    // Configure server address and start
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    info!("Server starting on http://{}", addr);
    
    warp::serve(routes)
        .run(addr)
        .await;
}

async fn handle_websocket_connection(ws: WebSocket) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    
    // Handle incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                info!("Received message: {:?}", msg);
                // Echo the message back
                if let Err(e) = ws_tx.send(msg).await {
                    error!("Error sending ws message: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Error receiving ws message: {}", e);
                break;
            }
        }
    }
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
