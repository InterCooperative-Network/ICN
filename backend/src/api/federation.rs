use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use icn_federation::{FederationService, FederationOperation};
use icn_governance::{DissolutionProtocol, DissolutionReason, DissolutionStatus};

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
    let initiate_federation = warp::path!("api" / "v1" / "federation" / "initiate")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and_then(initiate_federation_handler);

    let join_federation = warp::path!("api" / "v1" / "federation" / "join")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and_then(join_federation_handler);

    let initiate_federation_dissolution = warp::path!("api" / "v1" / "federation" / String / "dissolve")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and_then(initiate_federation_dissolution_handler);

    let get_dissolution_status = warp::path!("api" / "v1" / "federation" / String / "dissolution" / "status")
        .and(warp::get())
        .and(with_federation_service(federation_service.clone()))
        .and_then(get_dissolution_status_handler);

    let cancel_federation_dissolution = warp::path!("api" / "v1" / "federation" / String / "dissolution" / "cancel")
        .and(warp::post())
        .and(with_federation_service(federation_service.clone()))
        .and_then(cancel_federation_dissolution_handler);

    let get_asset_distribution = warp::path!("api" / "v1" / "federation" / String / "dissolution" / "assets")
        .and(warp::get())
        .and(with_federation_service(federation_service.clone()))
        .and_then(get_asset_distribution_handler);

    let get_debt_settlements = warp::path!("api" / "v1" / "federation" / String / "dissolution" / "debts")
        .and(warp::get())
        .and(with_federation_service(federation_service.clone()))
        .and_then(get_debt_settlements_handler);

    initiate_federation
        .or(join_federation)
        .or(initiate_federation_dissolution)
        .or(get_dissolution_status)
        .or(cancel_federation_dissolution)
        .or(get_asset_distribution)
        .or(get_debt_settlements)
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

async fn initiate_federation_dissolution_handler(
    federation_id: String,
    request: DissolutionRequest,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut service = federation_service.lock().await;
    let protocol = service.initiate_dissolution(&federation_id, &request.initiator_id, request.reason.clone()).await?;
    Ok(warp::reply::json(&protocol))
}

async fn get_dissolution_status_handler(
    federation_id: String,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let service = federation_service.lock().await;
    let status = service.get_federation_dissolution_status(&federation_id).await?;
    Ok(warp::reply::json(&status))
}

async fn cancel_federation_dissolution_handler(
    federation_id: String,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut service = federation_service.lock().await;
    service.cancel_dissolution(&federation_id).await?;
    Ok(warp::reply::json(&"Dissolution cancelled"))
}

async fn get_asset_distribution_handler(
    federation_id: String,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let service = federation_service.lock().await;
    let distribution = service.calculate_asset_distribution(&federation_id).await?;
    Ok(warp::reply::json(&distribution))
}

async fn get_debt_settlements_handler(
    federation_id: String,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let service = federation_service.lock().await;
    let settlements = service.settle_outstanding_debts(&federation_id).await?;
    Ok(warp::reply::json(&settlements))
}
