use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use icn_p2p::sdp::{SDPManager};
use x25519_dalek::PublicKey;
use icn_federation::{FederationError, Federation};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use tokio::time::Duration;
use serde_json::json;

// Example event types we might want to securely exchange between federations
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FederationSDPEvent {
    ProposalSubmitted {
        proposal_id: String,
        proposer_did: String,
        title: String,
        description: String,
    },
    VoteCast {
        proposal_id: String,
        voter_did: String,
        approve: bool,
        timestamp: u64,
    },
    ResourceShared {
        resource_id: String,
        provider_federation: String,
        consumer_federation: String,
        amount: u64,
    },
    DisputeInitiated {
        dispute_id: String,
        initiator_did: String,
        reason: String,
    },
}

// Example showing how to integrate SDP with federation communications
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ICN Secure Datagram Protocol Federation Example");
    
    // Initialize SDP managers for two example federations
    let fed1_sdp = Arc::new(Mutex::new(SDPManager::new("127.0.0.1:8081".parse::<SocketAddr>()?)?));
    let fed2_sdp = Arc::new(Mutex::new(SDPManager::new("127.0.0.1:8082".parse::<SocketAddr>()?)?));
    
    // Get public keys for key exchange
    let fed1_pubkey = PublicKey::from([0u8; 32]);
    let fed2_pubkey = PublicKey::from([0u8; 32]);
    
    // Register each federation with the other
    {
        let mut fed1_manager = fed1_sdp.lock().unwrap();
        fed1_manager.register(
            "federation1".to_string(),
            "127.0.0.1:8081".parse::<SocketAddr>()?,
            PublicKey::from([0u8; 32]),
        );
    }
    
    {
        let mut fed2_manager = fed2_sdp.lock().unwrap();
        fed2_manager.register(
            "federation2".to_string(),
            "127.0.0.1:8082".parse::<SocketAddr>()?,
            PublicKey::from([0u8; 32]),
        );
    }
    
    // Start SDP receivers
    {
        let fed1_sdp_clone = Arc::clone(&fed1_sdp);
        tokio::spawn(async move {
            let handler = |msg: Result<(Vec<u8>, SocketAddr, String), String>| {
                match msg {
                    Ok((data, _addr, _id)) => println!("Received message: {:?}", data),
                    Err(e) => eprintln!("Error receiving message: {}", e),
                }
            };
            if let Err(e) = fed1_sdp_clone.lock().unwrap().start(handler).await {
                eprintln!("Error in fed1 receiver: {}", e);
            }
        });
    }
    
    {
        let fed2_sdp_clone = Arc::clone(&fed2_sdp);
        tokio::spawn(async move {
            let handler = |msg: Result<(Vec<u8>, SocketAddr, String), String>| {
                match msg {
                    Ok((data, _addr, _id)) => println!("Received message: {:?}", data),
                    Err(e) => eprintln!("Error receiving message: {}", e),
                }
            };
            if let Err(e) = fed2_sdp_clone.lock().unwrap().start(handler).await {
                eprintln!("Error in fed2 receiver: {}", e);
            }
        });
    }
    
    // Example: Federation 1 sends a secure proposal to Federation 2
    let proposal_event = FederationSDPEvent::ProposalSubmitted {
        proposal_id: "prop-123".to_string(),
        proposer_did: "did:icn:fed1:12345".to_string(),
        title: "Cross-Federation Resource Sharing".to_string(),
        description: "Proposal to establish shared computing resources".to_string(),
    };
    
    // Serialize the event
    let event_json = serde_json::to_vec(&proposal_event)?;
    
    // Send via SDP - encrypted, integrity-protected, and potentially via multiple paths
    {
        let fed1_sdp_locked = fed1_sdp.lock().unwrap();
        fed1_sdp_locked.send_message("federation2", event_json.clone(), 8).await?;
    }
    
    println!("Proposal sent securely via SDP");
    
    // Example: Federation 2 sends a secure vote back to Federation 1
    let vote_event = FederationSDPEvent::VoteCast {
        proposal_id: "prop-123".to_string(),
        voter_did: "did:icn:fed2:67890".to_string(),
        approve: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    
    // Serialize the event
    let event_json = serde_json::to_vec(&vote_event)?;
    
    // Send via SDP
    {
        let fed2_sdp_locked = fed2_sdp.lock().unwrap();
        fed2_sdp_locked.send_message("federation1", event_json, 8).await?;
    }
    
    println!("Vote sent securely via SDP");
    
    // Keep the program running to demonstrate message exchange
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    println!("SDP Federation Example completed");
    
    Ok(())
}