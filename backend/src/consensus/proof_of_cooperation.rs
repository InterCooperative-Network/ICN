// backend/src/consensus/proof_of_cooperation.rs

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_validators: usize,
    pub vote_threshold: f64,
    pub round_timeout: u64,
    pub min_reputation: i64,
    pub participation_reward: i64,
    pub coordinator_reward: i64,
    pub missed_validation_penalty: i64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 3,
            vote_threshold: 0.66,
            round_timeout: 60,
            min_reputation: 50,
            participation_reward: 1,
            coordinator_reward: 2,
            missed_validation_penalty: -1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub did: String,
    pub reputation: i64,
    pub last_block_proposed: u64,
    pub consecutive_missed_validations: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Proposing,
    Voting,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, bool>,
    pub status: RoundStatus,
    pub start_time: u64,
    pub timeout: u64,
}

pub struct ProofOfCooperation {
    config: ConsensusConfig,
    validators: HashMap<String, Validator>,
    round_state: ConsensusRound,
    finalized_blocks: Vec<Block>,
    reputation_updates: Vec<(String, i64)>,
    is_round_active: bool,
}

impl ProofOfCooperation {
    pub fn new(config: ConsensusConfig) -> Self {
        ProofOfCooperation {
            config,
            validators: HashMap::new(),
            round_state: ConsensusRound {
                round_number: 0,
                coordinator: String::new(),
                proposed_block: None,
                votes: HashMap::new(),
                status: RoundStatus::Completed,
                start_time: 0,
                timeout: 60,
            },
            finalized_blocks: Vec::new(),
            reputation_updates: Vec::new(),
            is_round_active: false,
        }
    }

    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), String> {
        if initial_reputation < self.config.min_reputation {
            return Err("Insufficient reputation to become validator".to_string());
        }

        let validator = Validator {
            did: did.clone(),
            reputation: initial_reputation,
            last_block_proposed: 0,
            consecutive_missed_validations: 0,
            is_active: true,
        };

        self.validators.insert(did, validator);
        Ok(())
    }

    fn select_coordinator(&self) -> Option<String> {
        let active_validators: Vec<_> = self.validators
            .values()
            .filter(|v| v.is_active && v.reputation >= self.config.min_reputation)
            .collect();

        if active_validators.len() < self.config.min_validators {
            return None;
        }

        let total_reputation: i64 = active_validators.iter().map(|v| v.reputation).sum();
        let mut rng = rand::thread_rng();
        let selection_point = rng.gen_range(0..total_reputation);
        
        let mut cumulative_reputation = 0;
        for validator in active_validators {
            cumulative_reputation += validator.reputation;
            if cumulative_reputation > selection_point {
                return Some(validator.did.clone());
            }
        }
        
        None
    }

    pub fn start_round(&mut self) -> Result<(), String> {
        if self.is_round_active {
            return Err("Consensus round already in progress".to_string());
        }

        let coordinator = self.select_coordinator()
            .ok_or("Unable to select coordinator")?;

        self.round_state = ConsensusRound {
            round_number: self.finalized_blocks.len() as u64 + 1,
            coordinator,
            proposed_block: None,
            votes: HashMap::new(),
            status: RoundStatus::Proposing,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            timeout: self.config.round_timeout,
        };

        self.is_round_active = true;
        Ok(())
    }

    pub fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), String> {
        if !self.is_round_active {
            return Err("No active consensus round".to_string());
        }

        if self.round_state.status != RoundStatus::Proposing {
            return Err("Round not in proposing state".to_string());
        }

        if self.round_state.coordinator != proposer_did {
            return Err("Only the coordinator can propose blocks".to_string());
        }

        self.round_state.proposed_block = Some(block);
        self.round_state.status = RoundStatus::Voting;
        Ok(())
    }

    pub fn submit_vote(&mut self, validator_did: &str, approve: bool) -> Result<(), String> {
        if !self.validators.contains_key(validator_did) {
            return Err("Not a registered validator".to_string());
        }

        if !self.is_round_active {
            return Err("No active consensus round".to_string());
        }

        if self.round_state.status != RoundStatus::Voting {
            return Err("Round not in voting state".to_string());
        }

        self.round_state.votes.insert(validator_did.to_string(), approve);

        if self.check_can_finalize() {
            self.round_state.status = RoundStatus::Finalizing;
        }

        Ok(())
    }

    fn check_can_finalize(&self) -> bool {
        let total_validators = self.validators.values().filter(|v| v.is_active).count();
        let vote_count = self.round_state.votes.len();
        let approval_count = self.round_state.votes.values().filter(|&&v| v).count();

        let participation_ratio = vote_count as f64 / total_validators as f64;
        let approval_ratio = if vote_count > 0 {
            approval_count as f64 / vote_count as f64
        } else {
            0.0
        };

        participation_ratio >= self.config.vote_threshold && 
        approval_ratio >= self.config.vote_threshold
    }

    pub fn finalize_round(&mut self) -> Result<Block, String> {
        if !self.is_round_active {
            return Err("No active consensus round".to_string());
        }

        if self.round_state.status != RoundStatus::Finalizing {
            return Err("Round not ready for finalization".to_string());
        }

        let block = self.round_state.proposed_block.as_ref()
            .ok_or("No proposed block")?
            .clone();

        // Update reputations
        let coordinator = self.round_state.coordinator.clone();
        let votes = self.round_state.votes.clone();

        // Reward voters
        for (voter_did, _) in votes.iter() {
            if let Some(validator) = self.validators.get_mut(voter_did) {
                validator.reputation += self.config.participation_reward;
                validator.consecutive_missed_validations = 0;
                self.reputation_updates.push((
                    voter_did.clone(),
                    self.config.participation_reward
                ));
            }
        }

        // Reward coordinator
        if let Some(validator) = self.validators.get_mut(&coordinator) {
            validator.reputation += self.config.coordinator_reward;
            self.reputation_updates.push((
                coordinator.clone(),
                self.config.coordinator_reward
            ));
        }

        // Penalize non-voters
        for (did, validator) in self.validators.iter_mut() {
            if validator.is_active && !votes.contains_key(did) {
                validator.consecutive_missed_validations += 1;
                let penalty = self.config.missed_validation_penalty * 
                    validator.consecutive_missed_validations as i64;
                
                validator.reputation += penalty;
                self.reputation_updates.push((did.clone(), penalty));

                if validator.reputation < self.config.min_reputation {
                    validator.is_active = false;
                }
            }
        }

        self.finalized_blocks.push(block.clone());
        self.is_round_active = false;
        self.round_state.status = RoundStatus::Completed;

        Ok(block)
    }

    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        if self.is_round_active {
            Some(&self.round_state)
        } else {
            None
        }
    }
}