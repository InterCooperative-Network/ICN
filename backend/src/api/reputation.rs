use warp::Filter;
use crate::services::reputation_service::{get_reputation, adjust_reputation, verify_contribution, handle_sybil_resistance, apply_reputation_decay};
use icn_networking::p2p::{P2PManager, ReputationEvent}; // Import P2PManager and ReputationEvent
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
struct ZkSnarkProofRequest {
    proof: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SybilResistanceRequest {
    did: String,
    reputation_score: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReputationDecayRequest {
    did: String,
    decay_rate: f64,
}

pub fn reputation_routes(
    p2p_manager: Arc<Mutex<P2PManager>>, // Add P2PManager to reputation_routes
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(
            warp::path("v1")
                .and(
                    warp::path("reputation")
                        .and(
                            warp::path("get")
                                .and(warp::get())
                                .and(warp::query::<String>())
                                .and_then(get_reputation)
                        )
                        .or(
                            warp::path("adjust")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(adjust_reputation)
                        )
                        .or(
                            warp::path("verify")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(verify_contribution)
                        )
                        .or(
                            warp::path("zk_snark_proof")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and(with_p2p_manager(p2p_manager.clone())) // Add with_p2p_manager
                                .and_then(submit_zk_snark_proof_handler)
                        )
                        .or(
                            warp::path("sybil_resistance")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(handle_sybil_resistance)
                        )
                        .or(
                            warp::path("reputation_decay")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(apply_reputation_decay)
                        )
                        .or(
                            warp::path("apply_reputation_decay")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(apply_reputation_decay_handler)
                        )
                        .or(
                            warp::path("handle_sybil_resistance")
                                .and(warp::post())
                                .and(warp::body::json())
                                .and_then(handle_sybil_resistance_handler)
                        )
                )
        )
}

fn with_p2p_manager(
    p2p_manager: Arc<Mutex<P2PManager>>,
) -> impl Filter<Extract = (Arc<Mutex<P2PManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || p2p_manager.clone())
}

async fn submit_zk_snark_proof_handler(
    request: ZkSnarkProofRequest,
    p2p_manager: Arc<Mutex<P2PManager>>, // Add p2p_manager parameter
) -> Result<impl warp::Reply, warp::Rejection> {
    // Publish event
    let event = ReputationEvent::ZkSnarkProofSubmitted {
        proof: request.proof.clone(),
    };
    let mut p2p = p2p_manager.lock().await;
    p2p.publish(event).await.unwrap();

    // Placeholder logic for zk-SNARK proof submission
    Ok(warp::reply::json(&"zk-SNARK proof submitted"))
}

async fn apply_reputation_decay_handler(
    request: ReputationDecayRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Placeholder logic for applying reputation decay
    Ok(warp::reply::json(&"Reputation decay applied"))
}

async fn handle_sybil_resistance_handler(
    request: SybilResistanceRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Placeholder logic for handling sybil resistance
    Ok(warp::reply::json(&"Sybil resistance handled"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Filter;
    use crate::services::reputation_service::{ReputationService, ReputationServiceImpl};
    use std::sync::Arc;
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_submit_zk_snark_proof() {
        let reputation_service = Arc::new(ReputationServiceImpl::new(Arc::new(Database::new())));
        let p2p_manager = Arc::new(Mutex::new(P2PManager::new())); // Add P2PManager instance
        let api = reputation_routes(p2p_manager);

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/reputation/zk_snark_proof")
            .json(&ZkSnarkProofRequest { proof: "test_proof".to_string() })
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_apply_reputation_decay() {
        let reputation_service = Arc::new(ReputationServiceImpl::new(Arc::new(Database::new())));
        let api = reputation_routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/reputation/reputation_decay")
            .json(&ReputationDecayRequest { did: "did:icn:test".to_string(), decay_rate: 0.1 })
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_sybil_resistance() {
        let reputation_service = Arc::new(ReputationServiceImpl::new(Arc::new(Database::new())));
        let api = reputation_routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/reputation/sybil_resistance")
            .json(&SybilResistanceRequest { did: "did:icn:test".to_string(), reputation_score: 50 })
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
