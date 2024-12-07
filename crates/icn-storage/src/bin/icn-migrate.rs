// src/bin/icn-migrate.rs
use icn_storage::{StorageManager, state::migrations::{Migration, Migrator}};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize connection
    let storage = Arc::new(StorageManager::new(None).await?);
    let migrator = Migrator::new(storage);

    println!("Applying initial migration...");
    migrator.apply_migration(&Migration::initial_schema()).await?;
    println!("Initial migration complete!");

    Ok(())
}