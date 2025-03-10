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

impl IcnClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn check_health(&self) -> Result<HealthStatus, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;
            
        if response.status().is_success() {
            let health = response.json().await?;
            Ok(health)
        } else {
            Err(format!("API health check failed with status: {}", response.status()).into())
        }
    }

    pub async fn create_identity(&self) -> Result<Identity, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/identity/create", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn list_identities(&self) -> Result<Vec<Identity>, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/identity/list", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn join_cooperative(&self, coop_id: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .post(&format!("{}/api/v1/cooperative/join/{}", self.base_url, coop_id))
            .send()
            .await?;
        Ok(())
    }

    pub async fn list_cooperatives(&self) -> Result<Vec<Cooperative>, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/cooperative/list", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn register_resource(&self, resource_type: &str, capacity: &str) -> Result<Resource, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/resource/register", self.base_url))
            .json(&serde_json::json!({
                "type": resource_type,
                "capacity": capacity
            }))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn list_resources(&self) -> Result<Vec<Resource>, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/api/v1/resource/list", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn create_proposal(&self, title: &str, description: &str) -> Result<Proposal, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/governance/propose", self.base_url))
            .json(&serde_json::json!({
                "title": title,
                "description": description
            }))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn vote_proposal(&self, proposal_id: &str, vote: bool) -> Result<(), Box<dyn Error>> {
        self.client
            .post(&format!("{}/api/v1/governance/vote/{}", self.base_url, proposal_id))
            .json(&serde_json::json!({
                "vote": vote
            }))
            .send()
            .await?;
        Ok(())
    }
}