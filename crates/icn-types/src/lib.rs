mod block;
mod transaction;
mod identity;
mod relationship;
mod reputation;

pub use block::Block;
pub use transaction::{Transaction, TransactionType};
pub use identity::DID;
pub use relationship::{Relationship, RelationshipType};
pub use reputation::{ReputationSystem, ReputationContext, ReputationScore};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Validate {
    fn validate(&self) -> Result<()>;
}
