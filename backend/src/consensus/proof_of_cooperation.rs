// src/consensus/proof_of_cooperation.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::blockchain::Block;
use crate::identity::IdentitySystem;
use crate::reputation::ReputationSystem;
use crate::network::NetworkHandler;
use crate::websocket::WebSocketHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,
    pub start_time: chrono::DateTime<Utc>,
    pub timeout: chrono::DateTime<Utc>,
    pub status: RoundStatus,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, WeightedVote>,
    pub stats: ConsensusRoundStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_validator_reputation: i64,
    pub max_voting_power: f64,
    pub min_participation_rate: f64,
    pub min_approval_rate: f64,
    pub round_timeout_ms: u64,
    pub base_reward: i64,
    pub penalty_factor: f64,
}

pub struct ProofOfCooperation {
    config: ConsensusConfig,
    validators: HashMap<String, ValidatorInfo>,
    current_round: Option<ConsensusRound>,
    round_history: Vec<ConsensusRoundStats>,
    reputation_updates: Vec<(String, i64)>,
    ws_handler: Arc<WebSocketHandler>,
    network: Arc<NetworkHandler>,
    identity_system: Arc<Mutex<IdentitySystem>>,
    reputation_system: Arc<Mutex<ReputationSystem>>,
}

impl ProofOfCooperation {
    pub fn new(
        config: ConsensusConfig,
        ws_handler: Arc<WebSocketHandler>,
        network: Arc<NetworkHandler>,
        identity_system: Arc<Mutex<IdentitySystem>>,
        reputation_system: Arc<Mutex<ReputationSystem>>,
    ) -> Self {
        ProofOfCooperation {
            config,
            validators: HashMap::new(),
            current_round: None,
            round_history: Vec::new(),
            reputation_updates: Vec::new(),
            ws_handler,
            network,
            identity_system,
            reputation_system,
        }
    }

    // Start a new consensus round
    pub async fn start_round(&mut self) -> Result<(), String> {
        if self.current_round.is_some() {
            return Err("Round already in progress".to_string());
        }

        // Get active validators with sufficient reputation
        let active_validators: Vec<_> = self.validators.values()
            .filter(|v| v.reputation >= self.config.min_validator_reputation)
            .collect();

        if active_validators.len() < 3 {
            return Err("Insufficient validators".to_string());
        }

        // Select coordinator based on reputation-weighted lottery
        let coordinator = self.select_coordinator(&active_validators)?;

        // Create new round
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

        // Broadcast round start
        self.ws_handler.broadcast_consensus_update(&round);
        self.network.broadcast_consensus_start(&round).await?;
        
        self.current_round = Some(round);
        Ok(())
    }

    // Process a block proposal from the coordinator
    pub async fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), String> {
        let round = self.current_round.as_mut()
            .ok_or("No active round")?;

        // Verify proposer is the coordinator
        if round.coordinator != proposer_did {
            return Err("Not the round coordinator".to_string());
        }

        // Verify proposer's reputation
        let reputation = self.reputation_system.lock().unwrap()
            .get_reputation(proposer_did);
        if reputation < self.config.min_validator_reputation {
            return Err("Insufficient reputation to propose".to_string());
        }

        // Update round with proposed block
        round.proposed_block = Some(block.clone());
        round.status = RoundStatus::Voting;

        // Broadcast proposal
        self.ws_handler.broadcast_consensus_update(round);
        self.network.broadcast_consensus_proposal(round, &block).await?;

        Ok(())
    }

    // Process a vote from a validator
    pub async fn submit_vote(
        &mut self,
        validator_did: &str,
        approved: bool,
        signature: String
    ) -> Result<(), String> {
        let round = self.current_round.as_mut()
            .ok_or("No active round")?;

        // Verify validator status and reputation
        let validator = self.validators.get(validator_did)
            .ok_or("Not a registered validator")?;
        if validator.reputation < self.config.min_validator_reputation {
            return Err("Insufficient reputation to vote".to_string());
        }

        // Create and register the vote
        let vote = WeightedVote {
            validator: validator_did.to_string(),
            approve: approved,
            voting_power: validator.voting_power,
            timestamp: Utc::now(),
            signature,
        };
        round.votes.insert(validator_did.to_string(), vote.clone());

        // Update round statistics
        self.update_round_stats(round)?;

        // Check if consensus is reached
        if self.check_consensus(round)? {
            round.status = RoundStatus::Finalizing;
            self.finalize_round().await?;
        }

        // Broadcast vote
        self.ws_handler.broadcast_consensus_update(round);
        self.network.broadcast_consensus_vote(round.round_number, validator_did, approved, signature).await?;

        Ok(())
    }

    // Finalize the consensus round
    pub async fn finalize_round(&mut self) -> Result<Block, String> {
        let round = self.current_round.take()
            .ok_or("No active round")?;

        if round.status != RoundStatus::Finalizing {
            return Err("Round not ready for finalization".to_string());
        }

        let block = round.proposed_block
            .ok_or("No proposed block")?;

        // Update validator statistics and reputations
        self.update_validator_stats(&round);
        self.recalculate_voting_power();

        // Record round statistics
        let duration = (Utc::now() - round.start_time).num_milliseconds() as u64;
        let mut stats = round.stats;
        stats.round_duration_ms = duration;
        self.round_history.push(stats);

        // Broadcast finalization
        self.ws_handler.broadcast_block_finalized(&block, round.coordinator);
        self.network.broadcast_block_finalized(&block).await?;

        Ok(block)
    }

    // Helper methods
    fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, String> {
        // Implement reputation-weighted random selection
        let mut rng = rand::thread_rng();

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

    fn update_round_stats(&self, round: &mut ConsensusRound) -> Result<(), String> {
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

        round.stats.total_voting_power = total_voting_power;
        round.stats.participation_rate = votes_power / total_voting_power;
        round.stats.approval_rate = if votes_power > 0.0 {
            approval_power / votes_power
        } else {
            0.0
        };

        Ok(())
    }

    fn check_consensus(&self, round: &ConsensusRound) -> Result<bool, String> {
        Ok(
            round.stats.participation_rate >= self.config.min_participation_rate &&
            round.stats.approval_rate >= self.config.min_approval_rate
        )
    }

    fn update_validator_stats(&mut self, round: &ConsensusRound) {
        for (validator_id, validator) in self.validators.iter_mut() {
            if let Some(vote) = round.votes.get(validator_id) {
                // Reward for participation
                validator.consecutive_missed_rounds = 0;
                validator.last_active_round = round.round_number;
                validator.reputation += self.config.base_reward;

                // Extra reward for coordinator
                if validator_id == &round.coordinator {
                    validator.reputation += self.config.base_reward;
                }

                self.reputation_updates.push((
                    validator_id.clone(),
                    self.config.base_reward
                ));
            } else {
                // Penalty for missing the round
                validator.consecutive_missed_rounds += 1;
                let penalty = -(self.config.base_reward as f64 *
                    self.config.penalty_factor *
                    validator.consecutive_missed_rounds as f64) as i64;
                validator.reputation += penalty;

                self.reputation_updates.push((
                    validator_id.clone(),
                    penalty
                ));
            }

            // Update performance score
            validator.performance_score = validator.performance_score * 0.95 +
                (if round.votes.contains_key(validator_id) { 1.0 } else { 0.0 }) * 0.05;
        }
    }

    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.current_round.as_ref()
    }
}