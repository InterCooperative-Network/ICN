use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::services::governance_service::{GovernanceService, Proposal, Vote};

#[derive(Debug, Deserialize, Serialize)]
struct CreateProposalRequest {
    title: String,
    description: String,
    created_by: String,
    ends_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct VoteRequest {
    proposal_id: String,
    voter: String,
    approve: bool,
    zk_snark_proof: String, // Added zk-SNARK proof field
}

pub fn governance_routes(
    governance_service: Arc<Mutex<GovernanceService>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let create_proposal = warp::path!("api" / "v1" / "governance" / "proposals")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_governance_service(governance_service.clone()))
        .and_then(create_proposal_handler);

    let vote_on_proposal = warp::path!("api" / "v1" / "governance" / "proposals" / String / "vote")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_governance_service(governance_service.clone()))
        .and_then(vote_on_proposal_handler);

    let submit_zk_snark_proof = warp::path!("api" / "v1" / "governance" / "zk_snark_proof")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_governance_service(governance_service.clone()))
        .and_then(submit_zk_snark_proof_handler);

    create_proposal.or(vote_on_proposal).or(submit_zk_snark_proof)
}

fn with_governance_service(
    governance_service: Arc<Mutex<GovernanceService>>,
) -> impl Filter<Extract = (Arc<Mutex<GovernanceService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || governance_service.clone())
}

async fn create_proposal_handler(
    request: CreateProposalRequest,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let proposal = Proposal {
        title: request.title,
        description: request.description,
        created_by: request.created_by,
        ends_at: request.ends_at,
    };

    let mut service = governance_service.lock().await;
    match service.create_proposal(proposal).await {
        Ok(proposal_id) => Ok(warp::reply::json(&proposal_id)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

async fn vote_on_proposal_handler(
    request: VoteRequest,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let vote = Vote {
        proposal_id: request.proposal_id,
        voter: request.voter,
        approve: request.approve,
        zk_snark_proof: request.zk_snark_proof, // Added zk-SNARK proof field
    };

    let mut service = governance_service.lock().await;
    match service.record_vote(vote).await {
        Ok(_) => Ok(warp::reply::json(&"Vote recorded")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ZkSnarkProofRequest {
    proof: String,
}

async fn submit_zk_snark_proof_handler(
    request: ZkSnarkProofRequest,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut service = governance_service.lock().await;
    match service.submit_zk_snark_proof(request.proof).await {
        Ok(_) => Ok(warp::reply::json(&"zk-SNARK proof submitted")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
