use std::sync::Arc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use icn_types::{Block, Transaction, FederationOperation};
use icn_common::{ConsensusEngine, ReputationManager};

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

// Re-export main components
pub use self::{
    core::Core,
    storage::StorageInterface,
    networking::NetworkInterface,
    identity::IdentityInterface,
    reputation::ReputationInterface,
    vm::RuntimeInterface,
    telemetry::{TelemetryManager, PrometheusMetrics, Logger, TracingSystem},
    models::{ResourceAllocationSystem, FederationManager, ResourceAllocation},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub votes_for: i64,
    pub votes_against: i64,
    pub created_by: String,
    pub ends_at: String,
}

pub mod governance;
