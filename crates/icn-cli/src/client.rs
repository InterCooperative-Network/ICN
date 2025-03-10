use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use log::{debug, trace, warn, error, info};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcnClientError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API error: {status} - {message}")]
    ApiError { status: String, message: String },
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Not found: {0}")]
    NotFoundError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub struct IcnClient {
    client: Client,
    base_url: String,
    timeout: Duration,
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
    #[serde(default)]
    pub bandwidth_in: Option<f32>,
    #[serde(default)]
    pub bandwidth_out: Option<f32>,
    #[serde(default)]
    pub connected_since: Option<String>,
    #[serde(default)]
    pub peer_stats: Option<PeerStatistics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerStatistics {
    pub total_peers_seen: u32,
    pub avg_connection_time: u32,
    pub max_concurrent_peers: u32,
    pub successful_connections: u32,
    pub failed_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkPeer {
    pub id: String,
    pub address: String,
    pub latency: u32,
    pub connected_since: String,
    #[serde(default = "default_connected_status")]
    pub status: String,
    #[serde(default)]
    pub last_seen: Option<String>,
    #[serde(default)]
    pub bytes_sent: Option<u64>,
    #[serde(default)]
    pub bytes_received: Option<u64>,
    #[serde(default)]
    pub protocols: Option<Vec<String>>,
    #[serde(default)]
    pub connection_attempts: Option<u32>,
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
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub sequence: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsResult {
    pub timestamp: String,
    pub status: String,
    pub report: String,
    pub issues: Option<Vec<DiagnosticsIssue>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsIssue {
    pub severity: String,
    pub message: String,
    pub context: Option<String>,
    pub recommendation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl IcnClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        debug!("Created IcnClient with base URL: {}", base_url);
        
        Self {
            client,
            base_url,
            timeout: Duration::from_secs(30),
        }
    }
    
    pub fn with_timeout(base_url: String, timeout_seconds: u64) -> Self {
        let timeout = Duration::from_secs(timeout_seconds);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        debug!("Created IcnClient with base URL: {} and timeout: {}s", base_url, timeout_seconds);
        
        Self {
            client,
            base_url,
            timeout,
        }
    }
    
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self, 
        response: reqwest::Response
    ) -> Result<T, IcnClientError> {
        let status = response.status();
        let url = response.url().to_string();
        
        if status.is_success() {
            match response.json::<T>().await {
                Ok(data) => {
                    trace!("Successfully parsed response from {}", url);
                    Ok(data)
                },
                Err(e) => {
                    error!("Failed to parse response from {}: {}", url, e);
                    Err(IcnClientError::InvalidResponse(format!("Failed to parse response: {}", e)))
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Try to parse as ErrorResponse
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                error!("API error from {}: {} - {}", url, status, error_response.message);
                return Err(IcnClientError::ApiError { 
                    status: error_response.status, 
                    message: error_response.message 
                });
            }
            
            // Handle specific status codes
            let error = match status {
                StatusCode::NOT_FOUND => {
                    IcnClientError::NotFoundError(format!("Resource not found: {}", url))
                },
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    IcnClientError::AuthorizationError(format!("Authorization failed for: {}", url))
                },
                StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => {
                    IcnClientError::TimeoutError(format!("Request timed out: {}", url))
                },
                _ => {
                    IcnClientError::ApiError { 
                        status: status.to_string(), 
                        message: format!("API error: {}", error_text) 
                    }
                }
            };
            
            error!("Request to {} failed: {}", url, error);
            Err(error)
        }
    }
    
    pub async fn check_health(&self) -> Result<HealthStatus, IcnClientError> {
        debug!("Checking health at {}/health", self.base_url);
        
        let response = match self.client.get(&format!("{}/health", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Health check request failed: {}", e);
                if e.is_timeout() {
                    return Err(IcnClientError::TimeoutError("Health check timed out".to_string()));
                } else if e.is_connect() {
                    return Err(IcnClientError::ConnectionError("Failed to connect to API".to_string()));
                } else {
                    return Err(IcnClientError::HttpError(e));
                }
            }
        };
        
        self.handle_response::<HealthStatus>(response).await
    }
    
    pub async fn create_identity(&self) -> Result<Identity, IcnClientError> {
        debug!("Creating new identity at {}/identity", self.base_url);
        
        let response = match self.client.post(&format!("{}/identity", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Create identity request failed: {}", e);
                return Err(IcnClientError::from(e));
            }
        };
        
        self.handle_response::<Identity>(response).await
    }
    
    pub async fn list_identities(&self) -> Result<Vec<Identity>, IcnClientError> {
        debug!("Listing identities from {}/identity", self.base_url);
        
        let response = match self.client.get(&format!("{}/identity", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        let response_data = self.handle_response::<IdentitiesResponse>(response).await?;
        Ok(response_data.identities)
    }
    
    pub async fn join_cooperative(&self, coop_id: &str) -> Result<(), IcnClientError> {
        debug!("Joining cooperative {} at {}/cooperative/join", coop_id, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/cooperative/join", self.base_url))
            .json(&serde_json::json!({ "coop_id": coop_id }))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().to_string();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(IcnClientError::ApiError { 
                status, 
                message: error_text 
            })
        }
    }
    
    pub async fn list_cooperatives(&self) -> Result<Vec<Cooperative>, IcnClientError> {
        debug!("Listing cooperatives from {}/cooperative", self.base_url);
        
        let response = match self.client.get(&format!("{}/cooperative", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        let response_data = self.handle_response::<CooperativesResponse>(response).await?;
        Ok(response_data.cooperatives)
    }
    
    pub async fn register_resource(&self, resource_type: &str, capacity: &str) -> Result<Resource, IcnClientError> {
        debug!("Registering resource of type {} with capacity {} at {}/resource", 
               resource_type, capacity, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/resource", self.base_url))
            .json(&serde_json::json!({
                "resource_type": resource_type,
                "capacity": capacity
            }))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        self.handle_response::<Resource>(response).await
    }
    
    pub async fn list_resources(&self) -> Result<Vec<Resource>, IcnClientError> {
        debug!("Listing resources from {}/resource", self.base_url);
        
        let response = match self.client.get(&format!("{}/resource", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        let response_data = self.handle_response::<ResourcesResponse>(response).await?;
        Ok(response_data.resources)
    }
    
    pub async fn create_proposal(&self, title: &str, description: &str) -> Result<Proposal, IcnClientError> {
        debug!("Creating proposal '{}' at {}/governance/proposal", title, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/governance/proposal", self.base_url))
            .json(&serde_json::json!({
                "title": title,
                "description": description
            }))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        self.handle_response::<Proposal>(response).await
    }
    
    pub async fn vote_on_proposal(&self, proposal_id: &str, vote: &str) -> Result<(), IcnClientError> {
        let vote_value = match vote.to_lowercase().as_str() {
            "yes" | "y" | "true" => true,
            "no" | "n" | "false" => false,
            _ => {
                return Err(IcnClientError::InvalidResponse(
                    format!("Invalid vote value: {}. Must be 'yes' or 'no'.", vote)
                ));
            }
        };
        
        debug!("Voting {} on proposal {} at {}/governance/proposal/{}/vote", 
               vote, proposal_id, self.base_url, proposal_id);
        
        let response = match self.client
            .post(&format!("{}/governance/proposal/{}/vote", self.base_url, proposal_id))
            .json(&serde_json::json!({ "vote": vote_value }))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().to_string();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(IcnClientError::ApiError { 
                status, 
                message: error_text 
            })
        }
    }
    
    pub async fn get_network_status(&self, detailed: bool) -> Result<NetworkStatus, IcnClientError> {
        let url = if detailed {
            format!("{}/network/status?detailed=true", self.base_url)
        } else {
            format!("{}/network/status", self.base_url)
        };
        
        debug!("Getting network status from {}", url);
        
        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        self.handle_response::<NetworkStatus>(response).await
    }
    
    pub async fn list_peers(&self) -> Result<Vec<NetworkPeer>, IcnClientError> {
        debug!("Listing peers from {}/network/peers", self.base_url);
        
        let response = match self.client.get(&format!("{}/network/peers", self.base_url)).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        let response_data = self.handle_response::<NetworkPeersResponse>(response).await?;
        Ok(response_data.peers)
    }
    
    pub async fn list_peers_with_filter(&self, filter: &str) -> Result<Vec<NetworkPeer>, IcnClientError> {
        let url = format!("{}/network/peers?filter={}", self.base_url, filter);
        debug!("Listing peers with filter {} from {}", filter, url);
        
        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        let response_data = self.handle_response::<NetworkPeersResponse>(response).await?;
        
        // Additional client-side filtering if the API doesn't support it
        let peers = match filter {
            "all" => response_data.peers,
            "connected" => response_data.peers.into_iter()
                .filter(|p| p.status == "connected")
                .collect(),
            "disconnected" => response_data.peers.into_iter()
                .filter(|p| p.status == "disconnected")
                .collect(),
            _ => response_data.peers,
        };
        
        Ok(peers)
    }
    
    pub async fn connect_peer(&self, address: &str) -> Result<NetworkPeer, IcnClientError> {
        debug!("Connecting to peer at address {} via {}/network/connect", 
               address, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/network/connect", self.base_url))
            .json(&serde_json::json!({ "address": address }))
            .timeout(self.timeout)
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    error!("Connection to peer timed out after {}s", self.timeout.as_secs());
                    return Err(IcnClientError::TimeoutError(
                        format!("Connection to peer timed out after {}s", self.timeout.as_secs())
                    ));
                } else {
                    return Err(IcnClientError::from(e));
                }
            }
        };
        
        self.handle_response::<NetworkPeer>(response).await
    }
    
    pub async fn disconnect_peer(&self, peer_id: &str) -> Result<(), IcnClientError> {
        debug!("Disconnecting from peer {} via {}/network/disconnect", 
               peer_id, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/network/disconnect", self.base_url))
            .json(&serde_json::json!({ "peer_id": peer_id }))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => return Err(IcnClientError::from(e)),
        };
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().to_string();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(IcnClientError::ApiError { 
                status, 
                message: error_text 
            })
        }
    }
    
    pub async fn ping_peer(&self, peer_id: &str, count: u8) -> Result<Vec<PingResult>, IcnClientError> {
        debug!("Pinging peer {} ({} times) via {}/network/ping", 
               peer_id, count, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/network/ping", self.base_url))
            .json(&serde_json::json!({
                "peer_id": peer_id,
                "count": count
            }))
            .timeout(Duration::from_secs(count as u64 * 2)) // Allow enough time for the pings
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    error!("Ping request timed out after {}s", count as u64 * 2);
                    return Err(IcnClientError::TimeoutError(
                        format!("Ping request timed out after {}s", count as u64 * 2)
                    ));
                } else {
                    return Err(IcnClientError::from(e));
                }
            }
        };
        
        self.handle_response::<Vec<PingResult>>(response).await
    }
    
    pub async fn ping_peer_with_interval(
        &self, 
        peer_id: &str, 
        count: u8, 
        interval_ms: u64
    ) -> Result<Vec<PingResult>, IcnClientError> {
        debug!("Pinging peer {} ({} times with {}ms interval) via {}/network/ping", 
               peer_id, count, interval_ms, self.base_url);
        
        let response = match self.client
            .post(&format!("{}/network/ping", self.base_url))
            .json(&serde_json::json!({
                "peer_id": peer_id,
                "count": count,
                "interval_ms": interval_ms
            }))
            // Allow enough time for all pings plus a buffer
            .timeout(Duration::from_millis(count as u64 * interval_ms + 5000))
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    error!("Ping request timed out");
                    return Err(IcnClientError::TimeoutError("Ping request timed out".to_string()));
                } else {
                    return Err(IcnClientError::from(e));
                }
            }
        };
        
        self.handle_response::<Vec<PingResult>>(response).await
    }
    
    pub async fn run_diagnostics(&self) -> Result<String, IcnClientError> {
        debug!("Running network diagnostics via {}/network/diagnostics", self.base_url);
        
        let response = match self.client
            .get(&format!("{}/network/diagnostics", self.base_url))
            .timeout(Duration::from_secs(60)) // Diagnostics can take time
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    error!("Diagnostics request timed out after 60s");
                    return Err(IcnClientError::TimeoutError(
                        "Diagnostics request timed out after 60s".to_string()
                    ));
                } else {
                    return Err(IcnClientError::from(e));
                }
            }
        };
        
        if response.status().is_success() {
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(IcnClientError::from(e)),
            }
        } else {
            let status = response.status().to_string();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(IcnClientError::ApiError { 
                status, 
                message: error_text 
            })
        }
    }
    
    pub async fn run_diagnostics_with_options(
        &self, 
        comprehensive: bool
    ) -> Result<String, IcnClientError> {
        let url = format!(
            "{}/network/diagnostics?comprehensive={}", 
            self.base_url, 
            comprehensive
        );
        
        debug!("Running network diagnostics (comprehensive: {}) via {}", comprehensive, url);
        
        let timeout = if comprehensive {
            Duration::from_secs(180) // Comprehensive diagnostics need more time
        } else {
            Duration::from_secs(60)
        };
        
        let response = match self.client
            .get(&url)
            .timeout(timeout)
            .send()
            .await 
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    error!("Diagnostics request timed out after {}s", timeout.as_secs());
                    return Err(IcnClientError::TimeoutError(
                        format!("Diagnostics request timed out after {}s", timeout.as_secs())
                    ));
                } else {
                    return Err(IcnClientError::from(e));
                }
            }
        };
        
        // Try to parse as DiagnosticsResult for structured output
        if response.status().is_success() {
            let text = response.text().await?;
            
            // If it's JSON, try to extract the report field
            if let Ok(diagnostics) = serde_json::from_str::<DiagnosticsResult>(&text) {
                let mut output = format!(
                    "Diagnostic Report ({})\nStatus: {}\nTimestamp: {}\n\n",
                    if comprehensive { "Comprehensive" } else { "Basic" },
                    diagnostics.status,
                    diagnostics.timestamp
                );
                
                output.push_str(&diagnostics.report);
                
                if let Some(issues) = diagnostics.issues {
                    if !issues.is_empty() {
                        output.push_str("\n\nDetected Issues:\n");
                        for issue in issues {
                            output.push_str(&format!("- [{}] {}\n", issue.severity, issue.message));
                            if let Some(context) = issue.context {
                                output.push_str(&format!("  Context: {}\n", context));
                            }
                            if let Some(recommendation) = issue.recommendation {
                                output.push_str(&format!("  Recommendation: {}\n", recommendation));
                            }
                        }
                    }
                }
                
                Ok(output)
            } else {
                // If not a valid JSON, return the raw text
                Ok(text)
            }
        } else {
            let status = response.status().to_string();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(IcnClientError::ApiError { 
                status, 
                message: error_text 
            })
        }
    }
}