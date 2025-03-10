mod client;

use clap::{Command, Arg};
use std::error::Error;
use client::IcnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Command::new("icn-cli")
        .version("0.1.0")
        .author("ICN Team")
        .about("Command Line Interface for the Inter-Cooperative Network")
        .arg(
            Arg::new("api-url")
                .long("api-url")
                .help("ICN API URL")
                .default_value("http://localhost:8082")
        )
        .subcommand(
            Command::new("health")
                .about("Check the health of the ICN API")
        )
        .subcommand(
            Command::new("identity")
                .about("Identity management commands")
                .subcommand(
                    Command::new("create")
                        .about("Create a new DID identity")
                )
                .subcommand(
                    Command::new("list")
                        .about("List existing DIDs")
                )
        )
        .subcommand(
            Command::new("cooperative")
                .about("Cooperative management commands")
                .subcommand(
                    Command::new("join")
                        .about("Join a cooperative")
                        .arg(
                            Arg::new("coop-id")
                                .help("ID of the cooperative to join")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List available cooperatives")
                )
        )
        .subcommand(
            Command::new("resource")
                .about("Resource management commands")
                .subcommand(
                    Command::new("register")
                        .about("Register a new resource")
                        .arg(
                            Arg::new("resource-type")
                                .help("Type of resource (compute, storage, etc.)")
                                .required(true)
                        )
                        .arg(
                            Arg::new("capacity")
                                .help("Capacity of the resource")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List available resources")
                )
        )
        .subcommand(
            Command::new("governance")
                .about("Governance commands")
                .subcommand(
                    Command::new("propose")
                        .about("Create a new governance proposal")
                        .arg(
                            Arg::new("title")
                                .help("Title of the proposal")
                                .required(true)
                        )
                        .arg(
                            Arg::new("description")
                                .help("Description of the proposal")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("vote")
                        .about("Vote on a proposal")
                        .arg(
                            Arg::new("proposal-id")
                                .help("ID of the proposal to vote on")
                                .required(true)
                        )
                        .arg(
                            Arg::new("vote")
                                .help("Your vote (yes/no)")
                                .required(true)
                        )
                )
        );

    let matches = app.get_matches();
    let api_url = matches.get_one::<String>("api-url").unwrap().to_string();
    let client = IcnClient::new(api_url);

    if let Some(_) = matches.subcommand_matches("health") {
        match client.check_health().await {
            Ok(health) => {
                println!("✅ ICN API is healthy");
                println!("Status: {}", health.status);
                println!("Version: {}", health.version);
                println!("Uptime: {} seconds", health.uptime);
            },
            Err(e) => {
                println!("❌ ICN API health check failed: {}", e);
                return Err(e);
            }
        }
    } else if let Some(identity_matches) = matches.subcommand_matches("identity") {
        if let Some(_) = identity_matches.subcommand_matches("create") {
            match client.create_identity().await {
                Ok(identity) => {
                    println!("✅ Created new identity");
                    println!("DID: {}", identity.did);
                    println!("Public Key: {}", identity.public_key);
                },
                Err(e) => {
                    println!("❌ Failed to create identity: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(_) = identity_matches.subcommand_matches("list") {
            match client.list_identities().await {
                Ok(identities) => {
                    println!("✅ Found {} identities", identities.len());
                    for identity in identities {
                        println!("DID: {}", identity.did);
                        println!("Public Key: {}", identity.public_key);
                        println!("---");
                    }
                },
                Err(e) => {
                    println!("❌ Failed to list identities: {}", e);
                    return Err(e);
                }
            }
        }
    } else if let Some(cooperative_matches) = matches.subcommand_matches("cooperative") {
        if let Some(join_matches) = cooperative_matches.subcommand_matches("join") {
            let coop_id = join_matches.get_one::<String>("coop-id").unwrap();
            match client.join_cooperative(coop_id).await {
                Ok(_) => {
                    println!("✅ Successfully joined cooperative {}", coop_id);
                },
                Err(e) => {
                    println!("❌ Failed to join cooperative: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(_) = cooperative_matches.subcommand_matches("list") {
            match client.list_cooperatives().await {
                Ok(cooperatives) => {
                    println!("✅ Found {} cooperatives", cooperatives.len());
                    for coop in cooperatives {
                        println!("ID: {}", coop.id);
                        println!("Name: {}", coop.name);
                        println!("Members: {}", coop.member_count);
                        println!("---");
                    }
                },
                Err(e) => {
                    println!("❌ Failed to list cooperatives: {}", e);
                    return Err(e);
                }
            }
        }
    } else if let Some(resource_matches) = matches.subcommand_matches("resource") {
        if let Some(register_matches) = resource_matches.subcommand_matches("register") {
            let resource_type = register_matches.get_one::<String>("resource-type").unwrap();
            let capacity = register_matches.get_one::<String>("capacity").unwrap();
            match client.register_resource(resource_type, capacity).await {
                Ok(resource) => {
                    println!("✅ Successfully registered resource");
                    println!("ID: {}", resource.id);
                    println!("Type: {}", resource.resource_type);
                    println!("Capacity: {}", resource.capacity);
                },
                Err(e) => {
                    println!("❌ Failed to register resource: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(_) = resource_matches.subcommand_matches("list") {
            match client.list_resources().await {
                Ok(resources) => {
                    println!("✅ Found {} resources", resources.len());
                    for resource in resources {
                        println!("ID: {}", resource.id);
                        println!("Type: {}", resource.resource_type);
                        println!("Capacity: {}", resource.capacity);
                        println!("Owner: {}", resource.owner);
                        println!("---");
                    }
                },
                Err(e) => {
                    println!("❌ Failed to list resources: {}", e);
                    return Err(e);
                }
            }
        }
    } else if let Some(governance_matches) = matches.subcommand_matches("governance") {
        if let Some(propose_matches) = governance_matches.subcommand_matches("propose") {
            let title = propose_matches.get_one::<String>("title").unwrap();
            let description = propose_matches.get_one::<String>("description").unwrap();
            match client.create_proposal(title, description).await {
                Ok(proposal) => {
                    println!("✅ Successfully created proposal");
                    println!("ID: {}", proposal.id);
                    println!("Title: {}", proposal.title);
                    println!("Status: {}", proposal.status);
                },
                Err(e) => {
                    println!("❌ Failed to create proposal: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(vote_matches) = governance_matches.subcommand_matches("vote") {
            let proposal_id = vote_matches.get_one::<String>("proposal-id").unwrap();
            let vote = vote_matches.get_one::<String>("vote").unwrap();
            match client.vote_on_proposal(proposal_id, vote).await {
                Ok(_) => {
                    println!("✅ Successfully voted on proposal {}", proposal_id);
                },
                Err(e) => {
                    println!("❌ Failed to vote on proposal: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}