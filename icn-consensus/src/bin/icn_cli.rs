use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use clap::{Parser, Subcommand};
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use reqwest::Client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional config file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check status of a node
    Status {
        /// Node API URL
        #[arg(short, long, default_value = "http://localhost:8082")]
        url: String,
    },
    
    /// Get information about the consensus state
    Consensus {
        /// Node API URL
        #[arg(short, long, default_value = "http://localhost:8082")]
        url: String,
    },
    
    /// Work with validators
    Validator {
        /// Node API URL
        #[arg(short, long, default_value = "http://localhost:8082")]
        url: String,
        
        /// Validator operation
        #[command(subcommand)]
        op: ValidatorOp,
    },
    
    /// Work with federation management
    Federation {
        /// Node API URL
        #[arg(short, long, default_value = "http://localhost:8082")]
        url: String,
        
        /// Federation operation
        #[command(subcommand)]
        op: FederationOp,
    },
}

#[derive(Subcommand, Debug)]
enum ValidatorOp {
    /// List current validators
    List,
    /// Get information about a specific validator
    Info { 
        /// Validator ID
        id: String 
    },
    /// Register as a new validator
    Register {
        /// Path to validator key file
        #[arg(short, long)]
        key_file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum FederationOp {
    /// List federations
    List,
    /// Get information about a specific federation
    Info { 
        /// Federation ID
        id: String 
    },
    /// Create a new federation
    Create {
        /// Path to federation config file
        #[arg(short, long)]
        config_file: PathBuf,
    },
    /// Join a federation
    Join {
        /// Federation ID to join
        id: String,
        /// Path to join agreement file
        #[arg(short, long)]
        agreement_file: PathBuf,
    },
}

// API response types
#[derive(Deserialize, Debug)]
struct StatusResponse {
    status: String,
    node_id: String,
    node_type: String,
    uptime_seconds: u64,
    peers_connected: usize,
    cooperative_id: String,
    version: String,
}

#[derive(Deserialize, Debug)]
struct ValidatorInfo {
    id: String,
    address: String,
    status: String,
    last_seen: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create HTTP client
    let client = Client::new();
    
    // Process commands
    match &args.command {
        Commands::Status { url } => {
            status_command(&client, url).await?;
        },
        Commands::Consensus { url } => {
            consensus_command(&client, url).await?;
        },
        Commands::Validator { url, op } => {
            match op {
                ValidatorOp::List => validator_list_command(&client, url).await?,
                ValidatorOp::Info { id } => validator_info_command(&client, url, id).await?,
                ValidatorOp::Register { key_file } => validator_register_command(&client, url, key_file).await?,
            }
        },
        Commands::Federation { url, op } => {
            match op {
                FederationOp::List => federation_list_command(&client, url).await?,
                FederationOp::Info { id } => federation_info_command(&client, url, id).await?,
                FederationOp::Create { config_file } => federation_create_command(&client, url, config_file).await?,
                FederationOp::Join { id, agreement_file } => federation_join_command(&client, url, id, agreement_file).await?,
            }
        },
    }
    
    Ok(())
}

async fn status_command(client: &Client, url: &str) -> Result<()> {
    println!("Checking status of node at {}", url);
    
    let api_url = format!("{}/api/v1/status", url.trim_end_matches('/'));
    
    match client.get(&api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let status: StatusResponse = response.json().await?;
                println!("Status: {}", status.status);
                println!("Node ID: {}", status.node_id);
                println!("Node Type: {}", status.node_type);
                println!("Uptime: {} seconds", status.uptime_seconds);
                println!("Connected Peers: {}", status.peers_connected);
                println!("Cooperative ID: {}", status.cooperative_id);
                println!("Version: {}", status.version);
            } else {
                let error_text = response.text().await?;
                println!("Error: {} - {}", response.status(), error_text);
            }
        },
        Err(e) => {
            println!("Failed to connect to node: {}", e);
            return Err(anyhow!("Connection failed: {}", e));
        }
    }
    
    Ok(())
}

async fn consensus_command(client: &Client, url: &str) -> Result<()> {
    println!("Consensus information from node at {}", url);
    
    // This is a mock implementation until the API supports it
    println!("Current round: 42");
    println!("Last finalized block: 12345");
    println!("Number of validators: 5");
    
    Ok(())
}

async fn validator_list_command(client: &Client, url: &str) -> Result<()> {
    println!("Validators from node at {}", url);
    
    let api_url = format!("{}/api/v1/validators", url.trim_end_matches('/'));
    
    match client.get(&api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let validators: Vec<ValidatorInfo> = response.json().await?;
                
                if validators.is_empty() {
                    println!("No validators found");
                } else {
                    println!("{:<36} {:<15} {:<25}", "ID", "Status", "Last Seen");
                    println!("{:-<80}", "");
                    
                    for validator in validators {
                        println!("{:<36} {:<15} {}", 
                            validator.id, 
                            validator.status,
                            validator.last_seen.to_rfc3339());
                    }
                }
            } else {
                let error_text = response.text().await?;
                println!("Error: {} - {}", response.status(), error_text);
            }
        },
        Err(e) => {
            println!("Failed to connect to node: {}", e);
            return Err(anyhow!("Connection failed: {}", e));
        }
    }
    
    Ok(())
}

async fn validator_info_command(client: &Client, url: &str, id: &str) -> Result<()> {
    println!("Validator information from node at {}", url);
    
    // This endpoint doesn't exist yet in our implementation
    // For now, return mock data
    println!("ID: {}", id);
    println!("Status: Active");
    println!("Address: ws://localhost:9001");
    println!("Last Seen: 2024-03-10T12:00:00Z");
    
    Ok(())
}

async fn validator_register_command(client: &Client, url: &str, key_file: &PathBuf) -> Result<()> {
    println!("Registering validator with node at {}", url);
    println!("Using key file: {:?}", key_file);
    
    // This is a mock implementation until the API supports it
    println!("Registration submitted (Mock Implementation)");
    
    Ok(())
}

async fn federation_list_command(client: &Client, url: &str) -> Result<()> {
    println!("Federations from node at {}", url);
    
    // This is a mock implementation until the API supports it
    println!("ID\t\tName\t\t\tMembers");
    println!("fed_1\tExample Federation 1\t3");
    println!("fed_2\tExample Federation 2\t5");
    
    Ok(())
}

async fn federation_info_command(client: &Client, url: &str, id: &str) -> Result<()> {
    println!("Federation information from node at {}", url);
    
    // This is a mock implementation until the API supports it
    println!("ID: {}", id);
    println!("Name: Example Federation 1");
    println!("Members: 3");
    println!("Resources: 10 CPUs, 500GB storage");
    println!("Created: 2024-01-01");
    
    Ok(())
}

async fn federation_create_command(client: &Client, url: &str, config_file: &PathBuf) -> Result<()> {
    println!("Creating federation with node at {}", url);
    println!("Using config file: {:?}", config_file);
    
    // This is a mock implementation until the API supports it
    println!("Federation created with ID: fed_3 (Mock Implementation)");
    
    Ok(())
}

async fn federation_join_command(client: &Client, url: &str, id: &str, agreement_file: &PathBuf) -> Result<()> {
    println!("Joining federation {} with node at {}", id, url);
    println!("Using agreement file: {:?}", agreement_file);
    
    // This is a mock implementation until the API supports it
    println!("Join request submitted (Mock Implementation)");
    
    Ok(())
} 