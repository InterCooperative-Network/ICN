use crate::networking::p2p::{FederationManager, FederationMessageType, MessagePriority, SDPManager, PublicKey};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde_json::json;
use tokio::runtime::Runtime;

#[tokio::test]
async fn test_initialize_sdp() {
    let p2p_manager = Arc::new(Mutex::new(P2PManager::new()));
    let mut federation_manager = FederationManager::new(p2p_manager, "federation_1".to_string());
    let result = federation_manager.initialize_sdp("127.0.0.1:8080").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_peer_federation() {
    let p2p_manager = Arc::new(Mutex::new(P2PManager::new()));
    let mut federation_manager = FederationManager::new(p2p_manager, "federation_1".to_string());
    federation_manager.initialize_sdp("127.0.0.1:8080").await.unwrap();
    let public_key = PublicKey;
    let addresses = vec!["127.0.0.1:8081".parse::<SocketAddr>().unwrap()];
    let result = federation_manager.register_peer_federation("federation_2".to_string(), public_key, addresses).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_federation_message() {
    let p2p_manager = Arc::new(Mutex::new(P2PManager::new()));
    let mut federation_manager = FederationManager::new(p2p_manager, "federation_1".to_string());
    federation_manager.initialize_sdp("127.0.0.1:8080").await.unwrap();
    let public_key = PublicKey;
    let addresses = vec!["127.0.0.1:8081".parse::<SocketAddr>().unwrap()];
    federation_manager.register_peer_federation("federation_2".to_string(), public_key, addresses).await.unwrap();
    let payload = json!({ "key": "value" });
    let signature = vec![0u8; 64];
    let result = federation_manager.send_federation_message("federation_1", "federation_2", FederationMessageType::GeneralCommunication, payload, signature).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_verify_federation_message() {
    let p2p_manager = Arc::new(Mutex::new(P2PManager::new()));
    let mut federation_manager = FederationManager::new(p2p_manager, "federation_1".to_string());
    federation_manager.initialize_sdp("127.0.0.1:8080").await.unwrap();
    let public_key = PublicKey;
    let addresses = vec!["127.0.0.1:8081".parse::<SocketAddr>().unwrap()];
    federation_manager.register_peer_federation("federation_2".to_string(), public_key, addresses).await.unwrap();
    let message = vec![0u8; 128];
    let signature = vec![0u8; 64];
    let result = federation_manager.verify_federation_message("federation_2", &message, &signature).await;
    assert!(result.is_ok());
}
