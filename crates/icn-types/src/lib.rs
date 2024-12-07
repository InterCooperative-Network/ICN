// crates/icn-types/src/lib.rs

mod block;
mod transaction;
mod identity;
mod relationship;
mod reputation;
mod error;  // Add this line

pub use block::Block;
pub use transaction::{Transaction, TransactionType};
pub use identity::DID;
pub use relationship::{Relationship, RelationshipType};
pub use reputation::{ReputationSystem, ReputationContext, ReputationScore};
// Export error types
pub use error::{
    CoreError, StorageError, ConsensusError, NetworkError, IdentityError,
    CoreResult, StorageResult, ConsensusResult, NetworkResult, IdentityResult,
};

pub trait Validate {
    fn validate(&self) -> CoreResult<()>;  // Updated to use CoreResult
}