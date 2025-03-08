use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

use crate::api::health::health_routes;
use crate::api::identity::identity_routes;
use crate::api::governance::governance_routes;
use crate::api::resource::resource_routes;
use crate::api::federation::federation_routes;
use crate::api::federation_resource_sharing::federation_resource_sharing_routes;
use crate::api::reputation::reputation_routes;
use crate::services::identity_service::IdentityService;
use crate::services::governance_service::GovernanceService;
use crate::services::resource_service::ResourceService;
use crate::services::federation_service::FederationService;
use crate::services::reputation_service::ReputationService;
use crate::services::p2p::P2PManager;
use crate::middleware::auth::with_auth;
use crate::middleware::cors::cors;

pub fn routes(
    identity_service: Arc<Mutex<IdentityService>>,
    governance_service: Arc<Mutex<GovernanceService>>,
    resource_service: Arc<Mutex<ResourceService>>,
    federation_service: Arc<Mutex<FederationService>>,
    reputation_service: Arc<Mutex<ReputationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // Routes that don't require auth
    let health_route = health_routes();

    // Routes that require auth
    let auth_routes = identity_routes(identity_service.clone())
        .or(governance_routes(governance_service.clone(), p2p_manager.clone()))
        .or(resource_routes(resource_service.clone()))
        .or(federation_routes(federation_service.clone(), p2p_manager.clone()))
        .or(federation_resource_sharing_routes(federation_service.clone(), p2p_manager.clone()))
        .or(reputation_routes(reputation_service.clone()));

    // Apply middleware
    let auth_routes = auth_routes.and(with_auth(identity_service.clone()));

    // Combine routes
    health_route.or(auth_routes).with(cors())
}