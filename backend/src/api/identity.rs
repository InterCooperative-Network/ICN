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

    let rotate_key = warp::path!("api" / "v1" / "identity" / "rotate_key" / String)
        .and(warp::post())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_rotate_key);

    let revoke_key = warp::path!("api" / "v1" / "identity" / "revoke_key" / String)
        .and(warp::post())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_revoke_key);

    create_identity.or(get_identity).or(rotate_key).or(revoke_key)
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

async fn handle_rotate_key(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    identity_service.rotate_key(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::with_status("Key rotated", warp::http::StatusCode::OK))
}

async fn handle_revoke_key(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    identity_service.revoke_key(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::with_status("Key revoked", warp::http::StatusCode::OK))
}
