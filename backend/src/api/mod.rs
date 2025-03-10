pub mod health;
pub mod resource;
pub mod identity;
pub mod cooperative;
pub mod network;

use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::networking::NetworkManager;

// Re-exports for convenience
pub use health::health_routes;
pub use resource::resource_routes;
pub use identity::identity_routes;
pub use cooperative::cooperative_routes;
pub use network::network_routes;

/// Combines all API routes
pub fn routes(network_manager: Arc<Mutex<NetworkManager>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health::health_routes()
        .or(resource::resource_routes())
        .or(identity::identity_routes())
        .or(cooperative::cooperative_routes())
        .or(network::network_routes(network_manager))
}