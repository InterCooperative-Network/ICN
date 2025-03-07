use backend::{
    test_utils::TestServices,
    test_macros::*,
    models::{Proposal, Vote, Federation, FederationType, ResourceAllocation},
};

use chrono::{Duration, Utc};
use serde_json::json;
use tokio::time::sleep;
use warp::http::StatusCode;
use warp::test::request;

// Helper function to create a test server
async fn setup_test_server() -> (TestServices, String) {
    let services = TestServices::new().await;
    let routes = backend::create_routes(services.clone());
    let server = warp::serve(routes);
    let (addr, server) = server.bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);
    (services, format!("http://127.0.0.1:{}", addr.port()))
}

// E2E test: Complete federation lifecycle
async_test!(test_federation_lifecycle, |services| async {
    let (services, base_url) = setup_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create a federation
    let federation_response = client.post(&format!("{}/api/v1/federation/create", base_url))
        .json(&json!({
            "name": "Test Federation",
            "federation_type": "Cooperative",
            "terms": {
                "minimum_reputation": 50,
                "resource_sharing_policies": "Test policies",
                "governance_rules": "Test rules",
                "duration": "1 year"
            }
        }))
        .send()
        .await?;

    assert_eq!(federation_response.status(), StatusCode::OK);
    let federation: Federation = federation_response.json().await?;

    // 2. Join federation
    let join_response = client.post(&format!("{}/api/v1/federation/{}/join", base_url, federation.id))
        .json(&json!({
            "member_did": "did:icn:test",
            "commitment": "Test commitment"
        }))
        .send()
        .await?;

    assert_eq!(join_response.status(), StatusCode::OK);

    // 3. Create and vote on a proposal
    let proposal_response = client.post(&format!("{}/api/v1/federation/{}/proposals", base_url, federation.id))
        .json(&json!({
            "title": "Test Proposal",
            "description": "Test Description",
            "created_by": "did:icn:test",
            "ends_at": (Utc::now() + Duration::hours(24)).to_rfc3339()
        }))
        .send()
        .await?;

    assert_eq!(proposal_response.status(), StatusCode::OK);
    let proposal: Proposal = proposal_response.json().await?;

    // 4. Vote on proposal
    let vote_response = client.post(&format!("{}/api/v1/federation/{}/proposals/{}/vote", 
        base_url, federation.id, proposal.id))
        .json(&json!({
            "voter": "did:icn:test",
            "approve": true
        }))
        .send()
        .await?;

    assert_eq!(vote_response.status(), StatusCode::OK);

    // 5. Share resources
    let resource_response = client.post(&format!("{}/api/v1/federation/{}/resources/share", base_url, federation.id))
        .json(&json!({
            "resource_type": "TestResource",
            "amount": 100,
            "recipient_id": "did:icn:recipient"
        }))
        .send()
        .await?;

    assert_eq!(resource_response.status(), StatusCode::OK);

    // 6. Verify reputation changes
    sleep(Duration::from_secs(1)).await; // Wait for reputation updates
    let reputation_response = client.get(&format!("{}/api/v1/reputation/did:icn:test", base_url))
        .send()
        .await?;

    assert_eq!(reputation_response.status(), StatusCode::OK);
    let reputation: i64 = reputation_response.json().await?;
    assert!(reputation > 0, "Reputation should increase after participation");

    // 7. Dissolve federation
    let dissolution_response = client.post(&format!("{}/api/v1/federation/{}/dissolve", base_url, federation.id))
        .json(&json!({
            "reason": "Test dissolution",
            "initiator": "did:icn:test"
        }))
        .send()
        .await?;

    assert_eq!(dissolution_response.status(), StatusCode::OK);

    Ok(())
});

// E2E test: Resource allocation and management
async_test!(test_resource_management, |services| async {
    let (services, base_url) = setup_test_server().await;
    let client = reqwest::Client::new();

    // 1. Register a resource
    let register_response = client.post(&format!("{}/api/v1/resources/register", base_url))
        .json(&json!({
            "resource_type": "ComputeResource",
            "amount": 1000,
            "owner": "did:icn:test",
            "metadata": {
                "cpu_cores": 4,
                "memory_gb": 16
            }
        }))
        .send()
        .await?;

    assert_eq!(register_response.status(), StatusCode::OK);
    let resource_id: String = register_response.json().await?;

    // 2. Create allocation request
    let allocation_response = client.post(&format!("{}/api/v1/resources/{}/allocate", base_url, resource_id))
        .json(&json!({
            "requester": "did:icn:requester",
            "amount": 100,
            "duration_hours": 24
        }))
        .send()
        .await?;

    assert_eq!(allocation_response.status(), StatusCode::OK);
    let allocation: ResourceAllocation = allocation_response.json().await?;

    // 3. Verify allocation
    let status_response = client.get(&format!("{}/api/v1/resources/allocations/{}", base_url, allocation.id))
        .send()
        .await?;

    assert_eq!(status_response.status(), StatusCode::OK);
    let status: serde_json::Value = status_response.json().await?;
    assert_eq!(status["status"], "active");

    // 4. Release allocation
    let release_response = client.post(&format!("{}/api/v1/resources/allocations/{}/release", base_url, allocation.id))
        .send()
        .await?;

    assert_eq!(release_response.status(), StatusCode::OK);

    Ok(())
});

// E2E test: Identity and reputation system
async_test!(test_identity_and_reputation, |services| async {
    let (services, base_url) = setup_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create identity
    let identity_response = client.post(&format!("{}/api/v1/identity/create", base_url))
        .json(&json!({
            "did": "did:icn:newuser",
            "public_key": "test_public_key",
            "metadata": {
                "name": "Test User",
                "email": "test@example.com"
            }
        }))
        .send()
        .await?;

    assert_eq!(identity_response.status(), StatusCode::OK);

    // 2. Record contribution
    let contribution_response = client.post(&format!("{}/api/v1/reputation/record_contribution", base_url))
        .json(&json!({
            "did": "did:icn:newuser",
            "contribution_type": "CodeContribution",
            "details": "Implemented new feature",
            "proof": "commit_hash_123"
        }))
        .send()
        .await?;

    assert_eq!(contribution_response.status(), StatusCode::OK);

    // 3. Verify reputation increase
    sleep(Duration::from_secs(1)).await; // Wait for reputation update
    let reputation_response = client.get(&format!("{}/api/v1/reputation/did:icn:newuser", base_url))
        .send()
        .await?;

    assert_eq!(reputation_response.status(), StatusCode::OK);
    let reputation: i64 = reputation_response.json().await?;
    assert!(reputation > 0, "Reputation should increase after contribution");

    // 4. Generate and verify zk-SNARK proof
    let proof_response = client.post(&format!("{}/api/v1/reputation/generate_proof", base_url))
        .json(&json!({
            "did": "did:icn:newuser",
            "minimum_reputation": 10
        }))
        .send()
        .await?;

    assert_eq!(proof_response.status(), StatusCode::OK);
    let proof: String = proof_response.json().await?;

    let verify_response = client.post(&format!("{}/api/v1/reputation/verify_proof", base_url))
        .json(&json!({
            "proof": proof,
            "minimum_reputation": 10
        }))
        .send()
        .await?;

    assert_eq!(verify_response.status(), StatusCode::OK);
    let is_valid: bool = verify_response.json().await?;
    assert!(is_valid, "Proof should be valid");

    Ok(())
});

// E2E test: Governance proposal lifecycle
async_test!(test_governance_proposal_lifecycle, |services| async {
    let (services, base_url) = setup_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create a proposal
    let proposal_response = client.post(&format!("{}/api/v1/governance/proposals", base_url))
        .json(&json!({
            "title": "Test Proposal",
            "description": "Test Description",
            "created_by": "did:icn:test",
            "ends_at": (Utc::now() + Duration::hours(24)).to_rfc3339()
        }))
        .send()
        .await?;

    assert_eq!(proposal_response.status(), StatusCode::OK);
    let proposal: Proposal = proposal_response.json().await?;

    // 2. Vote on the proposal
    let vote_response = client.post(&format!("{}/api/v1/governance/proposals/{}/vote", base_url, proposal.id))
        .json(&json!({
            "voter": "did:icn:test",
            "approve": true
        }))
        .send()
        .await?;

    assert_eq!(vote_response.status(), StatusCode::OK);

    // 3. Verify proposal status
    let status_response = client.get(&format!("{}/api/v1/governance/proposals/{}", base_url, proposal.id))
        .send()
        .await?;

    assert_eq!(status_response.status(), StatusCode::OK);
    let status: serde_json::Value = status_response.json().await?;
    assert_eq!(status["status"], "approved");

    Ok(())
});

// E2E test: Blockchain operations
async_test!(test_blockchain_operations, |services| async {
    let (services, base_url) = setup_test_server().await;
    let client = reqwest::Client::new();

    // 1. Add a block
    let block_response = client.post(&format!("{}/api/v1/blockchain/blocks", base_url))
        .json(&json!({
            "index": 1,
            "previous_hash": "previous_hash",
            "timestamp": Utc::now().to_rfc3339(),
            "transactions": [],
            "hash": "test_hash",
            "proposer": "did:icn:test"
        }))
        .send()
        .await?;

    assert_eq!(block_response.status(), StatusCode::OK);

    // 2. Verify blockchain consistency
    let verify_response = client.get(&format!("{}/api/v1/blockchain/verify", base_url))
        .send()
        .await?;

    assert_eq!(verify_response.status(), StatusCode::OK);
    let is_valid: bool = verify_response.json().await?;
    assert!(is_valid, "Blockchain should be valid");

    Ok(())
});
