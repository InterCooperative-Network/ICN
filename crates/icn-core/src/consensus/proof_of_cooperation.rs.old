//! Proof of Cooperation (PoC) Consensus Implementation
//! 
//! This module implements a reputation-based consensus mechanism designed for cooperative networks.
//! Key features include:
//! - Reputation-weighted validator selection
//! - Multi-phase consensus process
//! - Built-in security against Sybil attacks
//! - Real-time consensus monitoring
//! - Performance-based rewards and penalties

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::websocket::WebSocketHandler;
use crate::blockchain::Block;
use crate::identity::DID;
use crate::monitoring::energy::EnergyAware;
use crate::consensus::types::{
    ConsensusConfig, RoundStatus, WeightedVote, ValidatorInfo,
    ConsensusRoundStats, ConsensusRound, ConsensusError
};

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

/// Main consensus implementation
pub struct ProofOfCooperation {
    /// Configuration parameters
    config: ConsensusConfig,
    
    /// Active validators with their information
    validators: HashMap<String, ValidatorInfo>,
    
    /// Current consensus round state
    current_round: Option<ConsensusRound>,
    
    /// History of completed rounds' statistics
    round_history: Vec<ConsensusRoundStats>,
    
    /// Pending reputation updates to be applied
    reputation_updates: Vec<(String, i64)>,
    
    /// WebSocket handler for real-time updates
    ws_handler: Arc<WebSocketHandler>,
    
    /// Broadcast channel for consensus events
    event_tx: broadcast::Sender<ConsensusEvent>,
    
    /// Total network voting power
    total_voting_power: f64,
    
    /// Last round cleanup time
    last_cleanup: DateTime<Utc>,
}

impl ProofOfCooperation {
    /// Creates a new instance of the PoC consensus mechanism
    pub fn new(config: ConsensusConfig, ws_handler: Arc<WebSocketHandler>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        ProofOfCooperation {
            config,
            validators: HashMap::new(),
            current_round: None,
            round_history: Vec::new(),
            reputation_updates: Vec::new(),
            ws_handler,
            event_tx,
            total_voting_power: 0.0,
            last_cleanup: Utc::now(),
        }
    }

    /// Starts a new consensus round
    pub async fn start_round(&mut self) -> Result<(), ConsensusError> {
        // Check if round already in progress
        if self.current_round.is_some() {
            return Err(ConsensusError::RoundInProgress);
        }

        // Clean up inactive validators periodically
        self.cleanup_inactive_validators();

        // Get active validators meeting reputation threshold
        let active_validators: Vec<_> = self.validators.values()
            .filter(|v| self.is_validator_eligible(v))
            .collect();

        if active_validators.len() < self.config.min_validators {
            return Err(ConsensusError::InsufficientValidators);
        }

        // Select coordinator using reputation-weighted random selection
        let coordinator = self.select_coordinator(&active_validators)?;

        // Calculate total voting power for this round
        self.total_voting_power = active_validators.iter()
            .map(|v| v.voting_power)
            .sum();

        // Create new round
        let round = ConsensusRound {
            round_number: self.round_history.len() as u64 + 1,
            coordinator: coordinator.did.clone(),
            start_time: Utc::now(),
            timeout: Utc::now() + Duration::milliseconds(self.config.round_timeout_ms as i64),
            status: RoundStatus::Proposing,
            proposed_block: None,
            votes: HashMap::new(),
            stats: ConsensusRoundStats {
                total_voting_power: self.total_voting_power,
                participation_rate: 0.0,
                approval_rate: 0.0,
                round_duration_ms: 0,
                validator_count: active_validators.len(),
            },
        };

        // Broadcast round start
        self.ws_handler.broadcast_consensus_update(&round);
        let _ = self.event_tx.send(ConsensusEvent::RoundStarted(round.round_number));
        
        self.current_round = Some(round);
        Ok(())
    }

    /// Proposes a new block for the current round
    pub async fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), ConsensusError> {
        // Validate proposer
        let validator = self.validators.get(proposer_did)
            .ok_or(ConsensusError::NotValidator)?;

        if !self.is_validator_eligible(validator) {
            return Err(ConsensusError::InsufficientReputation);
        }

        // Get current round
        let mut round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        // Verify proposer is coordinator
        if round.coordinator != proposer_did {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidCoordinator);
        }

        // Verify round status
        if round.status != RoundStatus::Proposing {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        // Verify block
        if !block.verify(None) {
            return Err(ConsensusError::ValidationFailed);
        }

        // Set proposed block and update status
        round.proposed_block = Some(block.clone());
        round.status = RoundStatus::Voting;

        // Broadcast updates
        self.ws_handler.broadcast_consensus_update(&round);
        self.ws_handler.broadcast_block_finalized(&block);
        let _ = self.event_tx.send(ConsensusEvent::BlockProposed {
            round: round.round_number,
            proposer: proposer_did.to_string(),
            block_hash: block.hash.clone(),
        });

        self.current_round = Some(round);
        Ok(())
    }

    /// Submits a vote for the current round
    pub async fn submit_vote(
        &mut self,
        validator_did: &str,
        approved: bool,
        signature: String
    ) -> Result<(), ConsensusError> {
        // Validate validator
        let validator = self.validators.get(validator_did)
            .ok_or(ConsensusError::NotValidator)?;

        if !self.is_validator_eligible(validator) {
            return Err(ConsensusError::InsufficientReputation);
        }

        // Get current round
        let mut round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        // Verify round status
        if round.status != RoundStatus::Voting {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        // Verify not already voted
        if round.votes.contains_key(validator_did) {
            self.current_round = Some(round);
            return Err(ConsensusError::Custom("Already voted".to_string()));
        }

        // Create vote
        let vote = WeightedVote {
            validator: validator_did.to_string(),
            approve: approved,
            voting_power: validator.voting_power,
            timestamp: Utc::now(),
            signature,
        };

        // Add vote
        round.votes.insert(validator_did.to_string(), vote.clone());

        // Update round statistics
        self.update_round_stats(&mut round);

        // Check if consensus reached
        if self.is_consensus_reached(&round) {
            round.status = RoundStatus::Finalizing;
        }

        // Broadcast updates
        self.ws_handler.broadcast_consensus_update(&round);
        let _ = self.event_tx.send(ConsensusEvent::VoteReceived {
            validator: validator_did.to_string(),
            approved,
            round: round.round_number,
            voting_power: validator.voting_power,
        });

        self.current_round = Some(round);
        Ok(())
    }

    /// Finalizes the current consensus round
    pub async fn finalize_round(&mut self) -> Result<Block, ConsensusError> {
        // Get current round
        let round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        // Verify round status
        if round.status != RoundStatus::Finalizing {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        // Get proposed block
        let block = round.proposed_block.clone()
            .ok_or(ConsensusError::Custom("No proposed block".to_string()))?;

        // Update validator statistics and reputation
        self.update_validator_stats(&round);

        // Update round history
        let mut stats = round.stats;
        stats.round_duration_ms = Utc::now()
            .signed_duration_since(round.start_time)
            .num_milliseconds() as u64;
            
        self.round_history.push(stats);

        // Broadcast completion
        self.ws_handler.broadcast_block_finalized(&block);
        let _ = self.event_tx.send(ConsensusEvent::RoundCompleted {
            round: round.round_number,
            block_hash: block.hash.clone(),
            validators: round.votes.keys().cloned().collect(),
            duration_ms: stats.round_duration_ms,
        });

        Ok(block)
    }

    // Helper Functions

    fn is_validator_eligible(&self, validator: &ValidatorInfo) -> bool {
        validator.reputation >= self.config.min_validator_reputation &&
        validator.performance_score >= self.config.min_performance_score &&
        validator.consecutive_missed_rounds < self.config.max_missed_rounds
    }

    fn update_round_stats(&self, round: &mut ConsensusRound) {
        let votes_power: f64 = round.votes.values()
            .map(|v| v.voting_power)
            .sum();

        round.stats.participation_rate = votes_power / self.total_voting_power;

        let approval_power: f64 = round.votes.values()
            .filter(|v| v.approve)
            .map(|v| v.voting_power)
            .sum();

        round.stats.approval_rate = if votes_power > 0.0 {
            approval_power / votes_power
        } else {
            0.0
        };
    }

    fn is_consensus_reached(&self, round: &ConsensusRound) -> bool {
        round.stats.participation_rate >= self.config.min_participation_rate &&
        round.stats.approval_rate >= self.config.min_approval_rate
    }

    fn update_validator_stats(&mut self, round: &ConsensusRound) {
        for (validator_id, validator) in self.validators.iter_mut() {
            if let Some(vote) = round.votes.get(validator_id) {
                // Reward participation
                validator.consecutive_missed_rounds = 0;
                validator.last_active_round = round.round_number;
                validator.total_blocks_validated += 1;

                // Calculate reward
                let base_reward = if validator_id == &round.coordinator {
                    self.config.base_reward * 2 // Double reward for coordinator
                } else {
                    self.config.base_reward
                };

                let performance_multiplier = if vote.approve {
                    1.0 + (validator.performance_score * 0.5)
                } else {
                    1.0
                };

                let reward = (base_reward as f64 * performance_multiplier) as i64;
                validator.reputation += reward;
                self.reputation_updates.push((validator_id.clone(), reward));

                // Update performance score
                validator.performance_score = validator.performance_score * 0.95 + 0.05;
            } else {
                // Penalize non-participation
                validator.consecutive_missed_rounds += 1;
                
                let penalty = -(self.config.base_reward as f64 *
                    self.config.penalty_factor *
                    validator.consecutive_missed_rounds as f64) as i64;
                
                validator.reputation += penalty;
                self.reputation_updates.push((validator_id.clone(), penalty));
                
                validator.performance_score = validator.performance_score * 0.95;
            }
        }
    }

    fn cleanup_inactive_validators(&mut self) {
        let now = Utc::now();
        if (now - self.last_cleanup).num_hours() >= 24 {
            self.validators.retain(|_, v| {
                v.consecutive_missed_rounds < self.config.max_missed_rounds &&
                v.performance_score >= self.config.min_performance_score
            });
            self.last_cleanup = now;
        }
    }

    /// Selects a coordinator using reputation-weighted random selection
    fn select_coordinator<'a>(&self, active_validators: &'a [&ValidatorInfo]) 
        -> Result<&'a ValidatorInfo, ConsensusError> 
    {
        let mut rng = thread_rng();

        let weights: Vec<f64> = active_validators.iter()
            .map(|v| (v.reputation as f64) * v.performance_score)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err(ConsensusError::Custom("No valid validators".to_string()));
        }

        let selection_point = rng.gen_range(0.0..total_weight);
        let mut cumulative_weight = 0.0;

        for (i, weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if cumulative_weight >= selection_point {
                return Ok(active_validators[i]);
            }
        }

        Err(ConsensusError::Custom("Failed to select coordinator".to_string()))
    }

    // Public interfaces

    /// Gets the current list of reputation updates
    pub fn get_reputation_updates(&self) -> &[(String, i64)] {
        &self.reputation_updates
    }

    /// Gets the current round state if one exists
    pub fn get_current_round(&self) -> Option<ConsensusRound> {
        self.current_round.clone()
    }

    /// Registers a new validator
    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), ConsensusError> {
        let validator = ValidatorInfo {
            did: did.clone(),
            reputation: initial_reputation,
            voting_power: self.calculate_voting_power(initial_reputation),
            last_active_round: 0,
            consecutive_missed_rounds: 0,
            total_blocks_validated: 0,
            performance_score: 1.0,
        };

        self.validators.insert(did, validator);
        Ok(())
    }

    /// Calculates voting power based on reputation
    fn calculate_voting_power(&self, reputation: i64) -> f64 {
        let base_power = (reputation as f64) / 1000.0;
        base_power.min(self.config.max_voting_power)
    }

    /// Gets a broadcast receiver for consensus events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ConsensusEvent> {
        self.event_tx.subscribe()
    }
}

// Implement energy awareness for the consensus mechanism
impl EnergyAware for ProofOfCooperation {
    fn record_energy_metrics(&self, monitor: &crate::monitoring::energy::EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record voting operations
        if let Some(round) = &self.current_round {
            let vote_count = round.votes.len();
            monitor.record_consensus_operation();
            monitor.record_network_operation((vote_count * 256) as u64); // Estimate network usage
        }
        
        // Record validator state size
        let validator_size = (self.validators.len() * std::mem::size_of::<ValidatorInfo>()) as u64;
        monitor.record_memory_operation(validator_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as StdDuration;

    fn setup_test_consensus() -> ProofOfCooperation {
        let config = ConsensusConfig {
            min_validator_reputation: 100,
            max_voting_power: 0.1,
            min_participation_rate: 0.67,
            min_approval_rate: 0.67,
            round_timeout_ms: 30_000,
            base_reward: 10,
            penalty_factor: 1.5,
            min_validators: 3,
            max_missed_rounds: 5,
            min_performance_score: 0.5,
        };
        let ws_handler = Arc::new(WebSocketHandler::new());
        ProofOfCooperation::new(config, ws_handler)
    }

    fn add_test_validators(consensus: &mut ProofOfCooperation) {
        for i in 1..=4 {
            let validator = ValidatorInfo {
                did: format!("did:icn:test{}", i),
                reputation: 1000,
                voting_power: 0.1,
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
        
        let round = consensus.current_round.unwrap();
        assert_eq!(round.status, RoundStatus::Proposing);
        assert!(round.votes.is_empty());
    }

    #[tokio::test]
    async fn test_propose_block() {
        let mut consensus = setup_test_consensus();
        add_test_validators(&mut consensus);
        
        // Start round
        consensus.start_round().await.unwrap();
        let coordinator_did = consensus.current_round.as_ref().unwrap().coordinator.clone();
        
        // Create test block
        let block = Block::new(1, "prev_hash".to_string(), vec![], coordinator_did.clone());
        
        // Propose block
        let result = consensus.propose_block(&coordinator_did, block).await;
        assert!(result.is_ok());
        
        let round = consensus.current_round.unwrap();
        assert_eq!(round.status, RoundStatus::Voting);
        assert!(round.proposed_block.is_some());
    }

    #[tokio::test]
    async fn test_voting_process() {
        let mut consensus = setup_test_consensus();
        add_test_validators(&mut consensus);
        
        // Start round
        consensus.start_round().await.unwrap();
        let coordinator_did = consensus.current_round.as_ref().unwrap().coordinator.clone();
        
        // Propose block
        let block = Block::new(1, "prev_hash".to_string(), vec![], coordinator_did.clone());
        consensus.propose_block(&coordinator_did, block).await.unwrap();
        
        // Submit votes
        for validator in consensus.validators.keys() {
            let result = consensus.submit_vote(
                validator,
                true,
                "test_signature".to_string()
            ).await;
            assert!(result.is_ok());
        }
        
        let round = consensus.current_round.unwrap();
        assert_eq!(round.status, RoundStatus::Finalizing);
        assert!(consensus.is_consensus_reached(&round));
    }

    #[tokio::test]
    async fn test_finalize_round() {
        let mut consensus = setup_test_consensus();
        add_test_validators(&mut consensus);
        
        // Complete full consensus process
        consensus.start_round().await.unwrap();
        let coordinator_did = consensus.current_round.as_ref().unwrap().coordinator.clone();
        
        let block = Block::new(1, "prev_hash".to_string(), vec![], coordinator_did.clone());
        consensus.propose_block(&coordinator_did, block.clone()).await.unwrap();
        
        for validator in consensus.validators.keys() {
            consensus.submit_vote(
                validator,
                true,
                "test_signature".to_string()
            ).await.unwrap();
        }
        
        let result = consensus.finalize_round().await;
        assert!(result.is_ok());
        
        let finalized_block = result.unwrap();
        assert_eq!(finalized_block.hash, block.hash);
        
        // Check reputation updates
        let updates = consensus.get_reputation_updates();
        assert!(!updates.is_empty());
    }

    #[tokio::test]
    async fn test_validator_eligibility() {
        let mut consensus = setup_test_consensus();
        
        // Add validator with low reputation
        consensus.register_validator(
            "did:icn:low_rep".to_string(),
            50  // Below minimum
        ).unwrap();
        
        // Add valid validator
        consensus.register_validator(
            "did:icn:valid".to_string(),
            1000
        ).unwrap();
        
        let low_rep = consensus.validators.get("did:icn:low_rep").unwrap();
        assert!(!consensus.is_validator_eligible(low_rep));
        
        let valid = consensus.validators.get("did:icn:valid").unwrap();
        assert!(consensus.is_validator_eligible(valid));
    }

    #[tokio::test]
    async fn test_performance_score_updates() {
        let mut consensus = setup_test_consensus();
        add_test_validators(&mut consensus);
        
        // Complete multiple rounds
        for _ in 0..3 {
            consensus.start_round().await.unwrap();
            let coordinator_did = consensus.current_round.as_ref().unwrap().coordinator.clone();
            
            let block = Block::new(1, "prev_hash".to_string(), vec![], coordinator_did.clone());
            consensus.propose_block(&coordinator_did, block).await.unwrap();
            
            // Only half validators vote
            for (i, validator) in consensus.validators.keys().enumerate() {
                if i % 2 == 0 {
                    consensus.submit_vote(
                        validator,
                        true,
                        "test_signature".to_string()
                    ).await.unwrap();
                }
            }
            
            consensus.finalize_round().await.unwrap();
        }
        
        // Check performance scores
        for (_, validator) in consensus.validators.iter() {
            if validator.consecutive_missed_rounds > 0 {
                assert!(validator.performance_score < 1.0);
            }
        }
    }
}