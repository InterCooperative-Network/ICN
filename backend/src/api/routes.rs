use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use crate::api::handlers::{
    health_handler,
    identity_handlers::{create_identity_handler, get_identity_list_handler},
    federation_handlers::{list_federations_handler, create_federation_handler, join_federation_handler, leave_federation_handler},
    governance_handlers::{list_proposals_handler, create_proposal_handler, vote_proposal_handler},
    resource_handlers::{list_resources_handler, share_resource_handler, request_resource_handler},
};
use crate::core::Core;

/// Build all API routes
pub fn api_routes(
    core: Arc<Core>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_path = warp::path("api").and(warp::path("v1"));
    
    // Health check endpoint
    let health_route = api_path
        .and(warp::path("health"))
        .and(warp::get())
        .and_then(health_handler);
    
    // Identity routes
    let identity_routes = api_path
        .and(warp::path("identity"))
        .and(
            warp::post()
                .and(with_core(core.clone()))
                .and_then(create_identity_handler)
                .or(warp::get()
                    .and(warp::path("list"))
                    .and(with_core(core.clone()))
                    .and_then(get_identity_list_handler)
                )
        );
    
    // Federation routes
    let federation_routes = api_path
        .and(warp::path("federations"))
        .and(
            warp::get()
                .and(with_core(core.clone()))
                .and_then(list_federations_handler)
                .or(warp::post()
                    .and(with_core(core.clone()))
                    .and(warp::body::json())
                    .and_then(create_federation_handler)
                )
                .or(warp::path::param()
                    .and(warp::path("join"))
                    .and(warp::post())
                    .and(with_core(core.clone()))
                    .and_then(join_federation_handler)
                )
                .or(warp::path::param()
                    .and(warp::path("leave"))
                    .and(warp::post())
                    .and(with_core(core.clone()))
                    .and_then(leave_federation_handler)
                )
        );
    
    // Governance routes
    let governance_routes = api_path
        .and(warp::path("governance"))
        .and(warp::path("proposals"))
        .and(
            warp::get()
                .and(with_core(core.clone()))
                .and_then(list_proposals_handler)
                .or(warp::post()
                    .and(with_core(core.clone()))
                    .and(warp::body::json())
                    .and_then(create_proposal_handler)
                )
                .or(warp::path::param()
                    .and(warp::path("vote"))
                    .and(warp::post())
                    .and(with_core(core.clone()))
                    .and(warp::body::json())
                    .and_then(vote_proposal_handler)
                )
        );
    
    // Resource routes
    let resource_routes = api_path
        .and(warp::path("resources"))
        .and(
            warp::get()
                .and(with_core(core.clone()))
                .and_then(list_resources_handler)
                .or(warp::path("share")
                    .and(warp::post())
                    .and(with_core(core.clone()))
                    .and(warp::body::json())
                    .and_then(share_resource_handler)
                )
                .or(warp::path("request")
                    .and(warp::post())
                    .and(with_core(core.clone()))
                    .and(warp::body::json())
                    .and_then(request_resource_handler)
                )
        );
    
    // Combine all routes
    health_route
        .or(identity_routes)
        .or(federation_routes)
        .or(governance_routes)
        .or(resource_routes)
        .with(warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_headers(vec!["Content-Type", "Authorization"])
        )
}

/// Utility function to pass Core state to handlers
fn with_core(
    core: Arc<Core>,
) -> impl Filter<Extract = (Arc<Core>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || core.clone())
}