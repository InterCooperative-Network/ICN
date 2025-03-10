use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use icn_p2p::sdp::{SDPManager, PublicKey};
use icn_types::{Block, Transaction, FederationType};
use icn_federation::{Federation, FederationManager, FederationError};
use serde_json::{json, Value};
use icn_crypto::KeyPair;

const TEST_BIND_ADDR: &str = "127.0.0.1:0"; // Let OS assign port

#[tokio::test]
async fn test_sdp_initialization() {
    let resource_manager = Arc::new(MockResourceManager::new());
    let mut federation_manager = FederationManager::new(resource_manager);

    let sdp_config = icn_federation::SDPConfig {
        bind_address: TEST_BIND_ADDR.to_string(),
        enable_multipath: true,
        enable_onion_routing: false,
        message_priority: Default::default(),
    };

    let result = federation_manager.init_sdp(sdp_config).await;
    assert!(result.is_ok(), "Failed to initialize SDP: {:?}", result);
}

#[tokio::test]
async fn test_federation_sdp_communication() {
    // Initialize two federation managers
    let resource_manager1 = Arc::new(MockResourceManager::new());
    let resource_manager2 = Arc::new(MockResourceManager::new());
    
    let mut federation1 = FederationManager::new(resource_manager1);
    let mut federation2 = FederationManager::new(resource_manager2);

    // Initialize SDP for both federations
    let sdp_config1 = icn_federation::SDPConfig {
        bind_address: "127.0.0.1:0".to_string(),
        enable_multipath: true,
        enable_onion_routing: false,
        message_priority: Default::default(),
    };
    
    let sdp_config2 = icn_federation::SDPConfig {
        bind_address: "127.0.0.1:0".to_string(),
        enable_multipath: true,
        enable_onion_routing: false,
        message_priority: Default::default(),
    };

    federation1.init_sdp(sdp_config1).await.unwrap();
    federation2.init_sdp(sdp_config2).await.unwrap();

    // Create federations
    let fed1_id = federation1.create_federation(
        "Federation 1".to_string(),
        FederationType::Cooperative,
        Default::default(),
        "did:icn:fed1".to_string(),
    ).await.unwrap();

    let fed2_id = federation2.create_federation(
        "Federation 2".to_string(),
        FederationType::Cooperative,
        Default::default(),
        "did:icn:fed2".to_string(),
    ).await.unwrap();

    // Register each federation with the other
    federation1.register_peer_federation(
        fed2_id.clone(),
        federation2.get_public_key().await.unwrap(),
        vec!["127.0.0.1:8082".parse::<SocketAddr>().unwrap()]
    ).await.unwrap();

    federation2.register_peer_federation(
        fed1_id.clone(),
        federation1.get_public_key().await.unwrap(),
        vec!["127.0.0.1:8081".parse::<SocketAddr>().unwrap()]
    ).await.unwrap();

    // Test sending a message from federation1 to federation2
    let test_message = json!({
        "type": "proposal",
        "content": "Cross-federation resource sharing proposal",
        "timestamp": chrono::Utc::now().timestamp()
    });

    let result = federation1.send_federation_message(
        &fed1_id,
        &fed2_id,
        icn_federation::FederationMessageType::ProposalSubmission,
        test_message.clone(),
        "test_signature"
    ).await;

    assert!(result.is_ok(), "Failed to send federation message: {:?}", result);

    // Give some time for message processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_federation_message_encryption() {
    let resource_manager = Arc::new(MockResourceManager::new());
    let mut federation_manager = FederationManager::new(resource_manager);

    let sdp_config = icn_federation::SDPConfig {
        bind_address: TEST_BIND_ADDR.to_string(),
        enable_multipath: true,
        enable_onion_routing: false,
        message_priority: Default::default(),
    };

    federation_manager.init_sdp(sdp_config).await.unwrap();

    // Create test federation
    let fed_id = federation_manager.create_federation(
        "Test Federation".to_string(),
        FederationType::Cooperative,
        Default::default(),
        "did:icn:test".to_string(),
    ).await.unwrap();

    // Get federation's public key
    let public_key = federation_manager.get_public_key().await.unwrap();

    // Test message encryption/decryption
    let test_message = json!({
        "sensitive_data": "test_value",
        "timestamp": chrono::Utc::now().timestamp()
    });

    let result = federation_manager.encrypt_federation_message(
        &fed_id,
        &test_message
    ).await;

    assert!(result.is_ok(), "Failed to encrypt federation message");
    
    let encrypted = result.unwrap();
    assert_ne!(encrypted, test_message.to_string(), "Message was not encrypted");

    let decrypted = federation_manager.decrypt_federation_message(
        &fed_id,
        &encrypted
    ).await.unwrap();

    assert_eq!(test_message, decrypted, "Decrypted message does not match original");
}

#[tokio::test]
async fn test_federation_message_verification() {
    let resource_manager = Arc::new(MockResourceManager::new());
    let mut federation_manager = FederationManager::new(resource_manager);

    let sdp_config = icn_federation::SDPConfig {
        bind_address: TEST_BIND_ADDR.to_string(),
        enable_multipath: true,
        enable_onion_routing: false,
        message_priority: Default::default(),
    };

    federation_manager.init_sdp(sdp_config).await.unwrap();

    // Create test federation
    let fed_id = federation_manager.create_federation(
        "Test Federation".to_string(),
        FederationType::Cooperative,
        Default::default(),
        "did:icn:test".to_string(),
    ).await.unwrap();

    // Test message signing and verification
    let test_message = "Test message content".as_bytes();
    let signature = federation_manager.sign_federation_message(
        &fed_id,
        test_message
    ).await.unwrap();

    let verification = federation_manager.verify_federation_message(
        &fed_id,
        test_message,
        &signature
    ).await;

    assert!(verification.is_ok(), "Message verification failed");
    assert!(verification.unwrap(), "Message signature is invalid");
}

// Mock Resource Manager for testing
struct MockResourceManager;

impl MockResourceManager {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl icn_federation::ResourceManager for MockResourceManager {
    async fn allocate_resources(&self, _amount: u64) -> Result<(), String> {
        Ok(())
    }

    async fn release_resources(&self, _amount: u64) -> Result<(), String> {
        Ok(())
    }

    async fn get_available_resources(&self) -> u64 {
        1000
    }
}

struct TestContext {
    federation1: FederationManager,
    federation2: FederationManager,
    sdp1: Arc<SDPManager>,
    sdp2: Arc<SDPManager>,
}

async fn setup_test_context() -> TestContext {
    let keypair1 = KeyPair::generate().unwrap();
    let keypair2 = KeyPair::generate().unwrap();

    let sdp1 = Arc::new(SDPManager::new(
        "127.0.0.1:0".parse().unwrap(),
        keypair1.clone(),
    ));

    let sdp2 = Arc::new(SDPManager::new(
        "127.0.0.1:0".parse().unwrap(),
        keypair2.clone(),
    ));

    let federation1 = FederationManager::new(sdp1.clone());
    let federation2 = FederationManager::new(sdp2.clone());

    TestContext {
        federation1,
        federation2,
        sdp1,
        sdp2,
    }
}

#[tokio::test]
async fn test_federation_sdp_connection() {
    let ctx = setup_test_context().await;
    
    // Create federations
    let fed1_id = ctx.federation1.create_federation(
        "Federation 1",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Federation 2", 
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    // Exchange connection info
    let fed1_info = ctx.federation1.get_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Verify connection
    assert!(ctx.federation1.is_connected_to(&fed2_id).await);
    assert!(ctx.federation2.is_connected_to(&fed1_id).await);
}

#[tokio::test]
async fn test_federation_message_exchange() {
    let ctx = setup_test_context().await;

    // Create federations
    let fed1_id = ctx.federation1.create_federation(
        "Federation 1",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Federation 2",
        FederationType::Cooperative, 
        None,
    ).await.unwrap();

    // Connect federations
    let fed1_info = ctx.federation1.get_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Send test message
    let test_message = "Hello Federation 2!";
    ctx.federation1.send_message(&fed1_id, &fed2_id, test_message.as_bytes()).await.unwrap();

    // Wait for message processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify message received
    let received = ctx.federation2.get_last_message(&fed2_id).await.unwrap();
    assert_eq!(received, test_message.as_bytes());
}

#[tokio::test]
async fn test_federation_encrypted_communication() {
    let ctx = setup_test_context().await;

    // Create federations with encryption enabled
    let encryption_config = icn_federation::EncryptionConfig {
        algorithm: "AES-256-GCM",
        key_size: 256,
    };

    let fed1_id = ctx.federation1.create_federation(
        "Encrypted Federation 1",
        FederationType::Cooperative,
        Some(encryption_config.clone()),
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Encrypted Federation 2",
        FederationType::Cooperative,
        Some(encryption_config),
    ).await.unwrap();

    // Exchange connection info with encryption
    let fed1_info = ctx.federation1.get_encrypted_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_encrypted_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Send encrypted message
    let sensitive_data = "Secret federation data";
    ctx.federation1.send_encrypted_message(
        &fed1_id,
        &fed2_id,
        sensitive_data.as_bytes()
    ).await.unwrap();

    // Wait for message processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify decrypted message
    let received = ctx.federation2.get_decrypted_message(&fed2_id).await.unwrap();
    assert_eq!(received, sensitive_data.as_bytes());
}

#[tokio::test]
async fn test_federation_resource_sharing() {
    let ctx = setup_test_context().await;

    // Create federations
    let fed1_id = ctx.federation1.create_federation(
        "Resource Federation 1",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Resource Federation 2",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    // Connect federations
    let fed1_info = ctx.federation1.get_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Create resource sharing agreement
    let agreement = icn_federation::ResourceAgreement {
        federation_id: fed1_id.clone(),
        resource_type: "compute".to_string(),
        amount: 1000,
        duration: Duration::from_secs(3600),
    };

    // Share resources
    ctx.federation1.share_resources(&fed1_id, &fed2_id, &agreement).await.unwrap();

    // Verify resource allocation
    let allocation = ctx.federation2.get_resource_allocation(&fed2_id).await.unwrap();
    assert_eq!(allocation.available_resources, 1000);
    assert_eq!(allocation.resource_type, "compute");
}

#[tokio::test]
async fn test_federation_consensus() {
    let ctx = setup_test_context().await;

    // Create federations
    let fed1_id = ctx.federation1.create_federation(
        "Consensus Federation 1",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Consensus Federation 2",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    // Connect federations
    let fed1_info = ctx.federation1.get_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Create cross-federation proposal
    let proposal = icn_federation::Proposal {
        id: "test_proposal".to_string(),
        federation_id: fed1_id.clone(),
        content: "Increase resource sharing limit".to_string(),
        created_at: chrono::Utc::now(),
    };

    // Submit proposal
    ctx.federation1.submit_proposal(&fed1_id, proposal.clone()).await.unwrap();

    // Vote on proposal
    ctx.federation2.vote_on_proposal(&fed2_id, &proposal.id, true).await.unwrap();

    // Wait for consensus
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify proposal status
    let status = ctx.federation1.get_proposal_status(&fed1_id, &proposal.id).await.unwrap();
    assert_eq!(status, icn_federation::ProposalStatus::Accepted);
}

#[tokio::test]
async fn test_federation_disconnection() {
    let ctx = setup_test_context().await;

    // Create and connect federations
    let fed1_id = ctx.federation1.create_federation(
        "Federation 1",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed2_id = ctx.federation2.create_federation(
        "Federation 2",
        FederationType::Cooperative,
        None,
    ).await.unwrap();

    let fed1_info = ctx.federation1.get_connection_info(&fed1_id).await.unwrap();
    let fed2_info = ctx.federation2.get_connection_info(&fed2_id).await.unwrap();

    ctx.federation1.connect_to_federation(&fed2_id, fed2_info).await.unwrap();
    ctx.federation2.connect_to_federation(&fed1_id, fed1_info).await.unwrap();

    // Verify initial connection
    assert!(ctx.federation1.is_connected_to(&fed2_id).await);
    assert!(ctx.federation2.is_connected_to(&fed1_id).await);

    // Disconnect federation 1
    ctx.federation1.disconnect_from_federation(&fed2_id).await.unwrap();

    // Verify disconnection
    assert!(!ctx.federation1.is_connected_to(&fed2_id).await);
    assert!(!ctx.federation2.is_connected_to(&fed1_id).await);

    // Attempt to send message after disconnection
    let result = ctx.federation1.send_message(
        &fed1_id,
        &fed2_id,
        "Should fail".as_bytes()
    ).await;

    assert!(result.is_err());
}