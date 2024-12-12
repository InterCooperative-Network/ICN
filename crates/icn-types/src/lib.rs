// File: crates/icn-types/src/lib.rs
//
// Core types for the Inter-Cooperative Network. This module provides the fundamental
// data structures and type definitions used throughout the ICN system.

mod block;
mod error;
mod transaction;
mod identity;
mod relationship;
mod state;

pub use block::{Block, BlockHeader, BlockMetadata};
pub use error::{Error as TypesError, Result as TypesResult};
pub use transaction::{Transaction, TransactionType, TransactionResult};
pub use identity::{Identity, DID, KeyPair, PublicKey, Signature};
pub use relationship::{Relationship, RelationshipType, RelationshipProof};
pub use state::{NetworkState, StateTransition, StateRoot};

/// Core validation trait implemented by all domain types
pub trait Validate {
    /// Validate the type instance
    fn validate(&self) -> TypesResult<()>;
}

/// Serialization helper trait
pub trait Serializable: serde::Serialize + serde::de::DeserializeOwned {
    /// Convert to JSON string
    fn to_json(&self) -> TypesResult<String> {
        serde_json::to_string(self)
            .map_err(|e| TypesError::Serialization(e.to_string()))
    }

    /// Create from JSON string
    fn from_json(json: &str) -> TypesResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| TypesError::Deserialization(e.to_string()))
    }
}

// Automatically implement Serializable for types that can be serialized
impl<T: serde::Serialize + serde::de::DeserializeOwned> Serializable for T {}