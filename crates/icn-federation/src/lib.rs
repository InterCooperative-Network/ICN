use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use icn_types::{Block, Transaction};
use icn_governance::{DissolutionProtocol, DissolutionReason, DissolutionStatus};
use icn_zkp::RollupBatch;
use thiserror::Error;
use icn_networking::p2p::{P2PManager, FederationEvent}; // Import P2PManager and FederationEvent
use zk_snarks::verify_proof; // Import zk-SNARK verification function
use std::time::{SystemTime, Duration};

// Add SDP support
use icn_p2p::sdp::{SDPManager, SDPPacket, SDPHeader, PublicKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: String,
    pub name: String,
    pub federation_type: FederationType,
    pub members: HashMap<String, MemberStatus>, // DID -> status
    pub member_roles: HashMap<String, MemberRole>, // DID -> role
    pub terms: FederationTerms,
    pub resources: HashMap<String, ResourcePool>,
    pub proposals: Vec<FederationProposal>, // Add proposals field
    pub created_at: u64,
    pub status: FederationStatus,
    pub disputes: HashMap<String, FederationDispute>, // Add disputes field
    pub cross_federation_disputes: HashMap<String, Vec<FederationDispute>>,
    pub audit_log: Vec<AuditEntry>,
    pub p2p_manager: Arc<tokio::sync::Mutex<P2PManager>>, // Changed to tokio::sync::Mutex
    // Add secure communication fields
    pub sdp_peers: HashMap<String, Vec<String>>, // federation_id -> [peer_addresses]
    pub federation_public_keys: HashMap<String, String>, // federation_id -> public_key (base58 encoded)
}

// Add an SDPConfig struct to handle SDP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDPConfig {
    pub bind_address: String,
    pub enable_multipath: bool,
    pub enable_onion_routing: bool,
    pub message_priority: HashMap<String, u8>, // message_type -> priority
}

impl Default for SDPConfig {
    fn default() -> Self {
        let mut message_priority = HashMap::new();
        message_priority.insert("governance_vote".to_string(), 8);
        message_priority.insert("dispute_resolution".to_string(), 9);
        message_priority.insert("resource_allocation".to_string(), 6);
        message_priority.insert("member_update".to_string(), 5);
        
        Self {
            bind_address: "0.0.0.0:0".to_string(), // Random port by default
            enable_multipath: true,
            enable_onion_routing: false, // Optional advanced feature
            message_priority,
        }
    }
}

// Add a struct for federation manager with SDP support
pub struct FederationManager {
    federations: Arc<RwLock<HashMap<String, Federation>>>,
    resource_manager: Arc<dyn ResourceManager>,
    // Add SDP manager
    sdp_manager: Option<Arc<RwLock<SDPManager>>>,
    sdp_config: SDPConfig,
}

impl FederationManager {
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
            sdp_manager: None,
            sdp_config: SDPConfig::default(),
        }
    }

    // Initialize SDP for secure federation communications
    pub async fn init_sdp(&mut self, config: SDPConfig) -> Result<(), FederationError> {
        match SDPManager::new(&config.bind_address) {
            Ok(manager) => {
                self.sdp_manager = Some(Arc::new(RwLock::new(manager)));
                self.sdp_config = config;
                
                // Start receiver for handling incoming messages
                self.start_sdp_receiver().await?;
                
                Ok(())
            },
            Err(e) => Err(FederationError::CommunicationError(format!("Failed to initialize SDP: {}", e))),
        }
    }

    // Start SDP receiver to handle incoming messages
    async fn start_sdp_receiver(&self) -> Result<(), FederationError> {
        if let Some(sdp) = &self.sdp_manager {
            let sdp_clone = sdp.clone();
            let federations_clone = self.federations.clone();
            
            let handler = move |data: Vec<u8>, src| {
                let federations = federations_clone.clone();
                
                tokio::spawn(async move {
                    // Handle incoming SDP messages
                    if let Ok(message) = serde_json::from_slice::<FederationMessage>(&data) {
                        // Process message based on type
                        match message.message_type {
                            FederationMessageType::ProposalSubmission => {
                                if let Ok(proposal) = serde_json::from_value(message.payload) {
                                    let mut federations_lock = federations.write().await;
                                    if let Some(federation) = federations_lock.get_mut(&message.target_federation) {
                                        // Add signature verification here
                                        let _ = federation.submit_proposal(proposal);
                                    }
                                }
                            },
                            FederationMessageType::Vote => {
                                if let Ok(vote) = serde_json::from_value(message.payload) {
                                    let mut federations_lock = federations.write().await;
                                    if let Some(federation) = federations_lock.get_mut(&message.target_federation) {
                                        let _ = federation.vote(vote);
                                    }
                                }
                            },
                            FederationMessageType::DisputeInitiation => {
                                if let Ok(dispute) = serde_json::from_value(message.payload) {
                                    let mut federations_lock = federations.write().await;
                                    if let Some(federation) = federations_lock.get_mut(&message.target_federation) {
                                        let _ = federation.submit_dissolution_dispute(dispute);
                                    }
                                }
                            },
                            FederationMessageType::ResourceAllocation => {
                                // Handle resource allocation messages
                            },
                            FederationMessageType::MembershipUpdate => {
                                // Handle membership updates
                            },
                        }
                    }
                });
            };
            
            sdp_clone.lock().await.start_receiver(handler).await
                .map_err(|e| FederationError::CommunicationError(format!("Failed to start SDP receiver: {}", e)))
        } else {
            Err(FederationError::CommunicationError("SDP manager not initialized".to_string()))
        }
    }

    // Send a federation message via SDP
    pub async fn send_federation_message(
        &self,
        source_federation: &str,
        target_federation: &str,
        message_type: FederationMessageType,
        payload: serde_json::Value,
        signature: &str,
    ) -> Result<(), FederationError> {
        let federations = self.federations.read().await;
        
        let source_fed = federations.get(source_federation)
            .ok_or(FederationError::FederationNotFound(source_federation.to_string()))?;
            
        let target_fed = federations.get(target_federation)
            .ok_or(FederationError::FederationNotFound(target_federation.to_string()))?;
            
        // Check if we have SDP peer info for the target
        if !source_fed.sdp_peers.contains_key(target_federation) {
            return Err(FederationError::CommunicationError(
                format!("No SDP routing information for federation {}", target_federation)
            ));
        }
        
        // Create federation message
        let message = FederationMessage {
            source_federation: source_federation.to_string(),
            target_federation: target_federation.to_string(),
            message_type,
            payload,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: signature.to_string(),
        };
        
        // Serialize message
        let serialized = serde_json::to_vec(&message)
            .map_err(|e| FederationError::CommunicationError(format!("Serialization error: {}", e)))?;
            
        // Get message priority
        let priority = self.sdp_config.message_priority
            .get(message.message_type.to_string().as_str())
            .cloned()
            .unwrap_or(5); // Default priority
            
        if let Some(sdp_manager) = &self.sdp_manager {
            let manager = sdp_manager.lock().await;
            manager.send_message(target_federation, &serialized, priority).await
                .map_err(|e| FederationError::CommunicationError(format!("Failed to send SDP message: {}", e)))
        } else {
            Err(FederationError::CommunicationError("SDP manager not initialized".to_string()))
        }
    }

    pub async fn create_federation(
        &self,
        name: String,
        federation_type: FederationType,
        initial_terms: FederationTerms,
        founding_member: String,
    ) -> Result<String, FederationError> {
        let federation_id = format!("fed_{}", uuid::Uuid::new_v4());
        let federation = Federation {
            id: federation_id.clone(),
            name,
            federation_type,
            members: vec![founding_member].into_iter().map(|m| (m, MemberStatus::Active)).collect(),
            member_roles: HashMap::new(),
            terms: initial_terms,
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
            status: FederationStatus::Active,
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
            p2p_manager: Arc::new(tokio::sync::Mutex::new(P2PManager::new())), // Initialize p2p_manager
            // Initialize SDP communication fields
            sdp_peers: HashMap::new(),
            federation_public_keys: HashMap::new(),
        };

        let mut federations = self.federations.write().await;
        federations.insert(federation_id.clone(), federation);

        Ok(federation_id)
    }

    pub async fn join_federation(
        &self,
        federation_id: &str,
        member_did: &str,
        commitment: Vec<String>,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            if federation.members.contains_key(member_did) {
                return Err(FederationError::AlreadyMember(member_did.to_string()));
            }

            // Verify commitments against federation terms
            if !self.verify_commitments(&federation.terms, &commitment).await {
                return Err(FederationError::InvalidCommitment(member_did.to_string()));
            }

            federation.members.insert(member_did.to_string(), MemberStatus::Active);
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }

        // Add SDP peer information if available
        if let Some(sdp) = &self.sdp_manager {
            let manager = sdp.lock().await;
            let public_key = manager.keypair.1;
            
            // Update the federation with this node's SDP information
            let mut federations = self.federations.write().await;
            if let Some(federation) = federations.get_mut(federation_id) {
                // Add public key in base58 encoding for interoperability
                federation.federation_public_keys.insert(
                    member_did.to_string(),
                    bs58::encode(public_key.as_bytes()).into_string()
                );
            }
        }
        
        Ok(())
    }

    async fn verify_commitments(&self, terms: &FederationTerms, commitment: &[String]) -> bool {
        // Add commitment verification logic here
        true // Placeholder
    }

    pub async fn submit_proposal(
        &self,
        federation_id: &str,
        proposal: FederationProposal,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;

        if let Some(federation) = federations.get_mut(federation_id) {
            federation.submit_proposal(proposal)?;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    pub async fn vote(
        &self,
        federation_id: &str,
        vote: Vote,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;

        if let Some(federation) = federations.get_mut(federation_id) {
            federation.vote(vote)?;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }
}

// Add FederationMessage struct for secure communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    pub source_federation: String,
    pub target_federation: String,
    pub message_type: FederationMessageType,
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub signature: String,
}

// Define message types for federation communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessageType {
    ProposalSubmission,
    Vote,
    DisputeInitiation,
    ResourceAllocation,
    MembershipUpdate,
}

impl ToString for FederationMessageType {
    fn to_string(&self) -> String {
        match self {
            FederationMessageType::ProposalSubmission => "governance_proposal".to_string(),
            FederationMessageType::Vote => "governance_vote".to_string(),
            FederationMessageType::DisputeInitiation => "dispute_resolution".to_string(),
            FederationMessageType::ResourceAllocation => "resource_allocation".to_string(),
            FederationMessageType::MembershipUpdate => "member_update".to_string(),
        }
    }
}

impl Federation {
    pub fn add_member(&mut self, did: String, role: MemberRole) -> Result<(), FederationError> {
        if self.members.contains_key(&did) {
            return Err(FederationError::AlreadyMember(did));
        }

        // Verify member meets minimum reputation requirements
        if !self.verify_member_eligibility(&did) {
            return Err(FederationError::InsufficientReputation(
                "Member does not meet minimum reputation requirements".to_string(),
            ));
        }

        self.members.insert(did, MemberStatus::Active);
        Ok(())
    }

    pub fn remove_member(&mut self, did: &str) -> Result<(), FederationError> {
        if !self.members.contains_key(did) {
            return Err(FederationError::MemberNotFound(did.to_string()));
        }

        self.members.remove(did);
        Ok(())
    }

    pub fn get_member_status(&self, did: &str) -> Option<&MemberStatus> {
        self.members.get(did)
    }

    pub fn update_member_status(&mut self, did: &str, status: MemberStatus) -> Result<(), FederationError> {
        if let Some(member_status) = self.members.get_mut(did) {
            *member_status = status;
            Ok(())
        } else {
            Err(FederationError::MemberNotFound(did.to_string()))
        }
    }

    pub fn get_active_members(&self) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, status)| matches!(status, MemberStatus::Active))
            .map(|(did, _)| did.clone())
            .collect()
    }

    pub fn verify_member_eligibility(&self, did: &str) -> bool {
        // This would integrate with the reputation system in practice
        true // Simplified for example
    }

    pub fn submit_proposal(&mut self, proposal: FederationProposal) -> Result<(), FederationError> {
        // Validate proposal
        self.validate_proposal(&proposal)?;

        // Set proposal voting period
        let mut proposal = proposal;
        proposal.voting_ends_at = chrono::Utc::now().timestamp() as u64 + 
            (self.terms.governance_rules.max_voting_period_hours * 3600);

        self.proposals.push(proposal);
        Ok(())
    }

    pub fn vote(&mut self, vote: Vote) -> Result<(), FederationError> {
        // Validate vote
        self.validate_vote(&vote)?;

        // Get proposal
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == vote.proposal_id)
            .ok_or(FederationError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Record vote
        proposal.votes.insert(vote.voter, vote.approve);

        // Check if voting period ended and finalize if needed
        let now = chrono::Utc::now().timestamp() as u64;
        if now > proposal.voting_ends_at {
            proposal.status = self.finalize_proposal(&proposal.id)?;
        }

        Ok(())
    }

    pub fn validate_proposal(&self, proposal: &FederationProposal) -> Result<(), FederationError> {
        // Check if proposal type is allowed
        if !self.terms.governance_rules.allowed_proposal_types.contains(&proposal.proposal_type.to_string()) {
            return Err(FederationError::InvalidProposalType(proposal.proposal_type.to_string()));
        }

        // Validate proposer has sufficient reputation
        if !self.verify_member_eligibility(&proposal.proposer) {
            return Err(FederationError::InsufficientReputation(
                "Proposer does not meet minimum reputation requirements".to_string()
            ));
        }

        Ok(())
    }

    pub fn validate_vote(&self, vote: &Vote) -> Result<(), FederationError> {
        // Check if voter is a member
        if !self.members.contains_key(&vote.voter) {
            return Err(FederationError::UnauthorizedAction { action: "vote".to_string(), did: vote.voter.clone() });
        }

        // Check if proposal exists
        let proposal = self.proposals.iter()
            .find(|p| p.id == vote.proposal_id)
            .ok_or(FederationError::ProposalNotFound(vote.proposal_id.clone()))?;

        // Check if voting period is still open
        let now = chrono::Utc::now().timestamp() as u64;
        if now > proposal.voting_ends_at {
            return Err(FederationError::VotingPeriodEnded(vote.proposal_id.clone()));
        }

        // Check for veto rights
        if let Some(member_role) = self.member_roles.get(&vote.voter) {
            if let Some(veto_actions) = self.terms.governance_rules.veto_rights.get(&member_role.to_string()) {
                if veto_actions.contains(&proposal.proposal_type.to_string()) && !vote.approve {
                    // Record veto
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<ProposalStatus, FederationError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(FederationError::ProposalNotFound(proposal_id.to_string()))?;

        // Create vote batch for on-chain processing
        let batch = RollupBatch {
            proposal_id: proposal_id.to_string(),
            votes: proposal.votes.iter()
                .map(|(voter, approve)| Vote { voter: voter.clone(), approve: *approve, signature: String::new() }) // Added signature field
                .collect(),
            rollup_root: [0u8; 32], // Computed by ZK prover
            proof: Vec::new(), // Generated by ZK prover
        };

        // Submit batch to chain
        self.contract.submit_vote_batch(batch)?;

        // Execute proposal on-chain
        let approved = self.contract.execute_proposal(proposal_id)?;
        
        proposal.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        Ok(proposal.status.clone())
    }

    pub fn calculate_asset_distribution(&self) -> HashMap<String, AssetAllocation> {
        let mut distributions = HashMap::new();
        // Implement fair asset distribution calculation
        distributions
    }

    pub fn settle_outstanding_debts(&self) -> Vec<DebtSettlement> {
        let mut settlements = Vec::new();
        // Implement debt settlement calculation
        settlements
    }

    pub fn reassign_members(&self) -> Vec<MemberReassignment> {
        let mut reassignments = Vec::new();
        // Implement member reassignment logic
        reassignments
    }

    pub fn calculate_vote_weight(&self, cooperative_id: &str, proposal: &FederationProposal) -> f64 {
        let voting_model = match proposal.proposal_type {
            ProposalType::GovernanceChange(_) | ProposalType::PolicyUpdate(_) => 
                &self.terms.governance_rules.governance_voting_model,
            ProposalType::ResourceAllocation(_) =>
                &self.terms.governance_rules.resource_voting_model,
            _ => &self.terms.governance_rules.default_voting_model,
        };

        voting_model.calculate_voting_power(self, cooperative_id)
    }

    pub fn get_cooperative_weight(&self, cooperative_id: &str) -> f64 {
        let total_members: u32 = self.members.values().map(|m| m.member_count).sum();
        let coop_members = self.members.get(cooperative_id)
            .map(|m| m.member_count)
            .unwrap_or(0);
        
        coop_members as f64 / total_members as f64
    }

    pub fn initiate_dissolution(&mut self, initiator: String, reason: String) -> Result<DissolutionProtocol, FederationError> {
        let protocol = DissolutionProtocol {
            federation_id: self.id.clone(),
            initiated_by: initiator,
            reason: DissolutionReason::Voluntary,
            status: DissolutionStatus::Initiated,
            asset_distribution: HashMap::new(),
            debt_settlements: Vec::new(),
            member_reassignments: Vec::new(),
            dispute_period_ends: SystemTime::now() + Duration::from_secs(7 * 24 * 60 * 60), // 7 days
        };

        self.status = FederationStatus::DisputePeriod;
        Ok(protocol)
    }

    pub fn submit_dissolution_dispute(&mut self, dispute: FederationDispute) -> Result<(), FederationError> {
        if self.status != FederationStatus::DisputePeriod {
            return Err(FederationError::InvalidStatusTransition { from: "DisputePeriod".to_string(), to: self.status.to_string() });
        }

        if !self.members.contains_key(&dispute.initiator) {
            return Err(FederationError::UnauthorizedAction { action: "submit_dissolution_dispute".to_string(), did: dispute.initiator.clone() });
        }

        self.disputes.insert(dispute.id.clone(), dispute);
        self.status = FederationStatus::DisputeResolution;
        Ok(())
    }

    pub fn vote_on_dispute(&mut self, dispute_id: &str, voter: String, support: bool) -> Result<(), FederationError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or(FederationError::DisputeNotFound(dispute_id.to_string()))?;

        if !self.members.contains_key(&voter) {
            return Err(FederationError::UnauthorizedAction { action: "vote_on_dispute".to_string(), did: voter.clone() });
        }

        dispute.supporting_votes.insert(voter, support);

        // Check if we have enough votes to resolve the dispute
        let total_votes = dispute.supporting_votes.len();
        let supporting_votes = dispute.supporting_votes.values().filter(|&&v| v).count();
        let required_votes = (self.members.len() * 2) / 3; // 2/3 majority

        if total_votes >= required_votes {
            if supporting_votes > total_votes / 2 {
                dispute.status = DisputeStatus::Resolved;
                self.status = FederationStatus::Active;
            } else {
                dispute.status = DisputeStatus::Rejected;
                self.status = FederationStatus::Dissolved;
            }
        }

        Ok(())
    }

    pub fn resolve_dispute(&mut self, dispute_id: &str, resolution: DisputeResolution) -> Result<(), FederationError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or(FederationError::DisputeNotFound(dispute_id.to_string()))?;

        dispute.resolution = Some(resolution);
        dispute.status = DisputeStatus::Resolved;

        // If all disputes are resolved, proceed with dissolution
        if self.disputes.values().all(|d| d.status == DisputeStatus::Resolved || d.status == DisputeStatus::Rejected) {
            let any_upheld = self.disputes.values().any(|d| d.status == DisputeStatus::Resolved);
            self.status = if any_upheld {
                FederationStatus::Active
            } else {
                FederationStatus::Dissolved;
            };
        }

        Ok(())
    }

    pub async fn initiate_cross_federation_dispute(
        &mut self,
        target_federation: &str,
        dispute: FederationDispute
    ) -> Result<(), FederationError> {
        // Verify both federations exist and have sufficient reputation
        self.verify_cross_federation_eligibility(target_federation).await?;
        
        let disputes = self.cross_federation_disputes
            .entry(target_federation.to_string())
            .or_insert_with(Vec::new);
        
        disputes.push(dispute);
        
        // Log dispute for audit
        self.audit_log.push(AuditEntry {
            action: "cross_federation_dispute".into(),
            target_federation: Some(target_federation.to_string()),
            timestamp: chrono::Utc::now(),
        });

        Ok(())
    }

    pub async fn submit_cross_federation_proposal(
        &mut self,
        target_federation: &str,
        proposal: CrossFederationProposal
    ) -> Result<(), FederationError> {
        // Verify both federations meet minimum reputation requirements
        self.verify_cross_federation_eligibility(target_federation).await?;
        
        // Create batch for cross-federation proposals
        let mut batch = ProposalBatch::new();
        batch.add_proposal(proposal.clone());
        
        // Submit to cross-federation coordinator
        self.coordinator.submit_batch(batch).await?;
        
        // Record proposal for local tracking
        self.cross_federation_proposals.insert(proposal.id.clone(), proposal);
        
        Ok(())
    }

    pub fn batch_process_proposals(&mut self, proposals: Vec<FederationProposal>) -> Result<(), FederationError> {
        let mut batch = ProposalBatch::new();
        
        for proposal in proposals {
            self.validate_proposal(&proposal)?;
            batch.add_proposal(proposal);
        }
        
        // Process batch through ZK rollup
        let rollup = self.create_proposal_rollup(batch);
        self.contract.submit_rollup(rollup)?;
        
        Ok(())
    }

    pub async fn publish_event(&self, event: FederationEvent) -> Result<(), FederationError> {
        let mut p2p = self.p2p_manager.lock().await;
        p2p.publish(event).await.map_err(|e| FederationError::EventPublishError(e.to_string()))
    }

    pub async fn subscribe_to_events(&self) -> Result<(), FederationError> {
        let mut p2p = self.p2p_manager.lock().await;
        p2p.subscribe().await.map_err(|e| FederationError::EventSubscribeError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub resource_type: String,
    pub total_amount: u64,
    pub available_amount: u64,
    pub contributors: HashMap<String, u64>, // DID -> amount contributed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationProposal {
    pub id: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub description: String,
    pub votes: HashMap<String, bool>, // DID -> vote
    pub status: ProposalStatus,
    pub created_at: u64,
    pub voting_ends_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    AddMember(String),
    RemoveMember(String),
    UpdateTerms(FederationTerms),
    AllocateResources(ResourceAllocation),
    UpdatePolicy(String),
}

impl std::fmt::Display for ProposalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalType::AddMember(_) => write!(f, "AddMember"),
            ProposalType::RemoveMember(_) => write!(f, "RemoveMember"),
            ProposalType::UpdateTerms(_) => write!(f, "UpdateTerms"),
            ProposalType::AllocateResources(_) => write!(f, "AllocateResources"),
            ProposalType::UpdatePolicy(_) => write!(f, "UpdatePolicy"),
        }
    }
}

pub struct FederationManager {
    federations: Arc<RwLock<HashMap<String, Federation>>>,
    resource_manager: Arc<dyn ResourceManager>,
}

impl FederationManager {
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
        }
    }

    pub async fn create_federation(
        &self,
        name: String,
        federation_type: FederationType,
        initial_terms: FederationTerms,
        founding_member: String,
    ) -> Result<String, FederationError> {
        let federation_id = format!("fed_{}", uuid::Uuid::new_v4());
        let federation = Federation {
            id: federation_id.clone(),
            name,
            federation_type,
            members: vec![founding_member].into_iter().map(|m| (m, MemberStatus::Active)).collect(),
            member_roles: HashMap::new(),
            terms: initial_terms,
            resources: HashMap::new(),
            proposals: Vec::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
            status: FederationStatus::Active,
            disputes: HashMap::new(),
            cross_federation_disputes: HashMap::new(),
            audit_log: Vec::new(),
            p2p_manager: Arc::new(tokio::sync::Mutex::new(P2PManager::new())), // Initialize p2p_manager
        };

        let mut federations = self.federations.write().await;
        federations.insert(federation_id.clone(), federation);

        Ok(federation_id)
    }

    pub async fn join_federation(
        &self,
        federation_id: &str,
        member_did: &str,
        commitment: Vec<String>,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;
        
        if let Some(federation) = federations.get_mut(federation_id) {
            if federation.members.contains_key(member_did) {
                return Err(FederationError::AlreadyMember(member_did.to_string()));
            }

            // Verify commitments against federation terms
            if !self.verify_commitments(&federation.terms, &commitment).await {
                return Err(FederationError::InvalidCommitment(member_did.to_string()));
            }

            federation.members.insert(member_did.to_string(), MemberStatus::Active);
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    async fn verify_commitments(&self, terms: &FederationTerms, commitment: &[String]) -> bool {
        // Add commitment verification logic here
        true // Placeholder
    }

    pub async fn submit_proposal(
        &self,
        federation_id: &str,
        proposal: FederationProposal,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;

        if let Some(federation) = federations.get_mut(federation_id) {
            federation.submit_proposal(proposal)?;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }

    pub async fn vote(
        &self,
        federation_id: &str,
        vote: Vote,
    ) -> Result<(), FederationError> {
        let mut federations = self.federations.write().await;

        if let Some(federation) = federations.get_mut(federation_id) {
            federation.vote(vote)?;
            Ok(())
        } else {
            Err(FederationError::FederationNotFound(federation_id.to_string()))
        }
    }
}

#[async_trait]
pub trait ResourceManager: Send + Sync {
    async fn allocate_resources(&self, allocation: ResourceAllocation) -> Result<(), String>;
    async fn release_resources(&self, resource_type: &str, amount: u64) -> Result<(), String>;
}

pub trait FederationDissolution {
    fn initiate_dissolution(&mut self, initiator: &str, reason: DissolutionReason) -> Result<DissolutionProtocol, Error>;
    fn process_dissolution(&mut self, protocol: &DissolutionProtocol) -> Result<DissolutionStatus, Error>;
    fn cancel_dissolution(&mut self, protocol_id: &str) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationType {
    Cooperative,
    Community,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTerms {
    pub minimum_reputation: i64,
    pub resource_sharing_policies: String,
    pub governance_rules: GovernanceRules,
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRules {
    pub min_votes_required: u32,
    pub approval_threshold_percent: u32,
    pub min_voting_period_hours: u32,
    pub max_voting_period_hours: u32,
    pub allowed_proposal_types: Vec<String>,
    pub veto_rights: HashMap<String, Vec<String>>, // role -> action types that can be vetoed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationStatus {
    Active,
    Suspended,
    Dissolved,
    DisputePeriod,
    DisputeResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberRole {
    Admin,
    Member,
    Observer,
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberRole::Admin => write!(f, "Admin"),
            MemberRole::Member => write!(f, "Member"),
            MemberRole::Observer => write!(f, "Observer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub approve: bool,
    pub signature: String, // Added required field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDispute {
    pub id: String,
    pub federation_id: String,
    pub initiator: String,
    pub reason: String,
    pub evidence: Option<String>,
    pub supporting_votes: HashMap<String, bool>,
    pub created_at: u64,
    pub status: DisputeStatus,
    pub resolution: Option<DisputeResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub decision: String,
    pub rationale: String,
    pub resolved_at: u64,
    pub resolver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputeStatus {
    Pending,
    Resolved,
    Rejected,
}

impl std::fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisputeStatus::Pending => write!(f, "Pending"),
            DisputeStatus::Resolved => write!(f, "Resolved"),
            DisputeStatus::Rejected => write!(f, "Rejected"),
        }
    }
}

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("Federation not found: {0}")]
    FederationNotFound(String),
    
    #[error("Already a member: {0}")]
    AlreadyMember(String),
    
    #[error("Invalid commitment: {0}")]
    InvalidCommitment(String),
    
    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },
    
    #[error("Unauthorized action: {action} by {did}")]
    UnauthorizedAction { action: String, did: String },
    
    #[error("Member not found: {0}")]
    MemberNotFound(String),
    
    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Invalid proposal type: {0}")] 
    InvalidProposalType(String),
    
    #[error("Voting period ended for proposal {0}")]
    VotingPeriodEnded(String),
    
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    
    #[error("Insufficient reputation: {0}")]
    InsufficientReputation(String),
    
    #[error("Dispute not found: {0}")]
    DisputeNotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Consensus error: {0}")]
    ConsensusError(#[from] ConsensusError),
    
    #[error("Event publish error: {0}")]
    EventPublishError(String),
    
    #[error("Event subscribe error: {0}")]
    EventSubscribeError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub action: String,
    pub target_federation: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFederationProposal {
    pub id: String,
    pub source_federation: String,
    pub target_federation: String,
    pub proposal_type: CrossFederationProposalType,
    pub terms: CrossFederationTerms,
    pub votes: HashMap<String, bool>,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossFederationProposalType {
    Merge,
    Alliance,
    ResourceSharing,
    DisputeResolution,
}

#[derive(Debug)]
pub struct BatchProcessor {
    pub coordinator: Arc<tokio::sync::Mutex<ProposalCoordinator>>,
}

#[derive(Debug)]
pub struct ProposalCoordinator {
    // Add fields as needed
}

impl ProposalCoordinator {
    pub async fn submit_batch(&self, batch: ProposalBatch) -> Result<(), FederationError> {
        // Implementation here
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProposalBatch {
    proposals: Vec<FederationProposal>,
}

impl ProposalBatch {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
        }
    }

    pub fn add_proposal(&mut self, proposal: FederationProposal) {
        self.proposals.push(proposal);
    }
}

// Add missing types
#[derive(Debug)]
pub struct StorageError;

#[derive(Debug)]
pub struct ConsensusError;

#[derive(Debug, Clone)]
pub struct AssetAllocation {
    pub asset_type: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct DebtSettlement {
    pub debtor: String,
    pub creditor: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct MemberReassignment {
    pub member_id: String,
    pub new_federation: String,
}

#[derive(Debug, Clone)]
pub struct CrossFederationTerms {
    pub resource_sharing_terms: String,
    pub governance_terms: String,
}

// Federation system for resource sharing and governance across cooperatives
mod federation;
mod resource_sharing;
mod resource_manager;

pub use federation::{
    Federation, 
    FederationError, 
    FederationType, 
    FederationTerms, 
    FederationManager,
    FederationProposal,
    FederationRole,
    FederationMember,
    Vote
};

pub use resource_sharing::{
    ResourceSharingAgreement,
    ResourceAllocation,
    ResourceUsageMetrics,
    SharingAgreementStatus
};

pub use resource_manager::{
    FederationResourceManager,
    ResourceProvider,
    ResourceError
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Federation resource pool that can be shared with other federations
#[derive(Clone, Debug)]
pub struct FederationResourcePool {
    /// ID of the federation that owns this resource pool
    pub federation_id: String,
    
    /// Available resources in this pool
    pub resources: HashMap<String, Resource>,
    
    /// Access control for this resource pool
    pub access_control: FederationAccessControl,
}

/// Access control settings for federation resource pools
#[derive(Clone, Debug)]
pub struct FederationAccessControl {
    /// List of federation IDs allowed to access this pool
    pub allowed_federations: Vec<String>,
    
    /// Minimum reputation required to access resources
    pub min_reputation: i64,
    
    /// Maximum allocation per federation
    pub max_allocation_per_federation: u64,
}

/// A generic resource that can be shared between federations
#[derive(Clone, Debug)]
pub struct Resource {
    /// Unique identifier for this resource
    pub id: String,
    
    /// Type of resource (e.g., "compute", "storage", "bandwidth")
    pub resource_type: String,
    
    /// Total amount of this resource
    pub total_amount: u64,
    
    /// Currently available (unallocated) amount
    pub available_amount: u64,
    
    /// ID of the federation that owns this resource
    pub owner_federation_id: String,
    
    /// Metadata about this resource
    pub metadata: HashMap<String, String>,
}

/// Creates a new resource sharing agreement between federations
/// 
/// # Arguments
/// * `source_federation_id` - ID of the federation providing resources
/// * `target_federation_id` - ID of the federation receiving resources
/// * `resource_type` - Type of resource being shared
/// * `amount` - Amount of resource to share
/// * `duration_seconds` - Optional duration for the agreement
/// * `terms` - Terms of the sharing agreement
/// * `min_reputation_score` - Minimum reputation score required for the target
/// 
/// # Returns
/// Agreement ID if successful, error otherwise
pub async fn create_resource_sharing_agreement(
    federation_resource_manager: &FederationResourceManager,
    source_federation_id: String,
    target_federation_id: String,
    resource_type: String,
    amount: u64,
    duration_seconds: Option<u64>,
    terms: String,
    min_reputation_score: i64,
) -> Result<String, ResourceError> {
    federation_resource_manager.propose_agreement(
        source_federation_id,
        target_federation_id,
        resource_type,
        amount,
        duration_seconds,
        terms,
        min_reputation_score,
    ).await
}
