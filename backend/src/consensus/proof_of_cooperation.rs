use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

// Validator represents a node participating in consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub did: String,
    pub reputation: i64,
    pub last_block_proposed: u64,
    pub consecutive_missed_validations: u32,
    pub is_active: bool,
}

// ConsensusRound represents a single round of the PoC consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub coordinator: String,  // DID of the coordinator
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, bool>,  // DID -> vote
    pub status: RoundStatus,
    pub start_time: u64,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Proposing,
    Voting,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct ProofOfCooperation {
    // Core consensus state
    validators: HashMap<String, Validator>,
    current_round: Option<ConsensusRound>,
    finalized_blocks: Vec<Block>,
    
    // Configuration
    min_validators: usize,
    vote_threshold: f64,
    round_timeout: u64,
    min_reputation: i64,
    
    // Reputation tracking
    reputation_updates: Vec<(String, i64)>,  // (DID, reputation change)
}

impl ProofOfCooperation {
    pub fn new(min_validators: usize, vote_threshold: f64, round_timeout: u64, min_reputation: i64) -> Self {
        ProofOfCooperation {
            validators: HashMap::new(),
            current_round: None,
            finalized_blocks: Vec::new(),
            min_validators,
            vote_threshold,
            round_timeout,
            min_reputation,
            reputation_updates: Vec::new(),
        }
    }

    // Register a new validator
    pub fn register_validator(&mut self, did: String, initial_reputation: i64) -> Result<(), String> {
        if initial_reputation < self.min_reputation {
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

    // Select the coordinator for the next round using reputation-weighted selection
    fn select_coordinator(&self) -> Option<String> {
        let active_validators: Vec<_> = self.validators
            .values()
            .filter(|v| v.is_active && v.reputation >= self.min_reputation)
            .collect();

        if active_validators.len() < self.min_validators {
            return None;
        }

        // Calculate total reputation of active validators
        let total_reputation: i64 = active_validators.iter().map(|v| v.reputation).sum();
        
        // Generate random number between 0 and total_reputation
        let mut rng = rand::thread_rng();
        let selection_point = rng.gen_range(0..total_reputation);
        
        // Select coordinator based on reputation weight
        let mut cumulative_reputation = 0;
        for validator in active_validators {
            cumulative_reputation += validator.reputation;
            if cumulative_reputation > selection_point {
                return Some(validator.did.clone());
            }
        }
        
        None
    }

    // Start a new consensus round
    pub fn start_round(&mut self) -> Result<(), String> {
        if self.current_round.is_some() {
            return Err("Consensus round already in progress".to_string());
        }

        let coordinator = self.select_coordinator()
            .ok_or("Unable to select coordinator")?;

        let round = ConsensusRound {
            round_number: self.finalized_blocks.len() as u64 + 1,
            coordinator,
            proposed_block: None,
            votes: HashMap::new(),
            status: RoundStatus::Proposing,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            timeout: self.round_timeout,
        };

        self.current_round = Some(round);
        Ok(())
    }

    // Propose a block (called by coordinator)
    pub fn propose_block(&mut self, proposer_did: &str, block: Block) -> Result<(), String> {
        let round = self.current_round.as_mut()
            .ok_or("No active consensus round")?;

        if round.status != RoundStatus::Proposing {
            return Err("Round not in proposing state".to_string());
        }

        if round.coordinator != proposer_did {
            return Err("Only the coordinator can propose blocks".to_string());
        }

        round.proposed_block = Some(block);
        round.status = RoundStatus::Voting;
        Ok(())
    }

    // Submit a vote for the proposed block
    pub fn submit_vote(&mut self, validator_did: &str, vote: bool) -> Result<(), String> {
        let round = self.current_round.as_mut()
            .ok_or("No active consensus round")?;

        if round.status != RoundStatus::Voting {
            return Err("Round not in voting state".to_string());
        }

        if !self.validators.contains_key(validator_did) {
            return Err("Not a registered validator".to_string());
        }

        round.votes.insert(validator_did.to_string(), vote);

        // Check if we have enough votes to finalize
        if self.can_finalize_round() {
            round.status = RoundStatus::Finalizing;
        }

        Ok(())
    }

    // Check if we can finalize the current round
    fn can_finalize_round(&self) -> bool {
        if let Some(round) = &self.current_round {
            let total_reputation: i64 = self.validators
                .values()
                .filter(|v| v.is_active)
                .map(|v| v.reputation)
                .sum();

            let voting_reputation: i64 = round.votes.keys()
                .filter_map(|did| self.validators.get(did))
                .map(|v| v.reputation)
                .sum();

            let participation_ratio = voting_reputation as f64 / total_reputation as f64;
            let approval_reputation: i64 = round.votes.iter()
                .filter(|(_, &vote)| vote)
                .filter_map(|(did, _)| self.validators.get(did))
                .map(|v| v.reputation)
                .sum();

            let approval_ratio = approval_reputation as f64 / voting_reputation as f64;

            participation_ratio >= self.vote_threshold && approval_ratio >= self.vote_threshold
        } else {
            false
        }
    }

    // Finalize the current round
    pub fn finalize_round(&mut self) -> Result<Block, String> {
        let round = self.current_round.as_ref()
            .ok_or("No active consensus round")?;

        if round.status != RoundStatus::Finalizing {
            return Err("Round not ready for finalization".to_string());
        }

        let block = round.proposed_block.as_ref()
            .ok_or("No proposed block")?
            .clone();

        // Update validator reputations based on participation
        self.update_reputations(&round);

        // Add block to finalized blocks
        self.finalized_blocks.push(block.clone());

        // Reset current round
        self.current_round = None;

        Ok(block)
    }

    // Update validator reputations based on round participation
    fn update_reputations(&mut self, round: &ConsensusRound) {
        let active_validators: HashSet<_> = self.validators
            .values()
            .filter(|v| v.is_active)
            .map(|v| v.did.clone())
            .collect();

        // Reward validators who voted
        for voter_did in round.votes.keys() {
            if let Some(validator) = self.validators.get_mut(voter_did) {
                validator.reputation += 1;
                validator.consecutive_missed_validations = 0;
                self.reputation_updates.push((voter_did.clone(), 1));
            }
        }

        // Penalize validators who didn't vote
        for did in active_validators {
            if !round.votes.contains_key(&did) {
                if let Some(validator) = self.validators.get_mut(&did) {
                    validator.consecutive_missed_validations += 1;
                    
                    // Larger penalty for consecutive misses
                    let penalty = -(validator.consecutive_missed_validations as i64);
                    validator.reputation += penalty;
                    self.reputation_updates.push((did.clone(), penalty));

                    // Deactivate validator if reputation drops too low
                    if validator.reputation < self.min_reputation {
                        validator.is_active = false;
                    }
                }
            }
        }

        // Extra reward for the coordinator if round was successful
        if let Some(validator) = self.validators.get_mut(&round.coordinator) {
            validator.reputation += 2;
            self.reputation_updates.push((round.coordinator.clone(), 2));
        }
    }

    // Get the current round status
    pub fn get_round_status(&self) -> Option<RoundStatus> {
        self.current_round.as_ref().map(|r| r.status.clone())
    }

    // Get the list of reputation updates from the last round
    pub fn get_reputation_updates(&self) -> &Vec<(String, i64)> {
        &self.reputation_updates
    }

    // Check if a round has timed out
    pub fn check_timeout(&mut self) -> bool {
        if let Some(round) = &self.current_round {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if current_time - round.start_time > round.timeout {
                // Handle timeout by failing the round
                self.handle_timeout();
                return true;
            }
        }
        false
    }

    // Handle a round timeout
    fn handle_timeout(&mut self) {
        if let Some(round) = &mut self.current_round {
            round.status = RoundStatus::Failed;
            
            // Penalize the coordinator for timeout
            if let Some(validator) = self.validators.get_mut(&round.coordinator) {
                validator.reputation -= 3;
                self.reputation_updates.push((round.coordinator.clone(), -3));
            }
        }
        
        // Reset the round
        self.current_round = None;
    }
}

// Helper functions for tests
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_consensus() -> ProofOfCooperation {
        ProofOfCooperation::new(3, 0.66, 60, 50)
    }

    #[test]
    fn test_validator_registration() {
        let mut consensus = setup_consensus();
        
        // Test successful registration
        assert!(consensus.register_validator("did:icn:1".to_string(), 100).is_ok());
        
        // Test registration with insufficient reputation
        assert!(consensus.register_validator("did:icn:2".to_string(), 40).is_err());
    }

    #[test]
    fn test_round_lifecycle() {
        let mut consensus = setup_consensus();
        
        // Register validators
        consensus.register_validator("did:icn:1".to_string(), 100).unwrap();
        consensus.register_validator("did:icn:2".to_string(), 100).unwrap();
        consensus.register_validator("did:icn:3".to_string(), 100).unwrap();
        
        // Start round
        assert!(consensus.start_round().is_ok());
        
        // Verify round status
        assert_eq!(consensus.get_round_status(), Some(RoundStatus::Proposing));
    }

    #[test]
    fn test_reputation_updates() {
        let mut consensus = setup_consensus();
        
        // Register validators
        consensus.register_validator("did:icn:1".to_string(), 100).unwrap();
        consensus.register_validator("did:icn:2".to_string(), 100).unwrap();
        
        // Start round and submit votes
        consensus.start_round().unwrap();
        if let Some(round) = &consensus.current_round {
            let coordinator = round.coordinator.clone();
            let block = Block::new(1, "prev_hash".to_string(), vec![]); // Create dummy block
            consensus.propose_block(&coordinator, block).unwrap();
            consensus.submit_vote("did:icn:1", true).unwrap();
            consensus.submit_vote("did:icn:2", true).unwrap();
        }
        
        // Check reputation updates
        let updates = consensus.get_reputation_updates();
        assert!(!updates.is_empty());
    }
}