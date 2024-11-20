// src/relationship/mod.rs

mod types;
mod models;
mod system;

// Re-export everything publicly
pub use types::RelationshipType;

pub use models::{
    Contribution,
    Feedback,
    EndorsementType,
    MutualAidInteraction,
    Relationship,
    Interaction,
    InteractionType,
    Endorsement,
    RelationshipNote,
    Visibility,
};

pub use system::RelationshipSystem;

// Re-export EnergyAware trait for implementations
pub(crate) use crate::monitoring::energy::EnergyAware;
pub(crate) use crate::monitoring::energy::EnergyMonitor;