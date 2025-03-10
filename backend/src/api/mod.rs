pub mod health;
pub mod resource;
pub mod identity;
pub mod cooperative;

use warp::Filter;

// Re-exports for convenience
pub use health::health_routes;
pub use resource::resource_routes;
pub use identity::identity_routes;
pub use cooperative::cooperative_routes;

/// Combines all API routes
pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health::health_routes()
        .or(resource::resource_routes())
        .or(identity::identity_routes())
        .or(cooperative::cooperative_routes())
}