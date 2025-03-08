pub mod proof_of_cooperation;
pub mod validation;
pub mod round_management;
pub mod timeout_handling;
pub mod federation;
pub mod sharding; // Add sharding module
pub mod pbft; // Add PBFT module

use async_trait::async_trait;
use std::collections::{HashMap, VecDeque, HashSet}; // Added HashSet import
use std::time::Duration;
use tokio::time::sleep;
use tokio::task;
use icn_common::{ReputationManager, ConsensusEngine, Vote, VoteStatus, GovernanceError};
use icn_types::{Block, Transaction}; // Import Transaction
use std::sync::Arc;
use bit_set::BitSet;
use trie_rs::Trie;
use thiserror::Error;
use federation::{Federation, FederationError};
use serde::{Serialize, Deserialize};
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use icn_crypto::KeyPair; // Import KeyPair for signature verification
use log::error;

// Interface for identity services
#[async_trait]
pub trait IdentityService: Send + Sync {
    async fn get_public_key(&self, did: &str) -> Option<Vec<u8>>;
    async fn verify_identity(&self, did: &str, proof: &str) -> bool;
}

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Failed to reach consensus: {0}")]
    ConsensusFailure(String),
    #[error("Block validation failed: {0}")]
    ValidationFailure(String),
    #[error("Timeout occurred: {0}")]
    TimeoutError(String),
    #[error("BFT error: {0}")]
    BftError(String),
}

pub struct ProofOfCooperation {
    current_round: u64,
    participants: VecDeque<String>,
    proposed_block: Option<Block>,
    votes: BitSet,
    vote_trie: Trie,
    timeout: Duration,
    timeout_handling: timeout_handling::TimeoutHandling,
    reputation_manager: Arc<dyn ReputationManager>,
    federation_operations: HashMap<String, FederationOperation>,
    federations: HashMap<String, Federation>,
    round_start_time: std::time::Instant,
    shard_manager: sharding::ShardManager, // Add shard manager field
    pbft_consensus: Option<pbft::PbftConsensus>, // Add PBFT consensus field
    identity_did: String, // Add the identity DID of the current node
    identity_service: Arc<dyn IdentityService>, // Add identity service
    active_proposals: HashMap<String, Proposal>, // Add active proposals
    rules: ConsensusRules,
}

// Define proposal structure
#[derive(Clone, Debug)]
pub struct Proposal {
    pub proposal_id: String,
    pub proposal_type: ProposalType,
    pub votes: HashSet<Vote>,
    pub status: ProposalStatus,
    pub created_at: std::time::SystemTime,
}

// Define proposal types
#[derive(Clone, Debug)]
pub enum ProposalType {
    AddValidator(ValidatorInfo),
    RemoveValidator(String), // DID
    UpdateRules(ConsensusRules),
}

// Define validator info
#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    pub did: String,
    pub reputation: i64,
    pub voting_power: u32,
}

// Define consensus rules
#[derive(Clone, Debug)]
pub struct ConsensusRules {
    pub min_reputation: i64,
    pub quorum_percentage: u8,
    pub block_time: Duration,
}

// Define proposal status
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl ProofOfCooperation {
    pub fn new(
        reputation_manager: Arc<dyn ReputationManager>,
        identity_service: Arc<dyn IdentityService>,
        identity_did: String
    ) -> Self {
        let shard_config = sharding::ShardConfig {
            shard_count: 4,  // Start with 4 shards
            shard_capacity: 1000, // Each shard can hold 1000 transactions
            rebalance_threshold: 0.3, // Rebalance when load differs by 30%
        };
        
        ProofOfCooperation {
            current_round: 0,
            participants: VecDeque::new(),
            proposed_block: None,
            votes: BitSet::new(),
            vote_trie: Trie::new(),
            timeout: Duration::from_secs(60),
            timeout_handling: timeout_handling::TimeoutHandling::new(Duration::from_secs(60)),
            reputation_manager,
            federation_operations: HashMap::new(),
            federations: HashMap::new(),
            round_start_time: std::time::Instant::now(),
            shard_manager: sharding::ShardManager::new(shard_config),
            pbft_consensus: None,
            identity_did,
            identity_service,
            active_proposals: HashMap::new(),
            rules: ConsensusRules {
                min_reputation: 50,
                quorum_percentage: 67, // 2/3 majority
                block_time: Duration::from_secs(30),
            },
        }
    }

    pub fn start_round(&mut self) {
        self.current_round += 1;
        self.proposed_block = None;
        self.votes.clear();
        self.vote_trie = Trie::new();
        self.round_start_time = std::time::Instant::now();
    }

    pub fn propose_block(&mut self, block: Block) {
        self.proposed_block = Some(block);
    }

    pub fn vote(&mut self, participant: String, vote: bool) {
        if self.is_eligible(&participant) {
            let index = self.participants.iter().position(|p| p == &participant).unwrap_or_else(|| {
                self.participants.push_back(participant.clone());
                self.participants.len() - 1
            });
            if vote {
                self.votes.insert(index);
            }
            self.vote_trie.insert(&participant);
        }
    }

    pub async fn finalize_block(&mut self) -> Result<Option<Block>, ConsensusError> {
        // Get validator list for this round
        let validators = self.select_validators(50).await
            .map_err(|e| ConsensusError::ConsensusFailure(e.to_string()))?;
            
        // Ensure we have enough validators for BFT
        let f = validators.len() / 4; // Maximum allowed Byzantine nodes
        let min_validators = 3 * f + 1; // Minimum required for BFT
        
        if validators.len() < min_validators {
            return Err(ConsensusError::BftError(
                format!("Insufficient validators: {} (need {})", validators.len(), min_validators)
            ));
        }
        
        if self.pbft_consensus.is_none() {
            self.pbft_consensus = Some(pbft::PbftConsensus::new(validators.clone()));
        }
        
        let pbft = self.pbft_consensus.as_mut().unwrap();
        
        if let Some(block) = &self.proposed_block {
            // Primary validator broadcasts pre-prepare message
            if pbft.is_primary(&self.identity_did) {
                let pre_prepare = pbft::ConsensusMessage {
                    message_type: pbft::MessageType::PrePrepare,
                    view_number: pbft.view_number,
                    sequence_number: pbft.sequence_number,
                    block_hash: block.hash.clone(),
                    sender: self.identity_did.clone(),
                    signature: "signature".to_string(), // This should be a proper signature
                };
                
                // Distribute to all validators
                self.broadcast_consensus_message(pre_prepare).await?;
            }
            
            // Wait for consensus to be reached
            let timeout = Duration::from_secs(30);
            let start = std::time::Instant::now();
            
            while start.elapsed() < timeout {
                if pbft.is_committed(&block.hash) {
                    // Consensus reached, update block metadata
                    let consensus_duration = self.round_start_time.elapsed().as_millis() as u64;
                    let mut final_block = block.clone();
                    final_block.metadata.consensus_duration_ms = consensus_duration;
                    
                    // Clear round state
                    self.start_round();
                    
                    return Ok(Some(final_block));
                }
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            // Timeout occurred
            Err(ConsensusError::TimeoutError("PBFT consensus timed out".into()))
        } else {
            Err(ConsensusError::ConsensusFailure("No proposed block".into()))
        }
    }

    pub async fn handle_timeout(&self) {
        if let Err(e) = self.timeout_handling.handle_timeout().await {
            eprintln!("Error handling timeout: {}", e);
        }
    }

    fn is_eligible(&self, participant: &str) -> bool {
        self.reputation_manager.is_eligible(participant, 10, "consensus")
    }

    pub async fn parallel_vote_counting(&self) -> Result<(i64, i64), Box<dyn std::error::Error>> {
        let chunks: Vec<_> = self.participants.chunks(self.participants.len() / 4).collect();
        let mut handles = vec![];

        for chunk in chunks {
            let chunk = chunk.to_vec();
            let reputation_manager = self.reputation_manager.clone();
            let votes = self.votes.clone();
            let handle = task::spawn(async move {
                let mut total_reputation = 0i64;
                let mut approval_reputation = 0i64;
                
                for (i, p) in chunk.iter().enumerate() {
                    let rep = reputation_manager.get_reputation(p, "consensus");
                    total_reputation += rep;
                    if votes.contains(i) {
                        approval_reputation += rep;
                    }
                }
                
                Ok((total_reputation, approval_reputation))
            });
            handles.push(handle);
        }

        let mut total_reputation = 0;
        let mut approval_reputation = 0;

        for handle in handles {
            let (chunk_total, chunk_approval) = handle.await??;
            total_reputation += chunk_total;
            approval_reputation += chunk_approval;
        }

        Ok((total_reputation, approval_reputation))
    }

    pub async fn select_validators(&mut self, min_reputation: i64) -> Result<Vec<String>, ConsensusError> {
        let mut validators = Vec::new();
        let participants: Vec<_> = self.participants.iter().cloned().collect();

        for participant in participants {
            if self.reputation_manager.is_eligible(&participant, min_reputation, "consensus") {
                validators.push(participant);
            }
        }

        // BFT requirement: Need at least 3f + 1 validators where f is max faulty nodes
        let min_validators = (self.max_faulty_nodes() * 3) + 1;
        if validators.len() < min_validators {
            return Err(ConsensusError::BftError(
                format!("Insufficient validators: {} (need {})", validators.len(), min_validators)
            ));
        }

        Ok(validators)
    }

    fn max_faulty_nodes(&self) -> usize {
        self.participants.len() / 3
    }

    pub async fn handle_consensus_round(&mut self) -> Result<Option<Block>, ConsensusError> {
        // Start timeout handler
        let timeout_handler = self.timeout_handling.start_timeout();
        
        tokio::select! {
            result = self.finalize_block() => {
                result
            }
            _ = timeout_handler => {
                self.handle_timeout().await;
                Err(ConsensusError::TimeoutError("Consensus round timed out".into()))
            }
        }
    }

    pub fn handle_federation_operation(&mut self, operation: FederationOperation) {
        match operation {
            FederationOperation::InitiateFederation { federation_type, partner_id, terms } => {
                self.create_federation(partner_id, federation_type, terms).unwrap();
            }
            FederationOperation::JoinFederation { federation_id, commitment } => {
                // Handle joining federation logic
            }
            FederationOperation::LeaveFederation { federation_id, reason } => {
                // Handle leaving federation logic
            }
            FederationOperation::ProposeAction { federation_id, action_type, description, resources } => {
                // Handle proposing action logic
            }
            FederationOperation::VoteOnProposal { federation_id, proposal_id, approve, notes } => {
                // Handle voting on proposal logic
            }
            FederationOperation::ShareResources { federation_id, resource_type, amount, recipient_id } => {
                // Handle sharing resources logic
            }
            FederationOperation::UpdateFederationTerms { federation_id, new_terms } => {
                // Handle updating federation terms logic
            }
        }
    }

    pub fn create_federation(
        &mut self,
        creator_id: String,
        federation_type: FederationType,
        terms: FederationTerms,
    ) -> Result<String, ConsensusError> {
        // Verify creator's reputation
        let creator_reputation = self.reputation_manager.get_reputation(&creator_id, "consensus");
        if creator_reputation < terms.minimum_reputation {
            return Err(ConsensusError::ConsensusFailure(
                "Insufficient reputation to create federation".into(),
            ));
        }

        // Generate unique federation ID
        let federation_id = format!("fed_{}", uuid::Uuid::new_v4());

        // Create new federation
        let federation = Federation::new(
            federation_id.clone(),
            federation_type,
            terms,
            creator_id,
        );

        // Store federation
        self.federations.insert(federation_id.clone(), federation);

        Ok(federation_id)
    }

    pub async fn adjust_validator_set(&mut self) -> Result<(), ConsensusError> {
        // Get current validator counts for BFT calculation
        let current_size = self.participants.len();
        let min_validators = (current_size / 3) * 3 + 1; // 3f + 1 where f is max faulty

        // Remove validators that fell below minimum reputation
        let mut to_remove = Vec::new();
        for participant in self.participants.iter() {
            if !self.reputation_manager.is_eligible(participant, 50, "consensus") {
                to_remove.push(participant.clone());
            }
        }

        // Remove disqualified validators if we maintain BFT requirements
        if (current_size - to_remove.len()) >= min_validators {
            for participant in to_remove {
                if let Some(pos) = self.participants.iter().position(|x| x == &participant) {
                    self.participants.remove(pos);
                }
            }
        }

        // Add new validators that meet higher reputation threshold
        let new_validators = self.select_validators(80).await?;
        
        // Add new validators while maintaining max size limit
        let max_validators = 100; // Example maximum validator set size
        for validator in new_validators {
            if self.participants.len() >= max_validators {
                break;
            }
            if !self.participants.contains(&validator) {
                self.participants.push_back(validator);
            }
        }

        Ok(())
    }

    pub async fn start_validator_rotation(&mut self) {
        tokio::spawn(async move {
            let rotation_interval = Duration::from_secs(3600); // 1 hour
            loop {
                sleep(rotation_interval).await;
                if let Err(e) = self.adjust_validator_set().await {
                    error!("Failed to adjust validator set: {}", e);
                }
            }
        });
    }

    pub async fn verify_zk_snark_proof(&self, proof: &str) -> Result<bool, String> {
        if !verify_proof(proof) {
            return Err("Invalid zk-SNARK proof".to_string());
        }
        Ok(true)
    }

    pub fn apply_anti_monopoly_reputation_decay(&self, reputation: i64, dominance: f64, total: f64, alpha: f64) -> i64 {
        (reputation as f64 * (1.0 - dominance / total).powf(alpha)) as i64
    }

    pub fn quadratic_vote_weight(&self, reputation_points: i64) -> f64 {
        (reputation_points as f64).sqrt()
    }

    pub fn randomized_delegation(&self, participants: Vec<String>, num_delegates: usize) -> Vec<String> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut delegates = participants.clone();
        delegates.shuffle(&mut rng);
        delegates.truncate(num_delegates);
        delegates
    }

    pub fn dynamic_contribution_valuation(&self, value: i64, repeated: i64, lambda: f64) -> i64 {
        (value as f64 * (-lambda * repeated as f64).exp()) as i64
    }

    pub async fn add_signature(&self, did: &str, signature: &str, message: &str) -> Result<(), ConsensusError> {
        // Retrieve public key from IdentityService
        if let Some(public_key) = self.identity_service.get_public_key(did).await {
            let key_pair = KeyPair {
                public_key,
                private_key: vec![], // Not needed for verification
                algorithm: icn_crypto::Algorithm::Secp256k1, // Assuming Secp256k1 for this example
            };
            if key_pair.verify(message.as_bytes(), signature.as_bytes()) {
                Ok(())
            } else {
                Err(ConsensusError::ValidationFailure("Invalid signature".to_string()))
            }
        } else {
            Err(ConsensusError::ValidationFailure("Public key not found".to_string()))
        }
    }

    // Add a method to handle transaction sharding
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<u32, ConsensusError> {
        // Assign transaction to a shard
        let shard_id = self.shard_manager.assign_transaction(transaction);
        
        // Check if shard is ready for finalization
        let shard = self.shard_manager.shards.get(&shard_id).unwrap();
        if shard.len() >= self.shard_manager.config.shard_capacity as usize {
            if let Some(block) = self.shard_manager.finalize_shard(shard_id) {
                self.propose_block(block);
            }
        }
        
        // Periodically rebalance shards
        if self.current_round % 10 == 0 {  // Every 10 rounds
            self.shard_manager.rebalance();
        }
        
        Ok(shard_id)
    }

    async fn broadcast_consensus_message(&self, message: pbft::ConsensusMessage) -> Result<(), ConsensusError> {
        // Implementation to broadcast message to all validators
        // This would use your networking layer to distribute the message
        // ...
        Ok(())
    }

    // Check if a participant is eligible to vote
    pub fn is_eligible_voter(&self, voter_did: &str) -> bool {
        // Check if they have enough reputation
        let reputation = self.reputation_manager.get_reputation(voter_did, "governance");
        if reputation < self.rules.min_reputation {
            return false;
        }
        
        // Check if they're a validator
        self.participants.iter().any(|p| p == voter_did)
    }
    
    // Add a validator to the consensus system
    pub fn add_validator(&mut self, info: ValidatorInfo) -> Result<(), GovernanceError> {
        // Verify validator meets minimum reputation requirements
        if info.reputation < self.rules.min_reputation {
            return Err(GovernanceError::NotEligibleToVote);
        }
        
        // Add to participant list if not already present
        if !self.participants.contains(&info.did) {
            self.participants.push_back(info.did);
        }
        
        Ok(())
    }
    
    // Remove a validator from the consensus system
    pub fn remove_validator(&mut self, did: &str) -> Result<(), GovernanceError> {
        // Find and remove from participant list
        if let Some(index) = self.participants.iter().position(|p| p == did) {
            self.participants.remove(index);
            Ok(())
        } else {
            Err(GovernanceError::ProposalNotFound)
        }
    }
    
    // Check proposal status and update if necessary
    pub fn check_proposal_status(&mut self, proposal_id: &str) -> Result<VoteStatus, GovernanceError> {
        let proposal = self.active_proposals.get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
            
        // Count votes
        let mut approve_votes = 0;
        let mut reject_votes = 0;
        
        for vote in &proposal.votes {
            if vote.approve {
                approve_votes += 1;
            } else {
                reject_votes += 1;
            }
        }
        
        // Check if we have enough votes for a decision
        let total_participants = self.participants.len();
        let quorum_threshold = (total_participants * self.rules.quorum_percentage as usize) / 100;
        
        if approve_votes + reject_votes < quorum_threshold {
            // Not enough votes yet
            proposal.status = ProposalStatus::Pending;
            return Ok(VoteStatus::Pending);
        }
        
        // We have enough votes to make a decision
        if approve_votes > reject_votes {
            proposal.status = ProposalStatus::Approved;
            Ok(VoteStatus::Accepted)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(VoteStatus::Rejected)
        }
    }
    
    // Submit a new proposal
    pub fn submit_proposal(&mut self, proposal_type: ProposalType, submitter: String) -> Result<String, GovernanceError> {
        // Check if submitter is eligible
        if !self.is_eligible_voter(&submitter) {
            return Err(GovernanceError::NotEligibleToVote);
        }
        
        // Generate proposal ID
        let proposal_id = format!("prop_{}", uuid::Uuid::new_v4());
        
        // Create new proposal
        let proposal = Proposal {
            proposal_id: proposal_id.clone(),
            proposal_type,
            votes: HashSet::new(),
            status: ProposalStatus::Pending,
            created_at: std::time::SystemTime::now(),
        };
        
        // Store proposal
        self.active_proposals.insert(proposal_id.clone(), proposal);
        
        Ok(proposal_id)
    }
}

#[async_trait]
impl ConsensusEngine for ProofOfCooperation {
    async fn start(&self) {
        // Start the consensus process
        // In a real implementation, this would initiate validator rounds
        // and transaction processing
        log::info!("Starting Proof of Cooperation consensus engine");
    }

    async fn stop(&self) {
        // Stop the consensus process
        log::info!("Stopping Proof of Cooperation consensus engine");
    }

    async fn submit_vote(&mut self, vote: Vote) -> Result<VoteStatus, GovernanceError> {
        let proposal = self.active_proposals.get_mut(&vote.proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        
        if !self.is_eligible_voter(&vote.voter) {
            return Err(GovernanceError::NotEligibleToVote);
        }

        proposal.votes.insert(vote);
        self.check_proposal_status(&proposal.proposal_id)
    }

    async fn process_approved_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError> {
        let proposal = self.active_proposals.remove(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        match proposal.proposal_type {
            ProposalType::AddValidator(info) => self.add_validator(info),
            ProposalType::RemoveValidator(did) => self.remove_validator(&did),
            ProposalType::UpdateRules(rules) => {
                self.rules = rules;
                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum FederationOperation {
    InitiateFederation {
        federation_type: FederationType,
        partner_id: String,
        terms: FederationTerms,
    },
    JoinFederation {
        federation_id: String,
        commitment: Vec<String>,
    },
    LeaveFederation {
        federation_id: String,
        reason: String,
    },
    ProposeAction {
        federation_id: String,
        action_type: String,
        description: String,
        resources: std::collections::HashMap<String, u64>,
    },
    VoteOnProposal {
        federation_id: String,
        proposal_id: String,
        approve: bool,
        notes: Option<String>,
    },
    ShareResources {
        federation_id: String,
        resource_type: String,
        amount: u64,
        recipient_id: String,
    },
    UpdateFederationTerms {
        federation_id: String,
        new_terms: FederationTerms,
    },
}

#[derive(Serialize, Deserialize)]
struct FederationTerms {
    minimum_reputation: i64,
    resource_sharing_policies: String,
    governance_rules: String,
    duration: String,
}

#[derive(Serialize, Deserialize)]
enum FederationType {
    Cooperative,
    Community,
    Hybrid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VoteStatus {
    Accepted,
    Rejected,
    Pending,
}

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Not eligible to vote")]
    NotEligibleToVote,
}
