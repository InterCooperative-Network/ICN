// crates/icn-consensus/src/proof_of_cooperation/mod.rs
mod types;
mod engine;
mod validator;
mod round;
mod events;

pub use types::{ValidatorInfo, RoundState};
pub use engine::ProofOfCooperation;
pub use validator::ValidatorManager;
pub use round::RoundManager;
pub use events::{ConsensusEvent, EventManager};

use crate::ConsensusConfig;
use icn_types::{Block, ConsensusError, ConsensusResult};