use reqwest;
use serde_json::json;
use std::env;

#[tokio::test]
async fn test_health_endpoint() {
    let api_url = env::var("TEST_API_URL").expect("TEST_API_URL must be set");
    let client = reqwest::Client::new();
    
    let response = client.get(format!("{}/health", api_url))
        .send()
        .await
        .expect("Failed to execute request");
        
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_basic_federation() {
    let api_url = env::var("TEST_API_URL").expect("TEST_API_URL must be set");
    let api_key = env::var("TEST_API_KEY").expect("TEST_API_KEY must be set");
    let client = reqwest::Client::new();
    
    // Test federation node registration
    let response = client.post(format!("{}/api/v1/federation/register", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "node_id": "test-node",
            "endpoint": "https://test-node.example.com"
        }))
        .send()
        .await
        .expect("Failed to execute request");
        
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_basic_resource() {
    let api_url = env::var("TEST_API_URL").expect("TEST_API_URL must be set");
    let api_key = env::var("TEST_API_KEY").expect("TEST_API_KEY must be set");
    let client = reqwest::Client::new();
    
    // Test resource registration
    let response = client.post(format!("{}/api/v1/resources", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "resource_type": "compute",
            "capacity": 100,
            "metadata": {
                "location": "test-dc"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");
        
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_basic_governance() {
    let api_url = env::var("TEST_API_URL").expect("TEST_API_URL must be set");
    let api_key = env::var("TEST_API_KEY").expect("TEST_API_KEY must be set");
    let client = reqwest::Client::new();
    
    // Test proposal creation
    let response = client.post(format!("{}/api/v1/governance/proposals", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "title": "Test Proposal",
            "description": "Smoke test proposal",
            "type": "configuration",
            "changes": {
                "parameter": "min_nodes",
                "value": 3
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");
        
    assert!(response.status().is_success());
}