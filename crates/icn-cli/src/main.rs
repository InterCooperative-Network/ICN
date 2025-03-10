use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use tokio::time::Duration;
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Parser)]
#[command(author, version, about = "ICN Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API endpoint URL
    #[arg(short, long, default_value = "http://localhost:8081")]
    endpoint: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the ICN network
    Start {
        /// Start in development mode
        #[arg(short, long)]
        dev: bool,
    },
    /// Stop the ICN network
    Stop,
    /// Status of the ICN network
    Status,
    /// Interact with federations
    Federation {
        #[command(subcommand)]
        action: FederationCommands,
    },
    /// Interact with identity management
    Identity {
        #[command(subcommand)]
        action: IdentityCommands,
    },
    /// Manage governance operations
    Governance {
        #[command(subcommand)]
        action: GovernanceCommands,
    },
    /// Resource sharing operations
    Resource {
        #[command(subcommand)]
        action: ResourceCommands,
    },
}

#[derive(Subcommand)]
enum FederationCommands {
    /// List all federations
    List,
    /// Create a new federation
    Create {
        /// Federation name
        #[arg(short, long)]
        name: String,
        /// Federation description
        #[arg(short, long)]
        description: String,
    },
    /// Join a federation
    Join {
        /// Federation ID
        #[arg(short, long)]
        id: String,
    },
    /// Leave a federation
    Leave {
        /// Federation ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum IdentityCommands {
    /// Create a new identity
    Create,
    /// Show current identity
    Show,
    /// List all identities in the network
    List,
}

#[derive(Subcommand)]
enum GovernanceCommands {
    /// List proposals
    Proposals,
    /// Create a new proposal
    CreateProposal {
        /// Proposal title
        #[arg(short, long)]
        title: String,
        /// Proposal description
        #[arg(short, long)]
        description: String,
    },
    /// Vote on a proposal
    Vote {
        /// Proposal ID
        #[arg(short, long)]
        proposal_id: String,
        /// Vote (approve/reject)
        #[arg(short, long)]
        approve: bool,
    },
}

#[derive(Subcommand)]
enum ResourceCommands {
    /// List available resources
    List,
    /// Share a resource
    Share {
        /// Resource type (compute, storage, bandwidth)
        #[arg(short, long)]
        resource_type: String,
        /// Resource amount
        #[arg(short, long)]
        amount: u64,
    },
    /// Request a resource
    Request {
        /// Resource type (compute, storage, bandwidth)
        #[arg(short, long)]
        resource_type: String,
        /// Resource amount
        #[arg(short, long)]
        amount: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    match &cli.command {
        Commands::Start { dev } => {
            println!("{}", "Starting ICN network...".green());
            let script_path = Path::new("scripts/start_icn.sh");
            
            if script_path.exists() {
                let mut cmd = Command::new("sh");
                
                if *dev {
                    cmd.arg("-c").arg("docker-compose -f docker/docker-compose.dev.yml up -d");
                    println!("{}", "Starting in development mode".blue());
                } else {
                    cmd.arg(script_path);
                }
                
                let status = cmd.status()?;
                
                if status.success() {
                    println!("{}", "ICN network started successfully!".green());
                } else {
                    println!("{}", "Failed to start ICN network".red());
                }
            } else {
                println!("{}", "Start script not found. Make sure you're in the ICN root directory.".red());
            }
        },
        Commands::Stop => {
            println!("{}", "Stopping ICN network...".yellow());
            let status = Command::new("sh")
                .arg("-c")
                .arg("docker-compose -f docker/docker-compose.yml down")
                .status()?;
                
            if status.success() {
                println!("{}", "ICN network stopped successfully!".green());
            } else {
                println!("{}", "Failed to stop ICN network".red());
            }
        },
        Commands::Status => {
            println!("{}", "Checking ICN network status...".blue());
            
            // Check if docker containers are running
            let output = Command::new("sh")
                .arg("-c")
                .arg("docker ps --format '{{.Names}}' | grep 'icn-'")
                .output()?;
                
            if output.stdout.is_empty() {
                println!("{}", "ICN network is not running".yellow());
                return Ok(());
            }
            
            println!("{}", "Active ICN containers:".green());
            println!("{}", String::from_utf8_lossy(&output.stdout));
            
            // Try to connect to backend API
            match client.get(format!("{}/api/v1/health", cli.endpoint)).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("{}", "Backend API is healthy ✓".green());
                        let status: Value = response.json().await?;
                        println!("API Status: {}", status);
                    } else {
                        println!("{}", format!("Backend API returned status: {}", response.status()).yellow());
                    }
                },
                Err(e) => {
                    println!("{}", format!("Failed to connect to backend API: {}", e).red());
                }
            }
        },
        Commands::Federation { action } => handle_federation(action, &client, &cli.endpoint).await?,
        Commands::Identity { action } => handle_identity(action, &client, &cli.endpoint).await?,
        Commands::Governance { action } => handle_governance(action, &client, &cli.endpoint).await?,
        Commands::Resource { action } => handle_resource(action, &client, &cli.endpoint).await?,
    }

    Ok(())
}

async fn handle_federation(
    action: &FederationCommands, 
    client: &Client, 
    endpoint: &str
) -> Result<(), Box<dyn Error>> {
    match action {
        FederationCommands::List => {
            println!("{}", "Listing federations...".blue());
            
            match client.get(format!("{}/api/v1/federations", endpoint)).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let federations: Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&federations)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        FederationCommands::Create { name, description } => {
            println!("{}", format!("Creating federation '{}'...", name).blue());
            
            let request_body = json!({
                "name": name,
                "description": description
            });
            
            match client.post(format!("{}/api/v1/federations", endpoint))
                .json(&request_body)
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: Value = response.json().await?;
                        println!("{}", "Federation created successfully!".green());
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        FederationCommands::Join { id } => {
            println!("{}", format!("Joining federation '{}'...", id).blue());
            
            match client.post(format!("{}/api/v1/federations/{}/join", endpoint, id))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("{}", "Joined federation successfully!".green());
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        FederationCommands::Leave { id } => {
            println!("{}", format!("Leaving federation '{}'...", id).blue());
            
            match client.post(format!("{}/api/v1/federations/{}/leave", endpoint, id))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("{}", "Left federation successfully!".green());
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
    }
    
    Ok(())
}

async fn handle_identity(
    action: &IdentityCommands, 
    client: &Client, 
    endpoint: &str
) -> Result<(), Box<dyn Error>> {
    match action {
        IdentityCommands::Create => {
            println!("{}", "Creating new identity...".blue());
            
            match client.post(format!("{}/api/v1/identity", endpoint))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let identity: Value = response.json().await?;
                        println!("{}", "Identity created successfully!".green());
                        println!("{}", serde_json::to_string_pretty(&identity)?);
                        
                        // Save identity in local config
                        let config_dir = dirs::config_dir()
                            .ok_or("Could not find config directory")?
                            .join("icn");
                        
                        fs::create_dir_all(&config_dir)?;
                        fs::write(
                            config_dir.join("identity.json"), 
                            serde_json::to_string_pretty(&identity)?
                        )?;
                        
                        println!("{}", format!("Identity saved to {}", config_dir.display()).blue());
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        IdentityCommands::Show => {
            println!("{}", "Showing current identity...".blue());
            
            let config_dir = dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("icn");
            
            let identity_file = config_dir.join("identity.json");
            
            if identity_file.exists() {
                let identity = fs::read_to_string(identity_file)?;
                let identity_json: Value = serde_json::from_str(&identity)?;
                println!("{}", serde_json::to_string_pretty(&identity_json)?);
            } else {
                println!("{}", "No local identity found. Create one with 'icn identity create'".yellow());
            }
        },
        IdentityCommands::List => {
            println!("{}", "Listing identities in the network...".blue());
            
            match client.get(format!("{}/api/v1/identity/list", endpoint))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let identities: Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&identities)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
    }
    
    Ok(())
}

async fn handle_governance(
    action: &GovernanceCommands, 
    client: &Client, 
    endpoint: &str
) -> Result<(), Box<dyn Error>> {
    match action {
        GovernanceCommands::Proposals => {
            println!("{}", "Listing proposals...".blue());
            
            match client.get(format!("{}/api/v1/governance/proposals", endpoint))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let proposals: Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&proposals)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        GovernanceCommands::CreateProposal { title, description } => {
            println!("{}", format!("Creating proposal '{}'...", title).blue());
            
            let request_body = json!({
                "title": title,
                "description": description
            });
            
            match client.post(format!("{}/api/v1/governance/proposals", endpoint))
                .json(&request_body)
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: Value = response.json().await?;
                        println!("{}", "Proposal created successfully!".green());
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        GovernanceCommands::Vote { proposal_id, approve } => {
            println!("{}", format!("Voting on proposal '{}'...", proposal_id).blue());
            
            let request_body = json!({
                "approve": approve
            });
            
            match client.post(format!("{}/api/v1/governance/proposals/{}/vote", endpoint, proposal_id))
                .json(&request_body)
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("{}", "Vote cast successfully!".green());
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
    }
    
    Ok(())
}

async fn handle_resource(
    action: &ResourceCommands, 
    client: &Client, 
    endpoint: &str
) -> Result<(), Box<dyn Error>> {
    match action {
        ResourceCommands::List => {
            println!("{}", "Listing resources...".blue());
            
            match client.get(format!("{}/api/v1/resources", endpoint))
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let resources: Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&resources)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        ResourceCommands::Share { resource_type, amount } => {
            println!("{}", format!("Sharing {} resources of type {}...", amount, resource_type).blue());
            
            let request_body = json!({
                "resource_type": resource_type,
                "amount": amount
            });
            
            match client.post(format!("{}/api/v1/resources/share", endpoint))
                .json(&request_body)
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: Value = response.json().await?;
                        println!("{}", "Resource shared successfully!".green());
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
        ResourceCommands::Request { resource_type, amount } => {
            println!("{}", format!("Requesting {} resources of type {}...", amount, resource_type).blue());
            
            let request_body = json!({
                "resource_type": resource_type,
                "amount": amount
            });
            
            match client.post(format!("{}/api/v1/resources/request", endpoint))
                .json(&request_body)
                .send()
                .await 
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: Value = response.json().await?;
                        println!("{}", "Resource request successful!".green());
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{}", format!("API returned error: {}", response.status()).red());
                        let error_text = response.text().await?;
                        println!("{}", error_text);
                    }
                },
                Err(e) => {
                    println!("{}", format!("Request failed: {}", e).red());
                }
            }
        },
    }
    
    Ok(())
}