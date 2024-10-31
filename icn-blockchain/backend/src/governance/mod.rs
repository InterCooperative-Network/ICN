// src/governance/mod.rs

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    voter: String,
    weight: i64,
    timestamp: DateTime<Utc>,
    delegated_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub creator: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub votes: HashMap<String, Vote>,
    pub required_quorum: f64,
    pub required_majority: f64,
    pub execution_threshold: Option<i64>,
}

#[derive(Debug)]
pub struct GovernanceSystem {
    proposals: HashMap<u64, Proposal>,
    delegations: HashMap<String, String>, // voter -> delegate
    reputation_weights: HashMap<String, i64>,
}

impl GovernanceSystem {
    pub fn new() -> Self {
        GovernanceSystem {
            proposals: HashMap::new(),
            delegations: HashMap::new(),
            reputation_weights: HashMap::new(),
        }
    }

    pub fn create_proposal(
        &mut self,
        creator: String,
        title: String,
        description: String,
        proposal_type: ProposalType,
        duration_days: i64,
        required_quorum: f64,
        required_majority: f64,
        execution_threshold: Option<i64>,
    ) -> Result<u64, String> {
        let id = (self.proposals.len() + 1) as u64;
        let now = Utc::now();
        
        let proposal = Proposal {
            id,
            creator,
            title,
            description,
            proposal_type,
            status: ProposalStatus::Active,
            created_at: now,
            ends_at: now + chrono::Duration::days(duration_days),
            votes: HashMap::new(),
            required_quorum,
            required_majority,
            execution_threshold,
        };

        self.proposals.insert(id, proposal);
        Ok(id)
    }

    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: String,
        support: bool,
    ) -> Result<(), String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        if Utc::now() > proposal.ends_at {
            return Err("Voting period has ended".to_string());
        }

        // Calculate vote weight including delegations
        let weight = self.calculate_vote_weight(&voter);
        let delegated_from = self.get_delegators(&voter);

        let vote = Vote {
            voter: voter.clone(),
            weight: if support { weight } else { -weight },
            timestamp: Utc::now(),
            delegated_from,
        };

        proposal.votes.insert(voter, vote);
        Ok(())
    }

    pub fn delegate_votes(&mut self, from: String, to: String) -> Result<(), String> {
        if from == to {
            return Err("Cannot delegate to self".to_string());
        }

        // Check for delegation cycles
        let mut visited = HashSet::new();
        let mut current = to.clone();
        visited.insert(from.clone());

        while let Some(delegate) = self.delegations.get(&current) {
            if !visited.insert(delegate.clone()) {
                return Err("Delegation would create a cycle".to_string());
            }
            current = delegate.clone();
        }

        self.delegations.insert(from, to);
        Ok(())
    }

    fn calculate_vote_weight(&self, voter: &str) -> i64 {
        let base_weight = self.reputation_weights.get(voter).cloned().unwrap_or(1);
        let delegated_weight: i64 = self.get_delegators(voter)
            .iter()
            .map(|delegator| self.reputation_weights.get(delegator).cloned().unwrap_or(1))
            .sum();

        base_weight + delegated_weight
    }

    fn get_delegators(&self, delegate: &str) -> Vec<String> {
        self.delegations
            .iter()
            .filter(|(_, to)| *to == delegate)
            .map(|(from, _)| from.clone())
            .collect()
    }

    pub fn finalize_proposal(&mut self, proposal_id: u64) -> Result<ProposalStatus, String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        if Utc::now() < proposal.ends_at {
            return Err("Voting period has not ended".to_string());
        }

        let total_possible_votes: i64 = self.reputation_weights.values().sum();
        let total_votes: i64 = proposal.votes.values()
            .map(|vote| vote.weight.abs())
            .sum();
        let total_support: i64 = proposal.votes.values()
            .map(|vote| vote.weight)
            .sum();

        let quorum = total_votes as f64 / total_possible_votes as f64;
        let majority = if total_votes > 0 {
            (total_support as f64 + total_votes as f64) / (2.0 * total_votes as f64)
        } else {
            0.0
        };

        let new_status = if quorum >= proposal.required_quorum 
            && majority >= proposal.required_majority {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        proposal.status = new_status.clone();
        Ok(new_status)
    }
}