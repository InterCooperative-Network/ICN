pub mod proof_of_cooperation;
pub mod validation;
pub mod round_management;
pub mod timeout_handling;
pub mod federation;

use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::time::sleep;
use tokio::task;
use icn_core::ReputationManager;
use icn_types::Block;
use std::sync::Arc;
use bit_set::BitSet;
use trie_rs::Trie;
use thiserror::Error;
use federation::{Federation, FederationError};
use zk_snarks::verify_proof; // Import zk-SNARK verification function

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
}

impl ProofOfCooperation {
    pub fn new(reputation_manager: Arc<dyn ReputationManager>) -> Self {
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
        let (total_reputation, approval_reputation) = self.parallel_vote_counting().await
            .map_err(|e| ConsensusError::ConsensusFailure(e.to_string()))?;

        // BFT requirement: Need more than 2/3 of total reputation for finalization
        let bft_threshold = (total_reputation as f64 * 2.0 / 3.0) as i64;
        
        if approval_reputation > bft_threshold {
            if let Some(block) = &self.proposed_block {
                // Update block metadata before finalization
                let mut final_block = block.clone();
                let consensus_duration = self.round_start_time.elapsed().as_millis() as u64;
                final_block.metadata.consensus_duration_ms = consensus_duration;
                
                // Clear round state
                self.start_round();
                
                Ok(Some(final_block))
            } else {
                Err(ConsensusError::ConsensusFailure("No proposed block".into()))
            }
        } else {
            Ok(None)
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
                let total_reputation: i64 = chunk.iter().map(|p| reputation_manager.get_reputation(p, "consensus")).sum();
                let approval_reputation: i64 = chunk.iter().enumerate().filter(|(i, _)| votes.contains(*i)).map(|(_, p)| reputation_manager.get_reputation(p, "consensus")).sum();
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
}

#[async_trait]
impl ConsensusEngine for ProofOfCooperation {
    async fn start(&self) {
        // Start the consensus process
    }

    async fn stop(&self) {
        // Stop the consensus process
    }

    async fn submit_vote(&mut self, vote: Vote) -> Result<VoteStatus, GovernanceError> {
        let proposal = self.active_proposals.get_mut(&vote.proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        
        if !self.is_eligible_voter(&vote.voter_did) {
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
