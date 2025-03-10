use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct IcnClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub did: String,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cooperative {
    pub id: String,
    pub name: String,
    pub member_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub capacity: String,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub status: String,
    pub peer_count: u32,
    pub avg_latency: u32,
    pub bandwidth_usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPeer {
    pub id: String,
    pub address: String,
    pub latency: u32,
    pub connected_since: String,
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

impl IcnClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn check_health(&self) -> Result<HealthStatus, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/health", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let health: HealthStatus = response.json().await?;
            Ok(health)
        } else {
            Err(format!("API health check failed with status: {}", response.status()).into())
        }
    }

    pub async fn create_identity(&self) -> Result<Identity, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/identities", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let identity = response_json["identity"].clone();
            let identity: Identity = serde_json::from_value(identity)?;
            Ok(identity)
        } else {
            Err(format!("Failed to create identity: {}", response.status()).into())
        }
    }

    pub async fn list_identities(&self) -> Result<Vec<Identity>, Box<dyn Error>> {
        let response: IdentitiesResponse = self.client
            .get(&format!("{}/api/v1/identities", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response.identities)
    }

    pub async fn join_cooperative(&self, coop_id: &str) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/cooperatives/join", self.base_url))
            .json(&serde_json::json!({ "cooperative_id": coop_id }))
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to join cooperative: {}", response.status()).into())
        }
    }

    pub async fn list_cooperatives(&self) -> Result<Vec<Cooperative>, Box<dyn Error>> {
        let response: CooperativesResponse = self.client
            .get(&format!("{}/api/v1/cooperatives", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response.cooperatives)
    }

    pub async fn register_resource(&self, resource_type: &str, capacity: &str) -> Result<Resource, Box<dyn Error>> {
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
            Ok(resource)
        } else {
            Err(format!("Failed to register resource: {}", response.status()).into())
        }
    }

    pub async fn list_resources(&self) -> Result<Vec<Resource>, Box<dyn Error>> {
        let response: ResourcesResponse = self.client
            .get(&format!("{}/api/v1/resources", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response.resources)
    }

    pub async fn create_proposal(&self, title: &str, description: &str) -> Result<Proposal, Box<dyn Error>> {
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
            Ok(proposal)
        } else {
            Err(format!("Failed to create proposal: {}", response.status()).into())
        }
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, vote: &str) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/governance/proposals/{}/vote", self.base_url, proposal_id))
            .json(&serde_json::json!({ "vote": vote }))
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to vote on proposal: {}", response.status()).into())
        }
    }

    // Network-related methods
    
    pub async fn get_network_status(&self, detailed: bool) -> Result<NetworkStatus, Box<dyn Error>> {
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
            Ok(status)
        } else {
            Err(format!("Failed to get network status: {}", response.status()).into())
        }
    }
    
    pub async fn list_peers(&self) -> Result<Vec<NetworkPeer>, Box<dyn Error>> {
        let response: NetworkPeersResponse = self.client
            .get(&format!("{}/api/v1/network/peers", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.peers)
    }
    
    pub async fn connect_peer(&self, address: &str) -> Result<NetworkPeer, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/network/peers/connect", self.base_url))
            .json(&serde_json::json!({ "address": address }))
            .send()
            .await?;
            
        if response.status().is_success() {
            let peer: NetworkPeer = response.json().await?;
            Ok(peer)
        } else {
            Err(format!("Failed to connect to peer: {}", response.status()).into())
        }
    }
    
    pub async fn disconnect_peer(&self, peer_id: &str) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/network/peers/{}/disconnect", self.base_url, peer_id))
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to disconnect peer: {}", response.status()).into())
        }
    }
}