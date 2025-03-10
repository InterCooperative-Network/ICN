use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use log::{debug, trace};

pub struct IcnClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identity {
    pub did: String,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cooperative {
    pub id: String,
    pub name: String,
    pub member_count: u32,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub capacity: String,
    pub owner: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub votes_yes: u32,
    #[serde(default)]
    pub votes_no: u32,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub ends_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    #[serde(default)]
    pub node_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub status: String,
    pub peer_count: u32,
    pub avg_latency: u32,
    pub bandwidth_usage: f32,
    #[serde(default)]
    pub uptime: Option<u64>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkPeer {
    pub id: String,
    pub address: String,
    pub latency: u32,
    pub connected_since: String,
    #[serde(default = "default_connected_status")]
    pub status: String,
}

fn default_connected_status() -> String {
    "connected".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeersResponse {
    pub peers: Vec<NetworkPeer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentitiesResponse {
    pub identities: Vec<Identity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CooperativesResponse {
    pub cooperatives: Vec<Cooperative>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcesResponse {
    pub resources: Vec<Resource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    pub peer_id: String,
    pub latency: u32,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl IcnClient {
    pub fn new(base_url: String) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
        }
    }

    pub fn with_timeout(base_url: String, timeout_seconds: u64) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_seconds))
            .connect_timeout(Duration::from_secs(timeout_seconds / 2))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
        }
    }

    pub async fn check_health(&self) -> Result<HealthStatus, Box<dyn Error>> {
        debug!("Checking API health at {}/api/v1/health", self.base_url);
        let response = self.client
            .get(&format!("{}/api/v1/health", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let health: HealthStatus = response.json().await?;
            trace!("Health response: {:?}", health);
            Ok(health)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API health check failed with status: {} - {}", status, error_text).into())
        }
    }

    pub async fn create_identity(&self) -> Result<Identity, Box<dyn Error>> {
        debug!("Creating new identity");
        let response = self.client
            .post(&format!("{}/api/v1/identities", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let identity = response_json["identity"].clone();
            let identity: Identity = serde_json::from_value(identity)?;
            trace!("Created identity: {:?}", identity);
            Ok(identity)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to create identity: {} - {}", status, error_text).into())
        }
    }

    pub async fn list_identities(&self) -> Result<Vec<Identity>, Box<dyn Error>> {
        debug!("Listing identities");
        let response = self.client
            .get(&format!("{}/api/v1/identities", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let identities: IdentitiesResponse = response.json().await?;
            trace!("Found {} identities", identities.identities.len());
            Ok(identities.identities)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to list identities: {} - {}", status, error_text).into())
        }
    }

    pub async fn join_cooperative(&self, coop_id: &str) -> Result<(), Box<dyn Error>> {
        debug!("Joining cooperative: {}", coop_id);
        let response = self.client
            .post(&format!("{}/api/v1/cooperatives/join", self.base_url))
            .json(&serde_json::json!({ "cooperative_id": coop_id }))
            .send()
            .await?;
            
        if response.status().is_success() {
            trace!("Successfully joined cooperative {}", coop_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to join cooperative: {} - {}", status, error_text).into())
        }
    }

    pub async fn list_cooperatives(&self) -> Result<Vec<Cooperative>, Box<dyn Error>> {
        debug!("Listing cooperatives");
        let response = self.client
            .get(&format!("{}/api/v1/cooperatives", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let cooperatives: CooperativesResponse = response.json().await?;
            trace!("Found {} cooperatives", cooperatives.cooperatives.len());
            Ok(cooperatives.cooperatives)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to list cooperatives: {} - {}", status, error_text).into())
        }
    }

    pub async fn register_resource(&self, resource_type: &str, capacity: &str) -> Result<Resource, Box<dyn Error>> {
        debug!("Registering resource: type={}, capacity={}", resource_type, capacity);
        let response = self.client
            .post(&format!("{}/api/v1/resources", self.base_url))
            .json(&serde_json::json!({
                "resource_type": resource_type,
                "capacity": capacity
            }))
            .send()
            .await?;
            
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let resource = response_json["resource"].clone();
            let resource: Resource = serde_json::from_value(resource)?;
            trace!("Registered resource: {:?}", resource);
            Ok(resource)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to register resource: {} - {}", status, error_text).into())
        }
    }

    pub async fn list_resources(&self) -> Result<Vec<Resource>, Box<dyn Error>> {
        debug!("Listing resources");
        let response = self.client
            .get(&format!("{}/api/v1/resources", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let resources: ResourcesResponse = response.json().await?;
            trace!("Found {} resources", resources.resources.len());
            Ok(resources.resources)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to list resources: {} - {}", status, error_text).into())
        }
    }

    pub async fn create_proposal(&self, title: &str, description: &str) -> Result<Proposal, Box<dyn Error>> {
        debug!("Creating proposal: title={}", title);
        let response = self.client
            .post(&format!("{}/api/v1/governance/proposals", self.base_url))
            .json(&serde_json::json!({
                "title": title,
                "description": description
            }))
            .send()
            .await?;
            
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let proposal = response_json["proposal"].clone();
            let proposal: Proposal = serde_json::from_value(proposal)?;
            trace!("Created proposal: {:?}", proposal);
            Ok(proposal)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to create proposal: {} - {}", status, error_text).into())
        }
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, vote: &str) -> Result<(), Box<dyn Error>> {
        debug!("Voting on proposal: id={}, vote={}", proposal_id, vote);
        
        // Validate vote value
        let vote_value = vote.to_lowercase();
        if vote_value != "yes" && vote_value != "no" {
            return Err("Vote must be 'yes' or 'no'".into());
        }
        
        let response = self.client
            .post(&format!("{}/api/v1/governance/proposals/{}/vote", self.base_url, proposal_id))
            .json(&serde_json::json!({ "vote": vote_value }))
            .send()
            .await?;
            
        if response.status().is_success() {
            trace!("Successfully voted on proposal {}", proposal_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to vote on proposal: {} - {}", status, error_text).into())
        }
    }

    // Network-related methods
    
    pub async fn get_network_status(&self, detailed: bool) -> Result<NetworkStatus, Box<dyn Error>> {
        debug!("Getting network status (detailed={})", detailed);
        let url = if detailed {
            format!("{}/api/v1/network/status?detail=true", self.base_url)
        } else {
            format!("{}/api/v1/network/status", self.base_url)
        };
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status().is_success() {
            let status: NetworkStatus = response.json().await?;
            trace!("Network status: {:?}", status);
            Ok(status)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to get network status: {} - {}", status, error_text).into())
        }
    }
    
    pub async fn list_peers(&self) -> Result<Vec<NetworkPeer>, Box<dyn Error>> {
        debug!("Listing network peers");
        let response = self.client
            .get(&format!("{}/api/v1/network/peers", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let peers_response: NetworkPeersResponse = response.json().await?;
            trace!("Found {} peers", peers_response.peers.len());
            Ok(peers_response.peers)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to list peers: {} - {}", status, error_text).into())
        }
    }
    
    pub async fn connect_peer(&self, address: &str) -> Result<NetworkPeer, Box<dyn Error>> {
        debug!("Connecting to peer: {}", address);
        let response = self.client
            .post(&format!("{}/api/v1/network/peers/connect", self.base_url))
            .json(&serde_json::json!({ "address": address }))
            .send()
            .await?;
            
        if response.status().is_success() {
            let peer: NetworkPeer = response.json().await?;
            trace!("Connected to peer: {:?}", peer);
            Ok(peer)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to connect to peer: {} - {}", status, error_text).into())
        }
    }
    
    pub async fn disconnect_peer(&self, peer_id: &str) -> Result<(), Box<dyn Error>> {
        debug!("Disconnecting from peer: {}", peer_id);
        let response = self.client
            .post(&format!("{}/api/v1/network/peers/{}/disconnect", self.base_url, peer_id))
            .send()
            .await?;
            
        if response.status().is_success() {
            trace!("Disconnected from peer {}", peer_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to disconnect peer: {} - {}", status, error_text).into())
        }
    }
    
    pub async fn ping_peer(&self, peer_id: &str, count: u8) -> Result<Vec<PingResult>, Box<dyn Error>> {
        debug!("Pinging peer {} ({} times)", peer_id, count);
        let response = self.client
            .post(&format!("{}/api/v1/network/peers/{}/ping", self.base_url, peer_id))
            .json(&serde_json::json!({ "count": count }))
            .send()
            .await?;
            
        if response.status().is_success() {
            let results: Vec<PingResult> = response.json().await?;
            trace!("Ping results: {:?}", results);
            Ok(results)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to ping peer: {} - {}", status, error_text).into())
        }
    }
    
    pub async fn run_diagnostics(&self) -> Result<String, Box<dyn Error>> {
        debug!("Running network diagnostics");
        let response = self.client
            .get(&format!("{}/api/v1/network/diagnostics", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let diagnostics = response.text().await?;
            trace!("Diagnostics result length: {} bytes", diagnostics.len());
            Ok(diagnostics)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to run diagnostics: {} - {}", status, error_text).into())
        }
    }
}