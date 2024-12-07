mod block;
mod transaction;
mod did;
mod relationship;
mod reputation;

pub use block::Block;
pub use transaction::{Transaction, TransactionType};
pub use did::{DID, DIDDocument};
pub use relationship::{Relationship, RelationshipType};
pub use reputation::ReputationScore;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}
