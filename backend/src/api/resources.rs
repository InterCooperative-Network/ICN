use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::services::resource_service::{ResourceService, ResourceQuery};

#[derive(Debug, Deserialize, Serialize)]
struct QuerySharedResourcesRequest {
    resource_type: String,
    owner: Option<String>,
}

pub fn resource_routes(
    resource_service: Arc<Mutex<ResourceService>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let query_shared_resources = warp::path!("api" / "v1" / "resources" / "query")
        .and(warp::get())
        .and(warp::body::json())
        .and(with_resource_service(resource_service.clone()))
        .and_then(query_shared_resources_handler);

    query_shared_resources
}

fn with_resource_service(
    resource_service: Arc<Mutex<ResourceService>>,
) -> impl Filter<Extract = (Arc<Mutex<ResourceService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || resource_service.clone())
}

async fn query_shared_resources_handler(
    request: QuerySharedResourcesRequest,
    resource_service: Arc<Mutex<ResourceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let query = ResourceQuery {
        resource_type: request.resource_type,
        owner: request.owner,
    };

    let mut service = resource_service.lock().await;
    match service.query_shared_resources(query).await {
        Ok(resources) => {
            // Use EXPLAIN ANALYZE to verify index usage
            let explain_query = format!(
                "EXPLAIN ANALYZE SELECT * FROM resources WHERE resource_type = '{}' AND owner = '{}'",
                request.resource_type,
                request.owner.clone().unwrap_or_default()
            );
            let explain_result = sqlx::query(&explain_query)
                .fetch_all(&service.pool)
                .await
                .map_err(|e| warp::reject::custom(e))?;
            println!("EXPLAIN ANALYZE result: {:?}", explain_result);

            Ok(warp::reply::json(&resources))
        },
        Err(e) => Err(warp::reject::custom(e)),
    }
}
