// src/consensus/proof_of_cooperation.rs

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use rand::Rng;
use crate::blockchain::Block;
use crate::consensus::types::*;
use crate::websocket::WebSocketHandler;

pub struct ProofOfCooperation {
    config: ConsensusConfig,
    validators: HashMap<String, ValidatorInfo>,
    current_round: Option<ConsensusRound>,
    round_history: Vec<ConsensusRoundStats>,
    reputation_updates: Vec<(String, i64)>,
    ws_handler: Arc<WebSocketHandler>,
}

impl ProofOfCooperation {
    pub fn new(config: ConsensusConfig, ws_handler: Arc<WebSocketHandler>) -> Self {
        ProofOfCooperation {
            config,
            validators: HashMap::new(),
            current_round: None,
            round_history: Vec::new(),
            reputation_updates: Vec::new(),
            ws_handler,
        }
    }

    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), ConsensusError> {
        if initial_reputation < self.config.min_validator_reputation {
            return Err(ConsensusError::InsufficientReputation);
        }

        let validator = ValidatorInfo {
            did: did.clone(),
            reputation: initial_reputation,
            voting_power: 0.0,
            last_active_round: 0,
            consecutive_missed_rounds: 0,
            total_blocks_validated: 0,
            performance_score: 1.0,
        };

        self.validators.insert(did.clone(), validator.clone());
        self.recalculate_voting_power();
        
        self.ws_handler.broadcast_validator_update(
            validator,
            self.round_history.len() as u64,
            "registered".to_string()
        );
        
        Ok(())
    }

    fn recalculate_voting_power(&mut self) {
        let total_reputation: i64 = self.validators.values()
            .map(|v| v.reputation)
            .sum();

        if total_reputation <= 0 {
            return;
        }

        for validator in self.validators.values_mut() {
            let raw_power = validator.reputation as f64 / total_reputation as f64;
            validator.voting_power = raw_power.min(self.config.max_voting_power);
        }
    }

    pub fn start_round(&mut self) -> Result<(), ConsensusError> {
        if self.current_round.is_some() {
            return Err(ConsensusError::RoundInProgress);
        }

        let active_validators: Vec<_> = self.validators.values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation)
            .collect();

        if active_validators.len() < 3 {
            return Err(ConsensusError::InsufficientValidators);
        }

        let coordinator = self.select_coordinator(&active_validators)?;

        let round = ConsensusRound {
            round_number: self.round_history.len() as u64 + 1,
            coordinator: coordinator.did.clone(),
            start_time: Utc::now(),
            timeout: Utc::now() + chrono::Duration::milliseconds(self.config.round_timeout_ms as i64),
            status: RoundStatus::Proposing,
            proposed_block: None,
            votes: HashMap::new(),
            stats: ConsensusRoundStats {
                total_voting_power: 0.0,
                participation_rate: 0.0,
                approval_rate: 0.0,
                round_duration_ms: 0,
                validator_count: active_validators.len(),
            },
        };

        self.ws_handler.broadcast_consensus_update(&round);
        self.current_round = Some(round);
        Ok(())
    }

    fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, ConsensusError> {
        let mut rng = rand::thread_rng();

        let weights: Vec<f64> = active_validators.iter()
            .map(|v| (v.reputation as f64) * v.performance_score)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err(ConsensusError::InvalidCoordinator);
        }

        let selection_point = rng.gen_range(0.0..total_weight);

        let mut cumulative_weight = 0.0;
        for (i, weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if cumulative_weight >= selection_point {
                return Ok(active_validators[i]);
            }
        }

        Err(ConsensusError::InvalidCoordinator)
    }

    pub fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), ConsensusError> {
        let round = self.current_round.as_mut()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.status != RoundStatus::Proposing {
            return Err(ConsensusError::InvalidRoundState);
        }

        if round.coordinator != proposer_did {
            return Err(ConsensusError::InvalidCoordinator);
        }

        round.proposed_block = Some(block);
        round.status = RoundStatus::Voting;
        
        self.ws_handler.broadcast_consensus_update(round);
        Ok(())
    }

    pub fn submit_vote(&mut self, validator_did: &str, approve: bool) -> Result<(), ConsensusError> {
        // First validate the validator
        let validator = match self.validators.get(validator_did) {
            Some(v) => v,
            None => return Err(ConsensusError::NotValidator),
        };

        if validator.reputation < self.config.min_validator_reputation {
            return Err(ConsensusError::InsufficientReputation);
        }

        let voting_power = validator.voting_power;

        // Then handle the round
        if let Some(round) = self.current_round.as_mut() {
            if round.status != RoundStatus::Voting {
                return Err(ConsensusError::InvalidRoundState);
            }

            // Create and register the vote
            let vote = WeightedVote {
                validator: validator_did.to_string(),
                approve,
                voting_power,
                timestamp: Utc::now(),
                signature: "".to_string(),
            };
            round.votes.insert(validator_did.to_string(), vote);

            // Calculate consensus values
            let total_voting_power: f64 = self.validators.values()
                .filter(|v| v.reputation >= self.config.min_validator_reputation)
                .map(|v| v.voting_power)
                .sum();

            let votes_power: f64 = round.votes.values()
                .map(|v| v.voting_power)
                .sum();

            let approval_power: f64 = round.votes.values()
                .filter(|v| v.approve)
                .map(|v| v.voting_power)
                .sum();

            // Update round statistics
            round.stats.participation_rate = votes_power / total_voting_power;
            round.stats.approval_rate = if votes_power > 0.0 {
                approval_power / votes_power
            } else {
                0.0
            };

            // Check if consensus is reached
            if round.stats.participation_rate >= self.config.min_participation_rate 
                && round.stats.approval_rate >= self.config.min_approval_rate {
                round.status = RoundStatus::Finalizing;
            }

            self.ws_handler.broadcast_consensus_update(round);
            Ok(())
        } else {
            Err(ConsensusError::NoActiveRound)
        }
    }

    pub fn finalize_round(&mut self) -> Result<Block, ConsensusError> {
        let round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.status != RoundStatus::Finalizing {
            self.current_round = Some(round);  // Restore the round if we're not ready
            return Err(ConsensusError::InvalidRoundState);
        }

        self.update_validator_stats(&round);
        self.recalculate_voting_power();

        let duration = round.duration_ms();
        let mut stats = round.stats;
        stats.round_duration_ms = duration as u64;
        self.round_history.push(stats);

        let block = round.proposed_block.ok_or(ConsensusError::Custom(
            "No proposed block found".to_string()
        ))?;

        self.ws_handler.broadcast_block_finalized(&block, round.coordinator.clone());

        Ok(block)
    }

    fn update_validator_stats(&mut self, round: &ConsensusRound) {
        let active_validators: Vec<_> = self.validators.values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation)
            .map(|v| v.did.clone())
            .collect();

        // Update coordinator stats
        if let Some(coordinator) = self.validators.get_mut(&round.coordinator) {
            coordinator.total_blocks_validated += 1;
            let reward = self.config.base_reward * 2;
            coordinator.reputation += reward;
            
            self.ws_handler.broadcast_reputation_update(
                round.coordinator.clone(),
                reward,
                coordinator.reputation,
                "coordinator_reward".to_string()
            );
            
            self.reputation_updates.push((round.coordinator.clone(), reward));
        }

        // Update all validator stats
        for validator_did in active_validators {
            let participated = round.votes.contains_key(&validator_did);
            
            if let Some(validator) = self.validators.get_mut(&validator_did) {
                if participated {
                    validator.consecutive_missed_rounds = 0;
                    validator.last_active_round = round.round_number;
                    validator.reputation += self.config.base_reward;
                    
                    self.ws_handler.broadcast_reputation_update(
                        validator_did.clone(),
                        self.config.base_reward,
                        validator.reputation,
                        "participation_reward".to_string()
                    );
                    
                    self.reputation_updates.push((validator_did.clone(), self.config.base_reward));
                } else {
                    validator.consecutive_missed_rounds += 1;
                    let penalty = -(self.config.base_reward as f64 * 
                        self.config.penalty_factor * 
                        validator.consecutive_missed_rounds as f64) as i64;
                    validator.reputation += penalty;
                    
                    self.ws_handler.broadcast_reputation_update(
                        validator_did.clone(),
                        penalty,
                        validator.reputation,
                        "missed_round_penalty".to_string()
                    );
                    
                    self.reputation_updates.push((validator_did.clone(), penalty));
                }

                let participation_weight = if participated { 1.0 } else { 0.8 };
                validator.performance_score = validator.performance_score * 0.95 + participation_weight * 0.05;
                
                self.ws_handler.broadcast_validator_update(
                    validator.clone(),
                    round.round_number,
                    if participated { "participated" } else { "missed" }.to_string()
                );
            }
        }
    }

    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.current_round.as_ref()
    }

    pub fn get_round_history(&self) -> &[ConsensusRoundStats] {
        &self.round_history
    }
}