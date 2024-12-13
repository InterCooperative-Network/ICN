
pub mod api;
pub mod claims;
pub mod community;
pub mod cooperative;
pub mod governance;
pub mod monitoring;
pub mod network;
pub mod relationship;


pub use blockchain::{Block, Blockchain, Transaction, TransactionType};
pub use consensus::{ProofOfCooperation, types::ConsensusConfig, types::ConsensusRound};
pub use governance::Proposal;
pub use identity::IdentitySystem;
pub use monitoring::energy::{EnergyAware, EnergyMonitor};
pub use relationship::{
    Contribution, 
    MutualAidInteraction, 
    RelationshipSystem,
    Relationship, 
    RelationshipType,
    Interaction,
    InteractionType,
    Endorsement
};
pub use reputation::ReputationSystem;
pub use vm::{Contract, ExecutionContext, VM};
pub use vm::cooperative_metadata::{CooperativeMetadata, ResourceImpact};
pub use websocket::WebSocketHandler;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid;
use chrono::{DateTime, Utc};

/// Events emitted by the ICN system
#[derive(Clone, Debug)]
pub enum SystemEvent {
    BlockCreated(Block),
    ProposalSubmitted(Proposal),
    VoteCast { proposal_id: u64, voter: String, vote: bool },
    ReputationChanged { did: String, change: i64, reason: String },
    ConsensusStarted(ConsensusRound),
    ConsensusFinished(Block),
    CooperativeCreated { id: String, creator: String },
    CooperativeJoined { id: String, member: String },
    ContributionRecorded(Contribution),
    MutualAidProvided(MutualAidInteraction),
    RelationshipUpdated { member_one: String, member_two: String },
