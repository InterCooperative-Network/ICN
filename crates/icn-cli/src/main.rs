mod client;

use clap::{App, Arg, SubCommand};
use std::error::Error;
use client::IcnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new("icn-cli")
        .version("0.1.0")
        .author("ICN Team")
        .about("Command Line Interface for the Inter-Cooperative Network")
        .arg(
            Arg::with_name("api-url")
                .long("api-url")
                .help("ICN API URL")
                .default_value("http://localhost:8081")
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("health")
                .about("Check the health of the ICN API")
        )
        .subcommand(
            SubCommand::with_name("identity")
                .about("Identity management commands")
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new DID identity")
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List existing DIDs")
                )
        )
        .subcommand(
            SubCommand::with_name("cooperative")
                .about("Cooperative management commands")
                .subcommand(
                    SubCommand::with_name("join")
                        .about("Join a cooperative")
                        .arg(
                            Arg::with_name("coop-id")
                                .help("ID of the cooperative to join")
                                .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List available cooperatives")
                )
        )
        .subcommand(
            SubCommand::with_name("resource")
                .about("Resource management commands")
                .subcommand(
                    SubCommand::with_name("register")
                        .about("Register a new resource")
                        .arg(
                            Arg::with_name("type")
                                .help("Type of resource (compute/storage/network)")
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("capacity")
                                .help("Resource capacity")
                                .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List available resources")
                )
        )
        .subcommand(
            SubCommand::with_name("governance")
                .about("Governance commands")
                .subcommand(
                    SubCommand::with_name("propose")
                        .about("Create a new proposal")
                        .arg(
                            Arg::with_name("title")
                                .help("Proposal title")
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("description")
                                .help("Proposal description")
                                .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("vote")
                        .about("Vote on a proposal")
                        .arg(
                            Arg::with_name("proposal-id")
                                .help("ID of the proposal")
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("vote")
                                .help("Vote (yes/no)")
                                .required(true)
                        )
                )
        );

    let matches = app.get_matches();
    let api_url = matches.value_of("api-url").unwrap().to_string();
    let client = IcnClient::new(api_url);

    match matches.subcommand() {
        ("health", Some(_)) => {
            match client.check_health().await {
                Ok(health) => {
                    println!("✅ ICN API is healthy");
                    println!("Status: {}", health.status);
                    println!("Version: {}", health.version);
                    println!("Uptime: {} seconds", health.uptime);
                }
                Err(e) => {
                    eprintln!("❌ ICN API health check failed: {}", e);
                    return Err(e);
                }
            }
        }
        ("identity", Some(identity_matches)) => {
            match identity_matches.subcommand() {
                ("create", Some(_)) => {
                    match client.create_identity().await {
                        Ok(identity) => println!("Created new DID: {}", identity.did),
                        Err(e) => eprintln!("Error creating identity: {}", e),
                    }
                }
                ("list", Some(_)) => {
                    match client.list_identities().await {
                        Ok(identities) => {
                            println!("Existing DIDs:");
                            for identity in identities {
                                println!("- {} (public key: {})", identity.did, identity.public_key);
                            }
                        }
                        Err(e) => eprintln!("Error listing identities: {}", e),
                    }
                }
                _ => unreachable!(),
            }
        }
        ("cooperative", Some(coop_matches)) => {
            match coop_matches.subcommand() {
                ("join", Some(join_matches)) => {
                    let coop_id = join_matches.value_of("coop-id").unwrap();
                    match client.join_cooperative(coop_id).await {
                        Ok(_) => println!("Successfully joined cooperative {}", coop_id),
                        Err(e) => eprintln!("Error joining cooperative: {}", e),
                    }
                }
                ("list", Some(_)) => {
                    match client.list_cooperatives().await {
                        Ok(cooperatives) => {
                            println!("Available cooperatives:");
                            for coop in cooperatives {
                                println!("- {} ({} members)", coop.name, coop.member_count);
                            }
                        }
                        Err(e) => eprintln!("Error listing cooperatives: {}", e),
                    }
                }
                _ => unreachable!(),
            }
        }
        ("resource", Some(resource_matches)) => {
            match resource_matches.subcommand() {
                ("register", Some(register_matches)) => {
                    let resource_type = register_matches.value_of("type").unwrap();
                    let capacity = register_matches.value_of("capacity").unwrap();
                    match client.register_resource(resource_type, capacity).await {
                        Ok(resource) => println!("Registered resource: {} (ID: {})", resource.resource_type, resource.id),
                        Err(e) => eprintln!("Error registering resource: {}", e),
                    }
                }
                ("list", Some(_)) => {
                    match client.list_resources().await {
                        Ok(resources) => {
                            println!("Available resources:");
                            for resource in resources {
                                println!("- {} ({} capacity, owned by {})", 
                                    resource.resource_type, 
                                    resource.capacity,
                                    resource.owner);
                            }
                        }
                        Err(e) => eprintln!("Error listing resources: {}", e),
                    }
                }
                _ => unreachable!(),
            }
        }
        ("governance", Some(governance_matches)) => {
            match governance_matches.subcommand() {
                ("propose", Some(propose_matches)) => {
                    let title = propose_matches.value_of("title").unwrap();
                    let description = propose_matches.value_of("description").unwrap();
                    match client.create_proposal(title, description).await {
                        Ok(proposal) => println!("Created proposal: {} (ID: {})", proposal.title, proposal.id),
                        Err(e) => eprintln!("Error creating proposal: {}", e),
                    }
                }
                ("vote", Some(vote_matches)) => {
                    let proposal_id = vote_matches.value_of("proposal-id").unwrap();
                    let vote = vote_matches.value_of("vote").unwrap().to_lowercase() == "yes";
                    match client.vote_proposal(proposal_id, vote).await {
                        Ok(_) => println!("Successfully voted on proposal {}", proposal_id),
                        Err(e) => eprintln!("Error voting on proposal: {}", e),
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => {
            println!("No command specified. Use --help to see available commands.");
        }
    }

    Ok(())
}