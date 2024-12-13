use std::collections::HashMap;
use chrono::{Utc, Duration};
use crate::blockchain::Block;
use crate::consensus::types::{
    ConsensusRound,
    ConsensusError,
    RoundStatus,
    WeightedVote,
    ConsensusConfig,
    ConsensusRoundStats
};
use crate::consensus::proof_of_cooperation::events::ConsensusEvent;

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

        Ok(ConsensusEvent::RoundStarted { 
            round: round_number,
            coordinator,
            timeout: self.config.round_timeout_ms,
        })
    }

    pub fn propose_block(
        &mut self,
        proposer: &str,
        block: Block,
    ) -> Result<ConsensusEvent, ConsensusError> {
        let round = self.current_round.as_mut()
            .ok_or(ConsensusError::NoActiveRound)?;

        if round.coordinator != proposer {
            return Err(ConsensusError::InvalidCoordinator);
        }

        if round.status != RoundStatus::Proposing {
            return Err(ConsensusError::InvalidRoundState);
        }

        let event = ConsensusEvent::BlockProposed {
            round: round.round_number,
            proposer: proposer.to_string(),
            block_hash: block.hash.clone(),
            transactions: block.transactions.len(),
        };

        round.proposed_block = Some(block);
        round.status = RoundStatus::Voting;

        Ok(event)
    }

    pub fn submit_vote(
        &mut self,
        validator: String,
        approve: bool,
        voting_power: f64,
        signature: String,
    ) -> Result<ConsensusEvent, ConsensusError> {
        // First get all the data we need from the current state
        let round_number;
        let current_votes_power: f64;
        let current_approval_power: f64;
        {
            let round = self.current_round.as_ref()
                .ok_or(ConsensusError::NoActiveRound)?;

            if round.status != RoundStatus::Voting {
                return Err(ConsensusError::InvalidRoundState);
            }

            if round.votes.contains_key(&validator) {
                return Err(ConsensusError::Custom("Already voted".to_string()));
            }

            round_number = round.round_number;
            current_votes_power = round.votes.values()
                .map(|v| v.voting_power)
                .sum();
            current_approval_power = round.votes.values()
                .filter(|v| v.approve)
                .map(|v| v.voting_power)
                .sum();
        }

        // Create the new vote
        let vote = WeightedVote {
            validator: validator.clone(),
            approve,
            voting_power,
            timestamp: Utc::now(),
            signature,
        };

        // Calculate new stats
        let new_total_power = current_votes_power + voting_power;
        let new_approval_power = if approve {
            current_approval_power + voting_power
        } else {
            current_approval_power
        };

        let participation_rate = new_total_power / self.total_voting_power;
        let approval_rate = if new_total_power > 0.0 {
            new_approval_power / new_total_power
        } else {
            0.0
        };

        // Now update the round with all our calculations
        let round = self.current_round.as_mut()
            .ok_or(ConsensusError::NoActiveRound)?;

        round.votes.insert(validator.clone(), vote);
        round.stats.participation_rate = participation_rate;
        round.stats.approval_rate = approval_rate;

        // Check if consensus is reached
        if participation_rate >= self.config.min_participation_rate && 
           approval_rate >= self.config.min_approval_rate {
            round.status = RoundStatus::Finalizing;
        }

        Ok(ConsensusEvent::VoteReceived {
            round: round_number,
            validator,
            approve,
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

    pub fn check_timeout(&mut self) -> bool {
        if let Some(round) = &mut self.current_round {
            if Utc::now() > round.timeout {
                round.status = RoundStatus::Failed;
                return true;
            }
        }
        false
    }

    pub fn get_current_round(&self) -> Option<&ConsensusRound> {
        self.current_round.as_ref()
    }

    pub fn get_round_history(&self) -> &[ConsensusRoundStats] {
        &self.round_history
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
        let result = manager.start_round(1, "did:icn:test".to_string(), 1.0, 3);
        assert!(result.is_ok());
        assert!(manager.get_current_round().is_some());
    }

    #[test]
    fn test_propose_block() {
        let mut manager = setup_test_round_manager();
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        let result = manager.propose_block("did:icn:test", block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vote_flow() {
        let mut manager = setup_test_round_manager();
        
        // Setup round
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        manager.propose_block("did:icn:test", block).unwrap();
        
        // Submit enough votes for consensus
        let vote_result = manager.submit_vote(
            "validator1".to_string(),
            true,
            0.7,
            "signature1".to_string()
        );
        
        assert!(vote_result.is_ok());
        assert_eq!(
            manager.get_current_round().unwrap().status,
            RoundStatus::Finalizing
        );
    }

    #[test]
    fn test_duplicate_vote() {
        let mut manager = setup_test_round_manager();
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        manager.propose_block("did:icn:test", block).unwrap();
        
        // First vote should succeed
        assert!(manager.submit_vote(
            "validator1".to_string(),
            true,
            0.3,
            "signature1".to_string()
        ).is_ok());
        
        // Second vote from same validator should fail
        assert!(matches!(
            manager.submit_vote(
                "validator1".to_string(),
                true,
                0.3,
                "signature2".to_string()
            ),
            Err(ConsensusError::Custom(_))
        ));
    }

    #[test]
    fn test_timeout() {
        let mut manager = setup_test_round_manager();
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        
        // Modify timeout to be in the past
        if let Some(round) = &mut manager.current_round {
            round.timeout = Utc::now() - Duration::seconds(1);
        }
        
        assert!(manager.check_timeout());
        assert_eq!(
            manager.get_current_round().unwrap().status,
            RoundStatus::Failed
        );
    }

    #[test]
    fn test_finalize_round() {
        let mut manager = setup_test_round_manager();
        
        // Setup and get to finalization state
        manager.start_round(1, "did:icn:test".to_string(), 1.0, 3).unwrap();
        let block = Block::new(1, "prev_hash".to_string(), vec![], "did:icn:test".to_string());
        manager.propose_block("did:icn:test", block).unwrap();
        
        // Submit vote with enough power for consensus
        manager.submit_vote(
            "validator1".to_string(),
            true,
            0.7,
            "signature1".to_string()
        ).unwrap();
        
        // Finalize
        let result = manager.finalize_round();
        assert!(result.is_ok());
        assert!(manager.get_current_round().is_none());
        assert_eq!(manager.get_round_history().len(), 1);
    }
}
