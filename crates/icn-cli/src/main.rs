mod client;

use clap::{Command, Arg, ArgAction, value_parser};
use std::error::Error;
use client::IcnClient;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN CLI...");

    let app = Command::new("icn-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .author("ICN Team")
        .about("Command Line Interface for the Inter-Cooperative Network")
        .arg(
            Arg::new("api-url")
                .long("api-url")
                .help("ICN API URL")
                .default_value("http://localhost:8082")
                .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue)
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
                .subcommand(
                    Command::new("show")
                        .about("Show details of a specific DID")
                        .arg(
                            Arg::new("did")
                                .help("DID to show details for")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
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
                                .value_parser(value_parser!(String))
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List available cooperatives")
                )
                .subcommand(
                    Command::new("create")
                        .about("Create a new cooperative")
                        .arg(
                            Arg::new("name")
                                .help("Name of the cooperative")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("description")
                                .help("Description of the cooperative")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
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
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("capacity")
                                .help("Capacity of the resource")
                                .required(true)
                                .value_parser(value_parser!(String))
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
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("description")
                                .help("Description of the proposal")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                )
                .subcommand(
                    Command::new("vote")
                        .about("Vote on a proposal")
                        .arg(
                            Arg::new("proposal-id")
                                .help("ID of the proposal to vote on")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("vote")
                                .help("Your vote (yes/no)")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List active proposals")
                )
        )
        .subcommand(
            Command::new("network")
                .about("Network management commands")
                .subcommand(
                    Command::new("status")
                        .about("Check network status")
                        .arg(
                            Arg::new("detail")
                                .short('d')
                                .long("detail")
                                .help("Show detailed status information")
                                .action(ArgAction::SetTrue)
                        )
                )
                .subcommand(
                    Command::new("peers")
                        .about("List connected peers")
                        .arg(
                            Arg::new("filter")
                                .short('f')
                                .long("filter")
                                .help("Filter peers by status (connected, disconnected, all)")
                                .default_value("connected")
                                .value_parser(value_parser!(String))
                        )
                )
                .subcommand(
                    Command::new("connect")
                        .about("Connect to a peer")
                        .arg(
                            Arg::new("address")
                                .help("Peer address to connect to")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("timeout")
                                .short('t')
                                .long("timeout")
                                .help("Connection timeout in seconds")
                                .default_value("30")
                                .value_parser(value_parser!(u64))
                        )
                )
                .subcommand(
                    Command::new("disconnect")
                        .about("Disconnect from a peer")
                        .arg(
                            Arg::new("peer-id")
                                .help("ID of the peer to disconnect from")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                )
                .subcommand(
                    Command::new("ping")
                        .about("Ping a peer to check connectivity")
                        .arg(
                            Arg::new("peer-id")
                                .help("ID of the peer to ping")
                                .required(true)
                                .value_parser(value_parser!(String))
                        )
                        .arg(
                            Arg::new("count")
                                .short('c')
                                .long("count")
                                .help("Number of pings to send")
                                .default_value("3")
                                .value_parser(value_parser!(u8))
                        )
                )
                .subcommand(
                    Command::new("diagnostics")
                        .about("Run network diagnostics")
                )
        );

    let matches = app.get_matches();
    let api_url = matches.get_one::<String>("api-url").unwrap().to_string();
    let verbose = matches.get_flag("verbose");
    let client = IcnClient::new(api_url);

    if verbose {
        info!("Verbose mode enabled");
    }

    if let Some(_) = matches.subcommand_matches("health") {
        match client.check_health().await {
            Ok(health) => {
                println!("✅ ICN API is healthy");
                println!("Status: {}", health.status);
                println!("Version: {}", health.version);
                println!("Uptime: {} seconds", health.uptime);
            },
            Err(e) => {
                error!("Health check failed: {}", e);
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
                    error!("Failed to create identity: {}", e);
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
                    error!("Failed to list identities: {}", e);
                    println!("❌ Failed to list identities: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(show_matches) = identity_matches.subcommand_matches("show") {
            let did = show_matches.get_one::<String>("did").unwrap();
            // Implement show identity details
            println!("Showing identity details for DID: {}", did);
            println!("Note: This functionality is not yet implemented.");
        }
    } else if let Some(cooperative_matches) = matches.subcommand_matches("cooperative") {
        if let Some(join_matches) = cooperative_matches.subcommand_matches("join") {
            let coop_id = join_matches.get_one::<String>("coop-id").unwrap();
            match client.join_cooperative(coop_id).await {
                Ok(_) => {
                    println!("✅ Successfully joined cooperative {}", coop_id);
                },
                Err(e) => {
                    error!("Failed to join cooperative: {}", e);
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
                    error!("Failed to list cooperatives: {}", e);
                    println!("❌ Failed to list cooperatives: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(create_matches) = cooperative_matches.subcommand_matches("create") {
            let name = create_matches.get_one::<String>("name").unwrap();
            let description = create_matches.get_one::<String>("description").unwrap();
            // Implement create cooperative
            println!("Creating cooperative: {} - {}", name, description);
            println!("Note: This functionality is not yet implemented.");
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
                    error!("Failed to register resource: {}", e);
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
                    error!("Failed to list resources: {}", e);
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
                    error!("Failed to create proposal: {}", e);
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
                    error!("Failed to vote on proposal: {}", e);
                    println!("❌ Failed to vote on proposal: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(_) = governance_matches.subcommand_matches("list") {
            // Implement list proposals
            println!("Listing active proposals...");
            println!("Note: This functionality is not yet implemented.");
        }
    } else if let Some(network_matches) = matches.subcommand_matches("network") {
        if let Some(status_matches) = network_matches.subcommand_matches("status") {
            let detailed = status_matches.get_flag("detail");
            match client.get_network_status(detailed).await {
                Ok(status) => {
                    println!("📊 Network Status");
                    println!("Status: {}", status.status);
                    println!("Connected peers: {}", status.peer_count);
                    if detailed {
                        println!("Average latency: {}ms", status.avg_latency);
                        println!("Bandwidth usage: {}%", status.bandwidth_usage);
                    }
                },
                Err(e) => {
                    error!("Failed to get network status: {}", e);
                    println!("❌ Failed to get network status: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(peers_matches) = network_matches.subcommand_matches("peers") {
            let filter = peers_matches.get_one::<String>("filter").unwrap();
            match client.list_peers().await {
                Ok(peers) => {
                    // Filter peers based on specified filter
                    let filtered_peers = match filter.as_str() {
                        "all" => peers,
                        "connected" => peers.into_iter().filter(|p| p.status == "connected").collect(),
                        "disconnected" => peers.into_iter().filter(|p| p.status == "disconnected").collect(),
                        _ => {
                            println!("❌ Invalid filter: {}", filter);
                            return Ok(());
                        }
                    };

                    if filtered_peers.is_empty() {
                        println!("No peers found matching filter: {}", filter);
                    } else {
                        println!("📊 Peers ({})", filtered_peers.len());
                        println!("{:<40} | {:<15} | {:<8} | {}", "Peer ID", "Address", "Latency", "Status");
                        println!("--------------------------------------------------------------------------------");
                        for peer in filtered_peers {
                            println!("{:<40} | {:<15} | {:<8}ms | {}", 
                                peer.id, 
                                peer.address, 
                                peer.latency,
                                peer.status
                            );
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to list peers: {}", e);
                    println!("❌ Failed to list peers: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(connect_matches) = network_matches.subcommand_matches("connect") {
            let address = connect_matches.get_one::<String>("address").unwrap();
            let timeout = connect_matches.get_one::<u64>("timeout").unwrap();
            
            println!("Connecting to {} (timeout: {}s)...", address, timeout);
            match client.connect_peer(address).await {
                Ok(peer) => {
                    println!("✅ Successfully connected to peer");
                    println!("Peer ID: {}", peer.id);
                    println!("Address: {}", peer.address);
                    println!("Latency: {}ms", peer.latency);
                    println!("Status: {}", peer.status);
                },
                Err(e) => {
                    error!("Failed to connect to peer: {}", e);
                    println!("❌ Failed to connect to peer: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(disconnect_matches) = network_matches.subcommand_matches("disconnect") {
            let peer_id = disconnect_matches.get_one::<String>("peer-id").unwrap();
            match client.disconnect_peer(peer_id).await {
                Ok(_) => {
                    println!("✅ Successfully disconnected from peer {}", peer_id);
                },
                Err(e) => {
                    error!("Failed to disconnect peer: {}", e);
                    println!("❌ Failed to disconnect peer: {}", e);
                    return Err(e);
                }
            }
        } else if let Some(ping_matches) = network_matches.subcommand_matches("ping") {
            let peer_id = ping_matches.get_one::<String>("peer-id").unwrap();
            let count = ping_matches.get_one::<u8>("count").unwrap();
            
            // Implement peer ping functionality
            println!("Pinging peer {} ({} times)...", peer_id, count);
            println!("Note: This functionality is not yet implemented.");
        } else if let Some(_) = network_matches.subcommand_matches("diagnostics") {
            // Implement network diagnostics
            println!("Running network diagnostics...");
            println!("Note: This functionality is not yet implemented.");
        }
    } else {
        println!("No command specified. Use --help to see available commands.");
    }

    Ok(())
}