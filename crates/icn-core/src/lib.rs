// Only keep needed imports
use icn_types::{RuntimeInterface};

// Module declarations
pub mod blockchain;
pub mod core;
pub mod db;
pub mod identity;
pub mod reputation;
pub mod storage;
pub mod vm;
pub mod networking;
pub mod models;
pub mod telemetry;
pub mod verifiable_credentials;
pub mod zk_snarks;

// Re-export main interfaces
pub use self::{
    storage::StorageInterface,
    networking::NetworkInterface,
    identity::IdentityInterface,
    reputation::ReputationInterface,
    telemetry::TelemetryManager,
    models::{ResourceAllocationSystem, FederationManager},
};

pub mod governance;
