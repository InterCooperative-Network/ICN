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

#[derive(Debug, Deserialize)]
struct RecallVoteRequest {
    target_member: String,
    voter: String,
    approve: bool,
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

    let sybil_resistance = warp::path!("api" / "v1" / "governance" / "sybil_resistance")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_governance_service(governance_service.clone()))
        .and_then(sybil_resistance_handler);

    let reputation_decay = warp::path!("api" / "v1" / "governance" / "reputation_decay")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_governance_service(governance_service.clone()))
        .and_then(reputation_decay_handler);

    let proposal_status = warp::path!("api" / "v1" / "governance" / "proposals" / String / "status")
        .and(warp::get())
        .and(with_governance_service(governance_service.clone()))
        .and_then(proposal_status_handler);

    create_proposal
        .or(vote_on_proposal)
        .or(sybil_resistance)
        .or(reputation_decay)
        .or(proposal_status)
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

async fn sybil_resistance_handler(
    request: SybilResistanceRequest,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut service = governance_service.lock().await;
    match service.handle_sybil_resistance(request.did, request.reputation_score).await {
        Ok(_) => Ok(warp::reply::json(&"Sybil resistance applied")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

async fn reputation_decay_handler(
    request: ReputationDecayRequest,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut service = governance_service.lock().await;
    match service.apply_reputation_decay(request.did, request.decay_rate).await {
        Ok(_) => Ok(warp::reply::json(&"Reputation decay applied")),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

async fn proposal_status_handler(
    proposal_id: String,
    governance_service: Arc<Mutex<GovernanceService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let service = governance_service.lock().await;
    match service.get_proposal_status(&proposal_id).await {
        Ok(status) => Ok(warp::reply::json(&status)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
