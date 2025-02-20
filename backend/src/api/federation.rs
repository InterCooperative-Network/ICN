use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use icn_federation::{FederationService, FederationOperation};

#[derive(Debug, Deserialize, Serialize)]
struct InitiateFederationRequest {
    federation_type: String,
    partner_id: String,
    terms: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JoinFederationRequest {
    federation_id: String,
    commitment: String,
}

pub fn federation_routes(
    federation_service: Arc<Mutex<FederationService>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let initiate_federation = warp::path!("api" / "federation" / "initiate")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and_then(initiate_federation_handler);

    let join_federation = warp::path!("api" / "federation" / "join")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and_then(join_federation_handler);

    initiate_federation.or(join_federation)
}

fn with_federation_service(
    federation_service: Arc<Mutex<FederationService>>,
) -> impl Filter<Extract = (Arc<Mutex<FederationService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || federation_service.clone())
}

async fn initiate_federation_handler(
    request: InitiateFederationRequest,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let operation = FederationOperation::InitiateFederation {
        federation_type: request.federation_type,
        partner_id: request.partner_id,
        terms: request.terms,
    };

    let mut service = federation_service.lock().await;
    match service.handle_operation(operation).await {
        Ok(_) => Ok(warp::reply::json(&"Federation initiated")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

async fn join_federation_handler(
    request: JoinFederationRequest,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let operation = FederationOperation::JoinFederation {
        federation_id: request.federation_id,
        commitment: request.commitment,
    };

    let mut service = federation_service.lock().await;
    match service.handle_operation(operation).await {
        Ok(_) => Ok(warp::reply::json(&"Joined federation")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
