use std::sync::Arc;
use log::{info, error, debug};
use icn_types::StorageBackend;
use icn_core::{
    Core, 
    TelemetryManager, 
    PrometheusMetrics,
    Logger, 
    TracingSystem,
    RuntimeManager,
    StorageManager,
    NetworkManager,
    IdentityManager,
    ReputationManager,
    FederationManager,
    ResourceAllocationSystem,
    storage::InMemoryStorageBackend
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting ICN node initialization...");

    // Initialize core components
    let storage_manager = StorageManager::new(Box::new(create_storage_backend()));
    let network_manager = NetworkManager::new();
    let runtime_manager = RuntimeManager::new();
    let prometheus = PrometheusMetrics::new();
    let logger = Logger::new();
    let tracing = TracingSystem::new();
    let telemetry_manager = TelemetryManager::new(prometheus, logger, tracing);
    let identity_manager = IdentityManager::new();
    let reputation_manager = ReputationManager::new();
    let resource_system = Arc::new(ResourceAllocationSystem::new());
    let federation_manager = Arc::new(FederationManager::new(resource_system.clone()));

    // Wrap components in Arc for shared ownership
    let storage = Arc::new(storage_manager);
    let network = Arc::new(network_manager);
    let runtime = Arc::new(runtime_manager);
    let telemetry = Arc::new(telemetry_manager);
    let identity = Arc::new(identity_manager);
    let reputation = Arc::new(reputation_manager);

    // Create core system
    let core = Core::new(
        storage.clone(),
        network.clone(),
        runtime.clone(),
        telemetry.clone(),
        identity.clone(),
        reputation.clone()
    );

    // Start the core system
    info!("Starting core system components...");
    match core.start().await {
        Ok(_) => info!("Core system started successfully"),
        Err(e) => {
            error!("Failed to start core system: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize P2P networking
    info!("Initializing P2P networking...");
    if let Err(e) = network.start().await {
        error!("Failed to start P2P networking: {}", e);
        std::process::exit(1);
    }

    // Start consensus engine
    info!("Starting consensus engine...");
    if let Err(e) = core.start_consensus().await {
        error!("Failed to start consensus engine: {}", e);
        std::process::exit(1);
    }

    // Start federation manager
    info!("Starting federation manager...");
    if let Err(e) = federation_manager.start().await {
        error!("Failed to start federation manager: {}", e);
        std::process::exit(1);
    }

    // Set up signal handlers for graceful shutdown
    let core_clone = core.clone();
    let network_clone = network.clone();
    let federation_clone = federation_manager.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("Received shutdown signal, initiating graceful shutdown...");
        
        // Shutdown sequence
        if let Err(e) = federation_clone.stop().await {
            error!("Error stopping federation manager: {}", e);
        }
        if let Err(e) = network_clone.stop().await {
            error!("Error stopping P2P networking: {}", e);
        }
        if let Err(e) = core_clone.stop().await {
            error!("Error stopping core system: {}", e);
        }
        
        info!("Node shutdown completed");
        std::process::exit(0);
    });

    info!("ICN node startup complete - running...");

    // Keep the main thread alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

fn create_storage_backend() -> impl StorageBackend {
    InMemoryStorageBackend::new()
}