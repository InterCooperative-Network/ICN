mod client;

use clap::{Command, Arg, ArgAction, value_parser};
use std::error::Error;
use client::IcnClient;
use log::{info, error, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    let mut builder = env_logger::Builder::from_default_env();
    
    let matches = Command::new("icn-cli")
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
                .help("Enable verbose output (can be used multiple times for increased verbosity)")
                .action(ArgAction::Count)
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .help("Global timeout for requests in seconds")
                .default_value("30")
                .value_parser(value_parser!(u64))
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to config file")
                .value_parser(value_parser!(String))
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
                                .value_parser(["connected", "disconnected", "all"])
                        )
                        .arg(
                            Arg::new("sort")
                                .short('s')
                                .long("sort")
                                .help("Sort peers by (latency, id, address)")
                                .default_value("id")
                                .value_parser(["latency", "id", "address"])
                        )
                        .arg(
                            Arg::new("output")
                                .short('o')
                                .long("output")
                                .help("Output format (table, json, csv)")
                                .default_value("table")
                                .value_parser(["table", "json", "csv"])
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
                        .about("Ping a peer")
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
                                .help("Number of ping packets to send")
                                .default_value("4")
                                .value_parser(value_parser!(u8))
                        )
                        .arg(
                            Arg::new("interval")
                                .short('i')
                                .long("interval")
                                .help("Interval between pings in milliseconds")
                                .default_value("1000")
                                .value_parser(value_parser!(u64))
                        )
                )
                .subcommand(
                    Command::new("diagnostics")
                        .about("Run network diagnostics")
                        .arg(
                            Arg::new("comprehensive")
                                .short('c')
                                .long("comprehensive")
                                .help("Run comprehensive diagnostics (may take longer)")
                                .action(ArgAction::SetTrue)
                        )
                        .arg(
                            Arg::new("output-file")
                                .short('o')
                                .long("output-file")
                                .help("Save diagnostics output to a file")
                                .value_parser(value_parser!(String))
                        )
                )
        )
        .get_matches();

    // Configure logging based on verbosity
    let verbosity = matches.get_count("verbose");
    match verbosity {
        0 => builder.filter_level(log::LevelFilter::Info),
        1 => builder.filter_level(log::LevelFilter::Debug),
        _ => builder.filter_level(log::LevelFilter::Trace),
    };
    builder.init();

    // Create client with global timeout if specified
    let api_url = matches.get_one::<String>("api-url").unwrap().clone();
    let timeout = *matches.get_one::<u64>("timeout").unwrap();
    let client = IcnClient::with_timeout(api_url, timeout);

    debug!("Created ICN client with timeout: {} seconds", timeout);

    match matches.subcommand() {
        Some(("health", _)) => {
            info!("Checking ICN API health...");
            let health = client.check_health().await?;
            println!("ICN API Status: {}", health.status);
            println!("Version: {}", health.version);
            println!("Uptime: {} seconds", health.uptime);
            if let Some(node_id) = health.node_id {
                println!("Node ID: {}", node_id);
            }
        }
        Some(("identity", identity_matches)) => {
            match identity_matches.subcommand() {
                Some(("create", _)) => {
                    let identity = client.create_identity().await?;
                    println!("✅ Created new identity");
                    println!("DID: {}", identity.did);
                    println!("Public Key: {}", identity.public_key);
                },
                Some(("list", _)) => {
                    let identities = client.list_identities().await?;
                    println!("✅ Found {} identities", identities.len());
                    for identity in identities {
                        println!("DID: {}", identity.did);
                        println!("Public Key: {}", identity.public_key);
                        println!("---");
                    }
                },
                Some(("show", show_matches)) => {
                    let did = show_matches.get_one::<String>("did").unwrap();
                    // Implement show identity details
                    println!("Showing identity details for DID: {}", did);
                    println!("Note: This functionality is not yet implemented.");
                },
                _ => {
                    println!("Unknown identity command. Run 'icn-cli identity --help' for usage information.");
                }
            }
        }
        Some(("cooperative", cooperative_matches)) => {
            match cooperative_matches.subcommand() {
                Some(("join", join_matches)) => {
                    let coop_id = join_matches.get_one::<String>("coop-id").unwrap();
                    client.join_cooperative(coop_id).await?;
                    println!("✅ Successfully joined cooperative {}", coop_id);
                },
                Some(("list", _)) => {
                    let cooperatives = client.list_cooperatives().await?;
                    println!("✅ Found {} cooperatives", cooperatives.len());
                    for coop in cooperatives {
                        println!("ID: {}", coop.id);
                        println!("Name: {}", coop.name);
                        println!("Members: {}", coop.member_count);
                        println!("---");
                    }
                },
                Some(("create", create_matches)) => {
                    let name = create_matches.get_one::<String>("name").unwrap();
                    let description = create_matches.get_one::<String>("description").unwrap();
                    // Implement create cooperative
                    println!("Creating cooperative: {} - {}", name, description);
                    println!("Note: This functionality is not yet implemented.");
                },
                _ => {
                    println!("Unknown cooperative command. Run 'icn-cli cooperative --help' for usage information.");
                }
            }
        }
        Some(("resource", resource_matches)) => {
            match resource_matches.subcommand() {
                Some(("register", register_matches)) => {
                    let resource_type = register_matches.get_one::<String>("resource-type").unwrap();
                    let capacity = register_matches.get_one::<String>("capacity").unwrap();
                    let resource = client.register_resource(resource_type, capacity).await?;
                    println!("✅ Successfully registered resource");
                    println!("ID: {}", resource.id);
                    println!("Type: {}", resource.resource_type);
                    println!("Capacity: {}", resource.capacity);
                },
                Some(("list", _)) => {
                    let resources = client.list_resources().await?;
                    println!("✅ Found {} resources", resources.len());
                    for resource in resources {
                        println!("ID: {}", resource.id);
                        println!("Type: {}", resource.resource_type);
                        println!("Capacity: {}", resource.capacity);
                        println!("Owner: {}", resource.owner);
                        println!("---");
                    }
                },
                _ => {
                    println!("Unknown resource command. Run 'icn-cli resource --help' for usage information.");
                }
            }
        }
        Some(("governance", governance_matches)) => {
            match governance_matches.subcommand() {
                Some(("propose", propose_matches)) => {
                    let title = propose_matches.get_one::<String>("title").unwrap();
                    let description = propose_matches.get_one::<String>("description").unwrap();
                    let proposal = client.create_proposal(title, description).await?;
                    println!("✅ Successfully created proposal");
                    println!("ID: {}", proposal.id);
                    println!("Title: {}", proposal.title);
                    println!("Status: {}", proposal.status);
                },
                Some(("vote", vote_matches)) => {
                    let proposal_id = vote_matches.get_one::<String>("proposal-id").unwrap();
                    let vote = vote_matches.get_one::<String>("vote").unwrap();
                    client.vote_on_proposal(proposal_id, vote).await?;
                    println!("✅ Successfully voted on proposal {}", proposal_id);
                },
                Some(("list", _)) => {
                    // Implement list proposals
                    println!("Listing active proposals...");
                    println!("Note: This functionality is not yet implemented.");
                },
                _ => {
                    println!("Unknown governance command. Run 'icn-cli governance --help' for usage information.");
                }
            }
        }
        Some(("network", network_matches)) => {
            match network_matches.subcommand() {
                Some(("status", status_matches)) => {
                    let detailed = status_matches.get_flag("detail");
                    info!("Checking network status (detailed: {})...", detailed);
                    let status = client.get_network_status(detailed).await?;
                    println!("Network Status: {}", status.status);
                    println!("Peer Count: {}", status.peer_count);
                    println!("Average Latency: {} ms", status.avg_latency);
                    println!("Bandwidth Usage: {:.2} MB/s", status.bandwidth_usage);
                    if let Some(uptime) = status.uptime {
                        println!("Uptime: {} seconds", uptime);
                    }
                    if let Some(version) = status.version {
                        println!("Version: {}", version);
                    }
                }
                Some(("peers", peers_matches)) => {
                    let filter = peers_matches.get_one::<String>("filter").unwrap();
                    let sort = peers_matches.get_one::<String>("sort").unwrap();
                    let output = peers_matches.get_one::<String>("output").unwrap();
                    
                    info!("Listing peers (filter: {}, sort: {}, output: {})...", filter, sort, output);
                    
                    let mut peers = client.list_peers_with_filter(filter).await?;
                    
                    // Sort peers based on the sort parameter
                    match sort.as_str() {
                        "latency" => peers.sort_by(|a, b| a.latency.cmp(&b.latency)),
                        "id" => peers.sort_by(|a, b| a.id.cmp(&b.id)),
                        "address" => peers.sort_by(|a, b| a.address.cmp(&b.address)),
                        _ => {} // Default sort by ID
                    }
                    
                    // Output in the specified format
                    match output.as_str() {
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&peers).unwrap());
                        }
                        "csv" => {
                            println!("ID,Address,Latency,Status,Connected Since");
                            for peer in peers {
                                println!("{},{},{},{},{}", 
                                    peer.id, peer.address, peer.latency, 
                                    peer.status, peer.connected_since);
                            }
                        }
                        _ => { // Default to table
                            if peers.is_empty() {
                                println!("No peers found.");
                            } else {
                                println!("{:<36} {:<15} {:<8} {:<12} {:<20}", 
                                    "ID", "Address", "Latency", "Status", "Connected Since");
                                println!("{}", "-".repeat(95));
                                for peer in peers {
                                    println!("{:<36} {:<15} {:<8} {:<12} {:<20}", 
                                        peer.id, peer.address, format!("{}ms", peer.latency), 
                                        peer.status, peer.connected_since);
                                }
                            }
                        }
                    }
                }
                Some(("connect", connect_matches)) => {
                    let address = connect_matches.get_one::<String>("address").unwrap();
                    info!("Connecting to peer at {}...", address);
                    let peer = client.connect_peer(address).await?;
                    println!("Successfully connected to peer:");
                    println!("ID: {}", peer.id);
                    println!("Address: {}", peer.address);
                    println!("Status: {}", peer.status);
                    println!("Connected Since: {}", peer.connected_since);
                    println!("Latency: {} ms", peer.latency);
                }
                Some(("disconnect", disconnect_matches)) => {
                    let peer_id = disconnect_matches.get_one::<String>("peer-id").unwrap();
                    info!("Disconnecting from peer {}...", peer_id);
                    client.disconnect_peer(peer_id).await?;
                    println!("Successfully disconnected from peer {}", peer_id);
                }
                Some(("ping", ping_matches)) => {
                    let peer_id = ping_matches.get_one::<String>("peer-id").unwrap();
                    let count = *ping_matches.get_one::<u8>("count").unwrap();
                    let interval = *ping_matches.get_one::<u64>("interval").unwrap();
                    
                    info!("Pinging peer {} ({} times with {}ms interval)...", peer_id, count, interval);
                    
                    let results = client.ping_peer_with_interval(peer_id, count, interval).await?;
                    
                    println!("Ping results for peer {}:", peer_id);
                    
                    let mut successful = 0;
                    let mut total_latency = 0;
                    
                    for (i, result) in results.iter().enumerate() {
                        if result.success {
                            println!("Ping {}: {} ms", i + 1, result.latency);
                            successful += 1;
                            total_latency += result.latency;
                        } else {
                            println!("Ping {}: timeout", i + 1);
                        }
                    }
                    
                    if successful > 0 {
                        let avg_latency = total_latency / successful;
                        println!("\n--- {} ping statistics ---", peer_id);
                        println!("{} packets transmitted, {} received, {}% packet loss", 
                            count, successful, ((count as f32 - successful as f32) / count as f32) * 100.0);
                        println!("round-trip min/avg/max = {}/{}/{} ms", 
                            results.iter().filter(|r| r.success).map(|r| r.latency).min().unwrap_or(0),
                            avg_latency,
                            results.iter().filter(|r| r.success).map(|r| r.latency).max().unwrap_or(0));
                    } else {
                        println!("\n--- {} ping statistics ---", peer_id);
                        println!("{} packets transmitted, 0 received, 100% packet loss", count);
                    }
                }
                Some(("diagnostics", diagnostics_matches)) => {
                    let comprehensive = diagnostics_matches.get_flag("comprehensive");
                    let output_file = diagnostics_matches.get_one::<String>("output-file");
                    
                    info!("Running network diagnostics (comprehensive: {})...", comprehensive);
                    
                    let result = client.run_diagnostics_with_options(comprehensive).await?;
                    
                    if let Some(file_path) = output_file {
                        match std::fs::write(file_path, &result) {
                            Ok(_) => println!("Diagnostics results written to {}", file_path),
                            Err(e) => {
                                error!("Failed to write diagnostics to file: {}", e);
                                println!("Failed to write to file: {}", e);
                                println!("Network Diagnostics Results:\n");
                                println!("{}", result);
                            }
                        }
                    } else {
                        println!("Network Diagnostics Results:\n");
                        println!("{}", result);
                    }
                }
                _ => {
                    println!("Unknown network command. Run 'icn-cli network --help' for usage information.");
                }
            }
        }
        _ => {
            println!("No command specified. Run 'icn-cli --help' for usage information.");
        }
    }

    Ok(())
}