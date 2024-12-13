use icn_types::{Block, DID};
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};

pub struct Validator {
    pub did: DID,
    pub reputation: f64,
    pub last_proposed_block: Option<DateTime<Utc>>,
    pub last_voted_round: Option<u64>,
}

impl Validator {
    pub fn new(did: DID) -> Self {
        Self {
            did,
            reputation: 1.0,
            last_proposed_block: None,
            last_voted_round: None,
        }
    }

    pub fn update_reputation(&mut self, delta: f64) {
        self.reputation = (self.reputation + delta).clamp(0.0, 1.0);
    }

    pub fn can_propose(&self, current_time: DateTime<Utc>, cooldown: Duration) -> bool {
        match self.last_proposed_block {
            None => true,
            Some(last_time) => current_time - last_time >= cooldown
        }
    }
}
