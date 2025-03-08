use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use log::{info, warn, debug};
use thiserror::Error;
use crate::pbft::PbftConsensus;
use crate::timeout_handling::TimeoutStrategy;
use icn_types::{Block, BlockHeader};

/// Round states in the consensus process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundState {
    Idle,
    Prepare,
    Commit,
    Finalize,
    ViewChange,
}

/// Error types for round management
#[derive(Error, Debug)]
pub enum RoundError {
    #[error("Round already in progress")]
    RoundInProgress,
    #[error("Invalid round state transition: {0}")]
    InvalidStateTransition(String),
    #[error("Round timeout: {0}")]
    RoundTimeout(String),
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Insufficient participation: {0}")]
    InsufficientParticipation(String),
}

/// Structure to manage consensus rounds
pub struct RoundManager {
    /// Current round number
    pub round_number: u64,
    /// Current round state
    pub state: RoundState,
    /// Timestamp when the current round started
    pub round_start_time: Instant,
    /// Maximum allowed round duration
    pub round_timeout: Duration,
    /// Timeout strategy for rounds
    pub timeout_strategy: TimeoutStrategy,
    /// Current proposed block for this round
    pub current_block: Option<Block>,
    /// History of consensus rounds
    pub round_history: Vec<RoundMetrics>,
    /// PBFT instance for consensus
    pub pbft: Arc<Mutex<PbftConsensus>>,
    /// The identity of this validator
    pub validator_id: String,
    /// Active validator set
    pub validator_set: Vec<String>,
    /// Tracks participation statistics
    pub validator_participation: HashMap<String, ParticipationStats>,
}

/// Statistics about validator participation
pub struct ParticipationStats {
    /// Number of rounds participated in
    pub rounds_participated: u64,
    /// Number of rounds missed
    pub rounds_missed: u64,
    /// Total blocks proposed
    pub blocks_proposed: u64,
    /// Total valid votes cast
    pub valid_votes: u64,
    /// Last seen timestamp
    pub last_seen: Instant,
}

/// Metrics collected for each consensus round
pub struct RoundMetrics {
    /// Round number
    pub round_number: u64,
    /// Round duration in milliseconds
    pub duration_ms: u64,
    /// Number of participating validators
    pub validator_count: usize,
    /// Success or failure status
    pub success: bool,
    /// Block height if successful
    pub block_height: Option<u64>,
    /// Participation rate (0.0 - 1.0)
    pub participation_rate: f64,
}

impl RoundManager {
    /// Create a new round manager
    pub fn new(
        validator_id: String, 
        validator_set: Vec<String>,
        timeout: Duration,
        pbft: Arc<Mutex<PbftConsensus>>,
    ) -> Self {
        let mut validator_participation = HashMap::new();
        
        for validator in &validator_set {
            validator_participation.insert(validator.clone(), ParticipationStats {
                rounds_participated: 0,
                rounds_missed: 0,
                blocks_proposed: 0,
                valid_votes: 0,
                last_seen: Instant::now(),
            });
        }
        
        Self {
            round_number: 0,
            state: RoundState::Idle,
            round_start_time: Instant::now(),
            round_timeout: timeout,
            timeout_strategy: TimeoutStrategy::ExponentialBackoff {
                initial: timeout,
                max_timeout: Duration::from_secs(120),
                factor: 1.5,
            },
            current_block: None,
            round_history: Vec::new(),
            pbft,
            validator_id,
            validator_set,
            validator_participation,
        }
    }
    
    /// Start a new consensus round
    pub async fn start_round(&mut self) -> Result<(), RoundError> {
        if self.state != RoundState::Idle {
            return Err(RoundError::RoundInProgress);
        }
        
        // Update round number and reset state
        self.round_number += 1;
        self.state = RoundState::Prepare;
        self.round_start_time = Instant::now();
        self.current_block = None;
        
        info!("Starting consensus round {}", self.round_number);
        
        // Reset PBFT consensus state
        let mut pbft = self.pbft.lock().await;
        pbft.reset_sequence();
        
        Ok(())
    }
    
    /// Propose a block for the current round
    pub fn propose_block(&mut self, block: Block) -> Result<(), RoundError> {
        if self.state != RoundState::Prepare {
            return Err(RoundError::InvalidStateTransition(
                format!("Cannot propose block in {:?} state", self.state)
            ));
        }
        
        // Store the proposed block
        self.current_block = Some(block);
        
        // Update proposer stats
        if let Some(stats) = self.validator_participation.get_mut(&self.validator_id) {
            stats.blocks_proposed += 1;
            stats.last_seen = Instant::now();
        }
        
        // Transition to commit state
        self.state = RoundState::Commit;
        info!("Block proposed for round {}", self.round_number);
        
        Ok(())
    }
    
    /// Register participation for a validator
    pub fn register_participation(&mut self, validator_id: &str) {
        if let Some(stats) = self.validator_participation.get_mut(validator_id) {
            stats.rounds_participated += 1;
            stats.last_seen = Instant::now();
        }
    }
    
    /// Register a missed round for a validator
    pub fn register_missed_round(&mut self, validator_id: &str) {
        if let Some(stats) = self.validator_participation.get_mut(validator_id) {
            stats.rounds_missed += 1;
        }
    }
    
    /// Register a valid vote from a validator
    pub fn register_vote(&mut self, validator_id: &str) {
        if let Some(stats) = self.validator_participation.get_mut(validator_id) {
            stats.valid_votes += 1;
            stats.last_seen = Instant::now();
        }
    }
    
    /// Finalize the current round
    pub async fn finalize_round(&mut self, success: bool) -> Result<Option<Block>, RoundError> {
        if self.state != RoundState::Commit && self.state != RoundState::ViewChange {
            return Err(RoundError::InvalidStateTransition(
                format!("Cannot finalize round in {:?} state", self.state)
            ));
        }
        
        // Calculate round duration
        let duration_ms = self.round_start_time.elapsed().as_millis() as u64;
        
        // Calculate participation rate
        let participating_validators = self.validator_participation.values()
            .filter(|stats| stats.last_seen.elapsed() < Duration::from_secs(300)) // Active in last 5 minutes
            .count();
            
        let participation_rate = participating_validators as f64 / self.validator_set.len() as f64;
        
        // Record round metrics
        self.round_history.push(RoundMetrics {
            round_number: self.round_number,
            duration_ms,
            validator_count: self.validator_set.len(),
            success,
            block_height: self.current_block.as_ref().map(|b| b.header.height),
            participation_rate,
        });
        
        // Update validator stats for those who didn't participate
        for validator_id in &self.validator_set {
            let stats = self.validator_participation.get_mut(validator_id).unwrap();
            if stats.last_seen.elapsed() > Duration::from_secs(300) {
                stats.rounds_missed += 1;
            }
        }
        
        // Transition back to idle state
        self.state = RoundState::Idle;
        
        // If round was successful, return the finalized block
        let result = if success {
            self.current_block.take()
        } else {
            None
        };
        
        Ok(result)
    }
    
    /// Check if the current round has timed out
    pub fn is_timed_out(&self) -> bool {
        self.round_start_time.elapsed() > self.round_timeout
    }
    
    /// Handle a timeout in the current round
    pub async fn handle_timeout(&mut self) -> Result<(), RoundError> {
        warn!("Round {} timed out after {}ms", 
            self.round_number, 
            self.round_start_time.elapsed().as_millis()
        );
        
        // Transition to view change state
        self.state = RoundState::ViewChange;
        
        // Trigger PBFT view change
        let mut pbft = self.pbft.lock().await;
        pbft.force_view_change();
        
        // Calculate new timeout for next round
        let mut timeout_strategy = std::mem::replace(
            &mut self.timeout_strategy, 
            TimeoutStrategy::Fixed(Duration::from_secs(60))
        );
        let new_timeout = timeout_strategy.next_timeout();
        self.timeout_strategy = timeout_strategy;
        self.round_timeout = new_timeout;
        
        // Finalize the round as failed
        self.finalize_round(false).await?;
        
        Ok(())
    }
    
    /// Run timeout checking in the background
    pub async fn run_timeout_checker(round_manager: Arc<RwLock<RoundManager>>) {
        loop {
            sleep(Duration::from_millis(100)).await;
            
            let is_timed_out = {
                let rm = round_manager.read().await;
                rm.state != RoundState::Idle && rm.is_timed_out()
            };
            
            if is_timed_out {
                let mut rm = round_manager.write().await;
                if let Err(e) = rm.handle_timeout().await {
                    warn!("Error handling timeout: {:?}", e);
                }
            }
        }
    }
    
    /// Get participation score for a validator (0.0-1.0)
    pub fn get_participation_score(&self, validator_id: &str) -> Option<f64> {
        if let Some(stats) = self.validator_participation.get(validator_id) {
            let total_rounds = stats.rounds_participated + stats.rounds_missed;
            if total_rounds > 0 {
                Some(stats.rounds_participated as f64 / total_rounds as f64)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
    
    /// Get the average round duration from history
    pub fn average_round_duration(&self) -> Option<Duration> {
        if self.round_history.is_empty() {
            return None;
        }
        
        let total_ms: u64 = self.round_history.iter().map(|m| m.duration_ms).sum();
        let avg_ms = total_ms / self.round_history.len() as u64;
        
        Some(Duration::from_millis(avg_ms))
    }
    
    /// Get the success rate of rounds from history
    pub fn round_success_rate(&self) -> Option<f64> {
        if self.round_history.is_empty() {
            return None;
        }
        
        let successful_rounds = self.round_history.iter().filter(|m| m.success).count();
        Some(successful_rounds as f64 / self.round_history.len() as f64)
    }
}
