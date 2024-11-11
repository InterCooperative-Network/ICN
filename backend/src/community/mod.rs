// src/community/mod.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::governance::Proposal;
use crate::claims::Claim;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub members: HashMap<String, CivicRole>, // DID -> Role mapping
    pub governance_model: GovernanceModel,
    pub active_proposals: Vec<Proposal>,
    pub policies: Vec<CivicPolicy>,
    pub cooperative_ids: Vec<String>, // Associated cooperatives
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicRole {
    pub role: String,
    pub permissions: Vec<String>,
    pub joined_at: DateTime<Utc>,
    pub verified_claims: Vec<Claim>,
    pub voting_history: Vec<VoteRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceModel {
    pub model_type: GovernanceType,
    pub voting_rules: VotingRules,
    pub quorum_requirement: f64,
    pub decision_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceType {
    DirectDemocracy,
    DelegativeDemocracy,
    ConsensusBase,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRules {
    pub voting_period_days: u32,
    pub min_participation: f64,
    pub allow_delegation: bool,
    pub require_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub proposal_id: String,
    pub vote: Vote,
    pub timestamp: DateTime<Utc>,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vote {
    For,
    Against,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicPolicy {
    pub id: String,
    pub policy_type: CivicPolicyType,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CivicPolicyType {
    Membership,
    Voting,
    DisputeResolution,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub condition: String,
    pub effect: String,
    pub parameters: HashMap<String, String>,
}

impl Community {
    pub fn new(
        id: String,
        name: String,
        description: String,
        governance_model: GovernanceModel,
    ) -> Self {
        Community {
            id,
            name,
            description,
            created_at: Utc::now(),
            members: HashMap::new(),
            governance_model,
            active_proposals: Vec::new(),
            policies: Vec::new(),
            cooperative_ids: Vec::new(),
        }
    }

    pub fn add_member(&mut self, did: String, role: CivicRole) -> Result<(), String> {
        if self.members.contains_key(&did) {
            return Err("Member already exists".to_string());
        }
        self.members.insert(did, role);
        Ok(())
    }

    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.active_proposals.push(proposal);
    }

    pub fn add_policy(&mut self, policy: CivicPolicy) {
        self.policies.push(policy);
    }

    pub fn associate_cooperative(&mut self, cooperative_id: String) {
        if !self.cooperative_ids.contains(&cooperative_id) {
            self.cooperative_ids.push(cooperative_id);
        }
    }

    pub fn record_vote(&mut self, did: &str, proposal_id: &str, vote: Vote) -> Result<(), String> {
        if let Some(member) = self.members.get_mut(did) {
            let vote_record = VoteRecord {
                proposal_id: proposal_id.to_string(),
                vote,
                timestamp: Utc::now(),
                weight: 1.0, // Basic weight, could be modified based on reputation/claims
            };
            member.voting_history.push(vote_record);
            Ok(())
        } else {
            Err("Member not found".to_string())
        }
    }
}

// Implement the trait for community energy tracking
impl crate::monitoring::energy::EnergyAware for Community {
    fn record_energy_metrics(&self, monitor: &crate::monitoring::energy::EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record proposal storage
        let proposals_size = (self.active_proposals.len() * std::mem::size_of::<Proposal>()) as u64;
        monitor.record_storage_operation(proposals_size);
        
        // Record member operations
        let members_size = (self.members.len() * std::mem::size_of::<CivicRole>()) as u64;
        monitor.record_memory_operation(members_size);
    }
}