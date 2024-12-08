// crates/icn-consensus/src/proof_of_cooperation/types.rs
use std::time::Instant;
use serde::{Serialize, Deserialize};
use icn_types::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub did: String,
    pub reputation: i64,
    pub last_active: i64,
    pub voting_power: f64,
}

impl ValidatorInfo {
    pub fn new(did: String, reputation: i64) -> Self {
        Self {
            did,
            reputation,
            last_active: chrono::Utc::now().timestamp(),
            voting_power: reputation as f64 / 100.0,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_active = chrono::Utc::now().timestamp();
    }

    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.last_active < 300 // 5 minutes timeout
    }
}

#[derive(Debug)]
pub struct RoundState {
    pub round_number: u64,
    pub start_time: Instant,
    pub coordinator: Option<String>,
    pub proposed_block: Option<Block>,
    pub votes: Vec<(String, bool)>,
    pub round_complete: bool,
}

impl RoundState {
    pub fn new(round_number: u64) -> Self {
        Self {
            round_number,
            start_time: Instant::now(),
            coordinator: None,
            proposed_block: None,
            votes: Vec::new(),
            round_complete: false,
        }
    }

    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        self.duration() > timeout
    }
}