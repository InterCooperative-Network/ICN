pub mod health;
pub mod identity;
pub mod governance;
pub mod resource;
pub mod federation;
pub mod federation_resource_sharing;
pub mod reputation;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use crate::services::{
    IdentityService,
    GovernanceService,
    ResourceService,
    FederationService,
    ReputationService,
    p2p::P2PManager
};
use crate::middleware::auth::with_auth;

pub fn routes(
    identity_service: Arc<Mutex<IdentityService>>,
    governance_service: Arc<Mutex<GovernanceService>>,
    resource_service: Arc<Mutex<ResourceService>>,
    federation_service: Arc<Mutex<FederationService>>,
    reputation_service: Arc<Mutex<ReputationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // Routes that require auth
    let auth_routes = identity::identity_routes(identity_service.clone())
        .or(governance::governance_routes(governance_service, p2p_manager.clone(), identity_service.clone()))
        .or(resource::resource_routes(resource_service))
        .or(federation::federation_routes(federation_service.clone(), p2p_manager.clone()))
        .or(federation_resource_sharing::federation_resource_sharing_routes(federation_service, p2p_manager.clone()))
        .or(reputation::reputation_routes(p2p_manager));

    // Apply auth middleware
    auth_routes.and(with_auth(identity_service))
}