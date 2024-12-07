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

pub struct ValidatorSet {
    validators: HashMap<DID, Validator>,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    pub fn add(&mut self, did: DID) {
        if !self.validators.contains_key(&did) {
            self.validators.insert(did.clone(), Validator::new(did));
        }
    }

    pub fn get_mut(&mut self, did: &DID) -> Option<&mut Validator> {
        self.validators.get_mut(did)
    }

    pub fn select_proposer(&self) -> Option<DID> {
        if self.validators.is_empty() {
            return None;
        }

        let total_reputation: f64 = self.validators.values()
            .map(|v| v.reputation)
            .sum();

        let mut rng = thread_rng();
        let selection = rng.gen_range(0.0..total_reputation);
        
        let mut cumulative = 0.0;
        for (did, validator) in &self.validators {
            cumulative += validator.reputation;
            if cumulative >= selection {
                return Some(did.clone());
            }
        }

        self.validators.keys().next().cloned()
    }
}
