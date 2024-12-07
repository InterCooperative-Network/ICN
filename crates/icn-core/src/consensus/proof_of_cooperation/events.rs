// src/consensus/proof_of_cooperation/events.rs

use serde::{Serialize, Deserialize};

/// Events emitted during consensus process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusEvent {
    /// Round started event
    RoundStarted {
        round: u64,
        coordinator: String,
        timeout: u64,
    },
    
    /// Block proposed event
    BlockProposed {
        round: u64,
        proposer: String,
        block_hash: String,
        transactions: usize,
    },
    
    /// Vote received event
    VoteReceived {
        round: u64,
        validator: String,
        approve: bool,
        voting_power: f64,
    },
    
    /// Round completed event
    RoundCompleted {
        round: u64,
        block_hash: String,
        validators: Vec<String>,
        duration_ms: u64,
    },
    
    /// Round failed event
    RoundFailed {
        round: u64,
        reason: String,
    },
    
    /// Validator status update
    ValidatorUpdate {
        did: String,
        reputation: i64,
        voting_power: f64,
        performance_score: f64,
    },
}