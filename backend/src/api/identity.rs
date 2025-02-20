use warp::Filter;
use crate::services::identity_service::{IdentityService, IdentityServiceImpl};
use std::sync::Arc;

pub fn identity_routes(
    identity_service: Arc<dyn IdentityService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let create_identity = warp::path!("api" / "v1" / "identity" / "create")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_create_identity);

    let get_identity = warp::path!("api" / "v1" / "identity" / "get" / String)
        .and(warp::get())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_get_identity);

    create_identity.or(get_identity)
}

fn with_identity_service(
    identity_service: Arc<dyn IdentityService>,
) -> impl Filter<Extract = (Arc<dyn IdentityService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || identity_service.clone())
}

async fn handle_create_identity(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    identity_service.create_identity(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::with_status("Identity created", warp::http::StatusCode::CREATED))
}

async fn handle_get_identity(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let data = identity_service.get_identity(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::json(&data))
}
