use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};

use crate::websocket::WebSocketHandler;
use crate::blockchain::Block;
use crate::consensus::types::{
    ConsensusConfig,
    RoundStatus,
    WeightedVote,
    ValidatorInfo,
    ConsensusRoundStats,
    ConsensusRound as ConsensusRoundType,
};

#[derive(Clone, Debug)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,
    pub start_time: DateTime<Utc>,
    pub timeout: DateTime<Utc>,
    pub status: RoundStatus,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, WeightedVote>,
    pub stats: ConsensusRoundStats,
}

impl From<ConsensusRound> for ConsensusRoundType {
    fn from(round: ConsensusRound) -> Self {
        ConsensusRoundType {
            round_number: round.round_number,
            coordinator: round.coordinator,
            start_time: round.start_time,
            timeout: round.timeout,
            status: round.status,
            proposed_block: round.proposed_block,
            votes: round.votes,
            stats: round.stats,
        }
    }
}

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

    /// Starts a new consensus round if one is not already in progress.
    pub async fn start_round(&mut self) -> Result<(), String> {
        if self.current_round.is_some() {
            return Err("Round already in progress".to_string());
        }

        let active_validators: Vec<_> = self.validators.values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation)
            .collect();

        if active_validators.len() < 3 {
            return Err("Insufficient validators".to_string());
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

        // Broadcast start of round
        let round_type = ConsensusRoundType::from(round.clone());
        self.ws_handler.broadcast_consensus_update(&round_type);

        self.current_round = Some(round);
        Ok(())
    }

    /// Allows the coordinator to propose a block for validation.
    pub async fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), String> {
        let validator = self.validators.get(proposer_did)
            .ok_or("Proposer not found")?;

        if validator.reputation < self.config.min_validator_reputation {
            return Err("Insufficient reputation to propose".to_string());
        }

        let mut round = self.current_round.take()
            .ok_or("No active round")?;

        if round.coordinator != proposer_did {
            self.current_round = Some(round);
            return Err("Not the round coordinator".to_string());
        }

        round.proposed_block = Some(block.clone());
        round.status = RoundStatus::Voting;

        // Broadcast updates
        let round_type = ConsensusRoundType::from(round.clone());
        self.ws_handler.broadcast_consensus_update(&round_type);
        self.ws_handler.broadcast_block_finalized(&block);

        self.current_round = Some(round);
        Ok(())
    }

    /// Allows a validator to submit a vote on the proposed block.
    pub async fn submit_vote(&mut self, validator_did: &str, approved: bool, signature: String) -> Result<(), String> {
        let validator = self.validators.get(validator_did)
            .ok_or("Not a registered validator")?;

        if validator.reputation < self.config.min_validator_reputation {
            return Err("Insufficient reputation to vote".to_string());
        }

        let vote = WeightedVote {
            validator: validator_did.to_string(),
            approve: approved,
            voting_power: validator.voting_power,
            timestamp: Utc::now(),
            signature,
        };

        let mut round = self.current_round.take()
            .ok_or("No active round")?;

        round.votes.insert(validator_did.to_string(), vote);

        let total_power: f64 = self.validators.values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation)
            .map(|v| v.voting_power)
            .sum();

        let votes_power: f64 = round.votes.values()
            .map(|v| v.voting_power)
            .sum();

        round.stats.total_voting_power = total_power;
        round.stats.participation_rate = votes_power / total_power;

        let approval_power: f64 = round.votes.values()
            .filter(|v| v.approve)
            .map(|v| v.voting_power)
            .sum();

        round.stats.approval_rate = if votes_power > 0.0 {
            approval_power / votes_power
        } else {
            0.0
        };

        if round.stats.participation_rate >= self.config.min_participation_rate
            && round.stats.approval_rate >= self.config.min_approval_rate {
            round.status = RoundStatus::Finalizing;
        }

        let round_type = ConsensusRoundType::from(round.clone());
        self.ws_handler.broadcast_consensus_update(&round_type);
        self.ws_handler.broadcast_validator_update(
            validator.clone(),
            round.round_number,
            if approved { "approved".to_string() } else { "rejected".to_string() }
        );

        self.current_round = Some(round);
        Ok(())
    }

    /// Finalizes the current round if it has reached the finalizing status and returns the block.
    pub async fn finalize_round(&mut self) -> Result<Block, String> {
        let round = self.current_round.take()
            .ok_or("No active round")?;

        if round.status != RoundStatus::Finalizing {
            self.current_round = Some(round);
            return Err("Round not ready for finalization".to_string());
        }

        let block = round.proposed_block.clone()
            .ok_or("No proposed block")?;

        // Update validator stats
        for (validator_id, validator) in self.validators.iter_mut() {
            if round.votes.contains_key(validator_id) {
                validator.consecutive_missed_rounds = 0;
                validator.last_active_round = round.round_number;
                validator.reputation += self.config.base_reward;

                if validator_id == &round.coordinator {
                    validator.reputation += self.config.base_reward;
                }

                self.reputation_updates.push((
                    validator_id.clone(),
                    self.config.base_reward
                ));

                validator.performance_score = validator.performance_score * 0.95 + 0.05;
            } else {
                validator.consecutive_missed_rounds += 1;
                let penalty = -(self.config.base_reward as f64 *
                    self.config.penalty_factor *
                    validator.consecutive_missed_rounds as f64) as i64;
                validator.reputation += penalty;

                self.reputation_updates.push((
                    validator_id.clone(),
                    penalty
                ));

                validator.performance_score = validator.performance_score * 0.95;
            }
        }

        // Update history
        let mut stats = round.stats;
        stats.round_duration_ms = round.timeout
            .signed_duration_since(round.start_time)
            .num_milliseconds() as u64;
        self.round_history.push(stats);

        // Broadcast completion - only passing block
        self.ws_handler.broadcast_block_finalized(&block);

        Ok(block)
    }

    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.current_round.as_ref()
    }

    fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, String> 
    {
        let mut rng = thread_rng();

        let weights: Vec<f64> = active_validators.iter()
            .map(|v| (v.reputation as f64) * v.performance_score)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err("No valid validators".to_string());
        }

        let selection_point = rng.gen_range(0.0..total_weight);
        let mut cumulative_weight = 0.0;

        for (i, weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if cumulative_weight >= selection_point {
                return Ok(active_validators[i]);
            }
        }

        Err("Failed to select coordinator".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::types::ConsensusConfig;

    fn setup_test_consensus() -> ProofOfCooperation {
        let config = ConsensusConfig::default();
        let ws_handler = Arc::new(WebSocketHandler::new());
        ProofOfCooperation::new(config, ws_handler)
    }

    fn add_test_validators(consensus: &mut ProofOfCooperation) {
        for i in 1..=3 {
            let validator = ValidatorInfo {
                did: format!("did:icn:test{}", i),
                reputation: 1000,
                voting_power: 1.0,
                last_active_round: 0,
                consecutive_missed_rounds: 0,
                total_blocks_validated: 0,
                performance_score: 1.0,
            };
            consensus.validators.insert(validator.did.clone(), validator);
        }
    }

    #[tokio::test]
    async fn test_start_round() {
        let mut consensus = setup_test_consensus();
        add_test_validators(&mut consensus);
        
        assert!(consensus.start_round().await.is_ok());
        assert!(consensus.current_round.is_some());
    }

    #[tokio::test]
    async fn test_insufficient_validators() {
        let mut consensus = setup_test_consensus();
        
        let result = consensus.start_round().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient validators");
    }
}
