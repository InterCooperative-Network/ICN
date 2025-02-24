use std::sync::Arc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use icn_types::{Block, Transaction, FederationOperation};

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

use tokio::time::{sleep, Duration};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub approve: bool,
}

pub mod governance;
