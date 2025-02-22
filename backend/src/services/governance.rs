use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::reputation::ReputationManager;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Invalid recall vote")]
    InvalidRecallVote,
    #[error("Insufficient reputation")]
    InsufficientReputation,
    #[error("Proposal expired")]
    ProposalExpired,
    #[error("Member not found")]
    MemberNotFound,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceMember {
    pub did: String,
    pub joined_at: DateTime<Utc>,
    pub missed_votes: u32,
    pub last_activity: DateTime<Utc>,
    pub reputation_score: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecallVote {
    pub target_member: String,
    pub reason: String,
    pub votes: HashMap<String, bool>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct GovernanceService {
    members: RwLock<HashMap<String, GovernanceMember>>,
    recall_votes: RwLock<HashMap<String, RecallVote>>,
    reputation_manager: ReputationManager,
    inactivity_threshold_days: u32,
    missed_votes_threshold: u32,
}

impl GovernanceService {
    pub fn new(reputation_manager: ReputationManager) -> Self {
        Self {
            members: RwLock::new(HashMap::new()),
            recall_votes: RwLock::new(HashMap::new()),
            reputation_manager,
            inactivity_threshold_days: 30,
            missed_votes_threshold: 3,
        }
    }

    pub async fn track_vote_participation(&self, member_did: &str, voted: bool) {
        let mut members = self.members.write().await;
        if let Some(member) = members.get_mut(member_did) {
            if !voted {
                member.missed_votes += 1;
                
                // Check if recall vote should be triggered
                if member.missed_votes >= self.missed_votes_threshold {
                    self.initiate_recall_vote(member_did).await;
                }
            } else {
                member.last_activity = Utc::now();
                // Reset missed votes on participation
                member.missed_votes = 0;
            }
        }
    }

    async fn initiate_recall_vote(&self, member_did: &str) {
        let recall = RecallVote {
            target_member: member_did.to_string(),
            reason: format!("Member missed {} consecutive votes", self.missed_votes_threshold),
            votes: HashMap::new(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        let mut recall_votes = self.recall_votes.write().await;
        recall_votes.insert(member_did.to_string(), recall);
    }

    pub async fn process_recall_vote(&self, voter: &str, target: &str, approve: bool) -> Result<(), GovernanceError> {
        let mut recall_votes = self.recall_votes.write().await;
        
        if let Some(recall) = recall_votes.get_mut(target) {
            recall.votes.insert(voter.to_string(), approve);
            
            // Check if recall threshold met
            let total_votes = recall.votes.len();
            let approve_votes = recall.votes.values().filter(|&&v| v).count();
            
            if total_votes >= 10 && (approve_votes * 2) > total_votes {
                self.remove_member(target).await?;
                recall_votes.remove(target);
            }
        }
        Ok(())
    }

    pub async fn check_proposal_expiration(&self, proposal_id: &str) -> Result<bool, GovernanceError> {
        // Check if proposal is nearing expiration and notify if needed
        // Return true if expiring soon
        Ok(false) // Placeholder
    }

    pub async fn decay_inactive_reputation(&self) {
        let members = self.members.read().await;
        let now = Utc::now();
        
        for member in members.values() {
            let days_inactive = (now - member.last_activity).num_days();
            if days_inactive > self.inactivity_threshold_days as i64 {
                // Apply reputation decay
                let decay_factor = -0.1 * (days_inactive as f64 / self.inactivity_threshold_days as f64);
                self.reputation_manager.dynamic_adjustment(&member.did, decay_factor as i64).await
                    .unwrap_or_else(|e| eprintln!("Failed to decay reputation: {}", e));
            }
        }
    }

    async fn remove_member(&self, member_did: &str) -> Result<(), GovernanceError> {
        let mut members = self.members.write().await;
        members.remove(member_did).ok_or(GovernanceError::MemberNotFound)?;
        Ok(())
    }
}
