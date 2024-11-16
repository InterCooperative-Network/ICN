// src/consensus/proof_of_cooperation/events.rs

use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

/// Events emitted during consensus process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusEvent {
    RoundStarted(u64),
    
    BlockProposed {
        round: u64,
        proposer: String,
        block_hash: String,
    },
    
    VoteReceived {
        validator: String,
        approved: bool,
        round: u64,
        voting_power: f64,
    },
    
    RoundCompleted {
        round: u64,
        block_hash: String,
        validators: Vec<String>,
        duration_ms: u64,
    },
    
    ValidationFailed {
        reason: String,
        round: u64,
    },
    
    ReputationUpdated {
        did: String,
        change: i64,
        new_total: i64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_event_serialization() {
        let event = ConsensusEvent::RoundStarted(1);
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.is_empty());

        let event = ConsensusEvent::BlockProposed {
            round: 1,
            proposer: "did:icn:test".to_string(),
            block_hash: "hash".to_string(),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.is_empty());
    }
}