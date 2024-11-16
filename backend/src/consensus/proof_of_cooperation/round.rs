// src/consensus/proof_of_cooperation/round.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::blockchain::Block;
use crate::consensus::types::{
    ConsensusRound,
    ConsensusError,
    RoundStatus,
    WeightedVote,
    ConsensusConfig,
    ConsensusRoundStats
};
use super::events::ConsensusEvent;

pub struct RoundManager {
    config: ConsensusConfig,
    current_round: Option<ConsensusRound>,
    round_history: Vec<ConsensusRoundStats>,
    total_voting_power: f64,
}

impl RoundManager {
    pub fn new(config: ConsensusConfig) -> Self {
        RoundManager {
            config,
            current_round: None,
            round_history: Vec::new(),
            total_voting_power: 0.0,
        }
    }

    pub fn start_round(
        &mut self,
        round_number: u64,
        coordinator: String,
        total_voting_power: f64,
        validator_count: usize,
    ) -> Result<ConsensusEvent, ConsensusError> {
        if self.current_round.is_some() {
            return Err(ConsensusError::RoundInProgress);
        }

        self.total_voting_power = total_voting_power;

        let round = ConsensusRound {
            round_number,
            coordinator: coordinator.clone(),
            start_time: Utc::now(),
            timeout: Utc::now() + Duration::milliseconds(self.config.round_timeout_ms as i64),
            status: RoundStatus::Proposing,
            proposed_block: None,
            votes: HashMap::new(),
            stats: ConsensusRoundStats {
                total_voting_power,
                participation_rate: 0.0,
                approval_rate: 0.0,
                round_duration_ms: 0,
                validator_count,
            },
        };

        self.current_round = Some(round);
        Ok(ConsensusEvent::RoundStarted(round_number))
    }

    pub fn propose_block(
        &mut self,
        proposer: &str,
        block: Block,
    ) -> Result<ConsensusEvent, ConsensusError> {
        let mut round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.coordinator != proposer {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidCoordinator);
        }

        if round.status != RoundStatus::Proposing {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        if !block.verify(None).unwrap_or(false) {
            return Err(ConsensusError::ValidationFailed);
        }

        round.proposed_block = Some(block.clone());
        round.status = RoundStatus::Voting;
        
        self.current_round = Some(round);

        Ok(ConsensusEvent::BlockProposed {
            round: round.round_number,
            proposer: proposer.to_string(),
            block_hash: block.hash,
        })
    }

    pub fn submit_vote(
        &mut self,
        validator: String,
        approve: bool,
        voting_power: f64,
        signature: String,
    ) -> Result<ConsensusEvent, ConsensusError> {
        let mut round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.status != RoundStatus::Voting {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        if round.votes.contains_key(&validator) {
            self.current_round = Some(round);
            return Err(ConsensusError::Custom("Already voted".to_string()));
        }

        let vote = WeightedVote {
            validator: validator.clone(),
            approve,
            voting_power,
            timestamp: Utc::now(),
            signature,
        };

        round.votes.insert(validator.clone(), vote);
        self.update_round_stats(&mut round);

        if self.is_consensus_reached(&round) {
            round.status = RoundStatus::Finalizing;
        }

        self.current_round = Some(round);

        Ok(ConsensusEvent::VoteReceived {
            validator,
            approved: approve,
            round: round.round_number,
            voting_power,
        })
    }

    pub fn finalize_round(&mut self) -> Result<(Block, ConsensusRoundStats), ConsensusError> {
        let round = self.current_round.take()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.status != RoundStatus::Finalizing {
            self.current_round = Some(round);
            return Err(ConsensusError::InvalidRoundState);
        }

        let block = round.proposed_block.clone()
            .ok_or_else(|| ConsensusError::Custom("No proposed block".to_string()))?;

        let mut stats = round.stats;
        stats.round_duration_ms = Utc::now()
            .signed_duration_since(round.start_time)
            .num_milliseconds() as u64;

        self.round_history.push(stats.clone());

        Ok((block, stats))
    }

    pub fn check_timeout(&self) -> bool {
        self.current_round.as_ref()
            .map(|round| Utc::now() > round.timeout)
            .unwrap_or(false)
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.current_round.as_ref()
    }

    pub fn get_round_history(&self) -> &[ConsensusRoundStats] {
        &self.round_history
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_round_manager() -> RoundManager {
        RoundManager::new(ConsensusConfig::default())
    }

    #[test]
    fn test_start_round() {
        let mut manager = setup_test_round_manager();
        let result = manager.start_round(
            1,
            "did:icn:test".to_string(),
            1.0,
            3
        );
        assert!(result.is_ok());
        assert!(manager.get_current_round().is_some());
    }

    #[test]
    fn test_vote_submission() {
        let mut manager = setup_test_round_manager();
        
        // Start round
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        
        // Add a block
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        manager.propose_block("did:icn:test", block).unwrap();
        
        // Submit vote
        let result = manager.submit_vote(
            "did:icn:validator1".to_string(),
            true,
            0.3,
            "signature".to_string()
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_consensus_reached() {
        let mut manager = setup_test_round_manager();
        
        // Start round
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        
        // Add block
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        manager.propose_block("did:icn:test", block).unwrap();
        
        // Submit enough votes for consensus
        for i in 1..=3 {
            manager.submit_vote(
                format!("did:icn:validator{}", i),
                true,
                0.3,
                "signature".to_string()
            ).unwrap();
        }
        
        let round = manager.get_current_round().unwrap();
        assert_eq!(round.status, RoundStatus::Finalizing);
    }
}