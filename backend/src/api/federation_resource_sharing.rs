use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::services::federation_service::FederationService;
use crate::services::p2p::P2PManager;
use crate::errors::IcnError;

/// Request to share resources between federations
#[derive(Debug, Deserialize, Serialize)]
pub struct FederationResourceSharingRequest {
    /// ID of the source federation sharing the resource
    pub source_federation_id: String,
    /// ID of the target federation receiving the resource
    pub target_federation_id: String,
    /// Type of resource being shared
    pub resource_type: String,
    /// Amount of resource to share
    pub amount: u64,
    /// Optional duration for the sharing agreement (in seconds)
    pub duration_seconds: Option<u64>,
    /// Terms of the sharing agreement
    pub terms: String,
    /// Minimum reputation score required for the target federation
    pub min_reputation_score: i64,
    /// Cryptographic signature of the request
    pub signature: String,
}

/// Response for federation resource sharing operations
#[derive(Debug, Serialize, Deserialize)]
pub struct FederationResourceSharingResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// ID of the created sharing agreement (if successful)
    pub agreement_id: Option<String>,
}

/// Request to allocate resources from a sharing agreement
#[derive(Debug, Deserialize, Serialize)]
pub struct AllocateSharedResourceRequest {
    /// ID of the sharing agreement
    pub agreement_id: String,
    /// Amount of resource to allocate
    pub amount: u64,
    /// DID of the requester
    pub requester_did: String,
    /// Cryptographic signature of the request
    pub signature: String,
}

/// Request to release resources back to a sharing agreement
#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseSharedResourceRequest {
    /// ID of the sharing agreement
    pub agreement_id: String,
    /// ID of the allocation
    pub allocation_id: String,
    /// Amount of resource to release
    pub amount: u64,
}

/// Response for resource allocation operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAllocationResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// ID of the allocation (if successful)
    pub allocation_id: Option<String>,
}

/// Generate federation resource sharing API routes
pub fn federation_resource_sharing_routes(
    federation_service: Arc<Mutex<FederationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let share_resources = warp::path!("api" / "v1" / "federation" / "resources" / "share")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and(with_p2p_manager(p2p_manager.clone()))
        .and_then(share_federation_resources_handler);

    let allocate_shared_resource = warp::path!("api" / "v1" / "federation" / "resources" / "shared" / "allocate")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and(with_p2p_manager(p2p_manager.clone()))
        .and_then(allocate_shared_resource_handler);

    let release_shared_resource = warp::path!("api" / "v1" / "federation" / "resources" / "shared" / "release")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_federation_service(federation_service.clone()))
        .and(with_p2p_manager(p2p_manager.clone()))
        .and_then(release_shared_resource_handler);

    let list_federation_sharing_agreements = warp::path!("api" / "v1" / "federation" / String / "sharing-agreements")
        .and(warp::get())
        .and(with_federation_service(federation_service.clone()))
        .and_then(list_federation_sharing_agreements_handler);

    share_resources
        .or(allocate_shared_resource)
        .or(release_shared_resource)
        .or(list_federation_sharing_agreements)
}

/// Helper to include federation service in route handlers
fn with_federation_service(
    federation_service: Arc<Mutex<FederationService>>,
) -> impl Filter<Extract = (Arc<Mutex<FederationService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || federation_service.clone())
}

/// Helper to include P2P manager in route handlers
fn with_p2p_manager(
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> impl Filter<Extract = (Arc<Mutex<P2PManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || p2p_manager.clone())
}

/// Handler for sharing resources between federations
async fn share_federation_resources_handler(
    request: FederationResourceSharingRequest,
    federation_service: Arc<Mutex<FederationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Verify signature
    if !verify_signature(&request.source_federation_id, &request.signature).await {
        return Ok(warp::reply::json(&FederationResourceSharingResponse {
            success: false,
            message: "Invalid signature".to_string(),
            agreement_id: None,
        }));
    }

    // Process the request
    let mut service = federation_service.lock().await;
    match service.share_federation_resources(
        request.source_federation_id,
        request.target_federation_id,
        request.resource_type,
        request.amount,
        request.duration_seconds,
        request.terms,
        request.min_reputation_score,
    ).await {
        Ok(agreement_id) => {
            // Publish event
            let event = FederationEvent::ResourceSharing {
                source_federation_id: request.source_federation_id,
                target_federation_id: request.target_federation_id,
                resource_type: request.resource_type,
                amount: request.amount,
                agreement_id: agreement_id.clone(),
            };
            
            let mut p2p = p2p_manager.lock().await;
            if let Err(e) = p2p.publish(event).await {
                return Ok(warp::reply::json(&FederationResourceSharingResponse {
                    success: false,
                    message: format!("Failed to publish event: {}", e),
                    agreement_id: None,
                }));
            }
            
            Ok(warp::reply::json(&FederationResourceSharingResponse {
                success: true,
                message: "Resource sharing agreement created successfully".to_string(),
                agreement_id: Some(agreement_id),
            }))
        },
        Err(e) => {
            Ok(warp::reply::json(&FederationResourceSharingResponse {
                success: false,
                message: format!("Failed to share resources: {}", e),
                agreement_id: None,
            }))
        }
    }
}

/// Handler for allocating resources from a sharing agreement
async fn allocate_shared_resource_handler(
    request: AllocateSharedResourceRequest,
    federation_service: Arc<Mutex<FederationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Verify signature
    if !verify_signature(&request.requester_did, &request.signature).await {
        return Ok(warp::reply::json(&ResourceAllocationResponse {
            success: false,
            message: "Invalid signature".to_string(),
            allocation_id: None,
        }));
    }

    // Process the request
    let mut service = federation_service.lock().await;
    match service.allocate_shared_resource(
        &request.agreement_id,
        &request.requester_did,
        request.amount,
    ).await {
        Ok(allocation_id) => {
            // Publish event
            let event = FederationEvent::ResourceAllocation {
                agreement_id: request.agreement_id,
                requester_did: request.requester_did,
                amount: request.amount,
                allocation_id: allocation_id.clone(),
            };
            
            let mut p2p = p2p_manager.lock().await;
            if let Err(e) = p2p.publish(event).await {
                return Ok(warp::reply::json(&ResourceAllocationResponse {
                    success: false,
                    message: format!("Failed to publish event: {}", e),
                    allocation_id: None,
                }));
            }
            
            Ok(warp::reply::json(&ResourceAllocationResponse {
                success: true,
                message: "Resources allocated successfully".to_string(),
                allocation_id: Some(allocation_id),
            }))
        },
        Err(e) => {
            Ok(warp::reply::json(&ResourceAllocationResponse {
                success: false,
                message: format!("Failed to allocate resources: {}", e),
                allocation_id: None,
            }))
        }
    }
}

/// Handler for releasing resources back to a sharing agreement
async fn release_shared_resource_handler(
    request: ReleaseSharedResourceRequest,
    federation_service: Arc<Mutex<FederationService>>,
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Process the request
    let mut service = federation_service.lock().await;
    match service.release_shared_resource(
        &request.agreement_id,
        &request.allocation_id,
        request.amount,
    ).await {
        Ok(_) => {
            // Publish event
            let event = FederationEvent::ResourceRelease {
                agreement_id: request.agreement_id,
                allocation_id: request.allocation_id,
                amount: request.amount,
            };
            
            let mut p2p = p2p_manager.lock().await;
            if let Err(e) = p2p.publish(event).await {
                return Err(warp::reject::custom(IcnError::NetworkError(e.to_string())));
            }
            
            Ok(warp::reply::json(&"Resources released successfully"))
        },
        Err(e) => Err(warp::reject::custom(IcnError::FederationError(e.to_string()))),
    }
}

/// Handler for listing all sharing agreements of a federation
async fn list_federation_sharing_agreements_handler(
    federation_id: String,
    federation_service: Arc<Mutex<FederationService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let service = federation_service.lock().await;
    match service.list_federation_sharing_agreements(&federation_id).await {
        Ok(agreements) => Ok(warp::reply::json(&agreements)),
        Err(e) => Err(warp::reject::custom(IcnError::FederationError(e.to_string()))),
    }
}

/// Helper function to verify cryptographic signatures
async fn verify_signature(did: &str, signature: &str) -> bool {
    // This is a placeholder. In a real implementation, this would:
    // 1. Fetch the DID Document to get the public key
    // 2. Verify the signature against the public key
    // 3. Return true if valid, false otherwise
    
    // For now, we'll just return true for development purposes
    // TODO: Replace with actual signature verification
    true
}

/// Events related to federation activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationEvent {
    ResourceSharing {
        source_federation_id: String,
        target_federation_id: String,
        resource_type: String,
        amount: u64,
        agreement_id: String,
    },
    ResourceAllocation {
        agreement_id: String,
        requester_did: String,
        amount: u64,
        allocation_id: String,
    },
    ResourceRelease {
        agreement_id: String,
        allocation_id: String,
        amount: u64,
    },
}