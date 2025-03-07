pub mod api;
pub mod services;
pub mod database;
pub mod networking;
pub mod core;
pub mod main;

pub mod db;
pub mod models;
pub mod blockchain;
pub mod federation;
pub mod governance;
pub mod identity;
pub mod notification;
pub mod reputation;
pub mod resources;
pub mod vm;

// Re-export for the integration tests
pub use blockchain::Blockchain;
pub use governance::{Proposal, ProposalType, ProposalStatus, ProposalHistory, Federation, FederationType, FederationTerms, MemberRole, MemberStatus, handle_federation_operation};
pub use identity::{DID, Algorithm, DIDError, IdentitySystem};
pub use reputation::ReputationSystem;
pub use vm::{VM, opcode, cooperative_metadata};
pub use notification::NotificationManager;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub mod test_config;
#[cfg(test)]
pub mod test_macros;

// Re-export test utilities for integration tests
#[cfg(test)]
pub use test_utils::*;
#[cfg(test)]
pub use test_config::*;
#[cfg(test)]
pub use test_macros::*;
