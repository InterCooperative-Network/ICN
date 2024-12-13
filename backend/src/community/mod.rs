use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::claims::Claim;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub members: HashMap<String, CivicRole>, // DID -> Role mapping
    pub governance_model: GovernanceModel,
    #[serde(default)]
    pub active_proposals: Vec<String>, // Store proposal IDs instead of whole proposals
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
    pub action: String,
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

    pub fn add_proposal(&mut self, proposal_id: String) {
        if !self.active_proposals.contains(&proposal_id) {
            self.active_proposals.push(proposal_id);
        }
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

impl crate::monitoring::energy::EnergyAware for Community {
    fn record_energy_metrics(&self, monitor: &crate::monitoring::energy::EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record proposal storage
        let proposals_size = (self.active_proposals.len() * std::mem::size_of::<String>()) as u64;
        monitor.record_storage_operation(proposals_size);
        
        // Record member operations
        let members_size = (self.members.len() * std::mem::size_of::<CivicRole>()) as u64;
        monitor.record_memory_operation(members_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_governance_model() -> GovernanceModel {
        GovernanceModel {
            model_type: GovernanceType::DirectDemocracy,
            voting_rules: VotingRules {
                voting_period_days: 7,
                min_participation: 0.5,
                allow_delegation: false,
                require_claims: vec![],
            },
            quorum_requirement: 0.5,
            decision_threshold: 0.66,
        }
    }

    #[test]
    fn test_community_creation() {
        let community = Community::new(
            "test_id".to_string(),
            "Test Community".to_string(),
            "Test Description".to_string(),
            create_test_governance_model(),
        );

        assert_eq!(community.name, "Test Community");
        assert_eq!(community.members.len(), 0);
        assert_eq!(community.active_proposals.len(), 0);
    }

    #[test]
    fn test_member_management() {
        let mut community = Community::new(
            "test_id".to_string(),
            "Test Community".to_string(),
            "Test Description".to_string(),
            create_test_governance_model(),
        );

        let role = CivicRole {
            role: "member".to_string(),
            permissions: vec!["vote".to_string()],
            joined_at: Utc::now(),
            verified_claims: vec![],
            voting_history: vec![],
        };

        assert!(community.add_member("test_did".to_string(), role).is_ok());
        assert!(community.add_member("test_did".to_string(), role).is_err());
    }
}
