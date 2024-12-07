// src/vm/contracts/voting_rules.rs

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomVotingRules {
    pub cooperative_id: String,
    pub use_reputation_weighting: bool,
    pub min_reputation_to_vote: Option<i64>,
    pub max_vote_weight: Option<f64>,
    pub quorum_requirement: f64,
    pub special_majority_requirement: Option<f64>,
    pub proposal_categories: HashMap<String, ProposalRules>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProposalRules {
    pub category: String,
    pub quorum_requirement: f64,
    pub approval_threshold: f64,
    pub reputation_multiplier: Option<f64>,
}

impl VotingRules for CustomVotingRules {
    fn calculate_vote_weight(&self, voter: &str, context: &VotingContext) -> f64 {
        if !self.use_reputation_weighting {
            return 1.0;
        }

        let reputation = context.reputation_scores
            .as_ref()
            .and_then(|scores| scores.get(voter))
            .copied()
            .unwrap_or(0);

        // Check minimum reputation requirement
        if let Some(min_rep) = self.min_reputation_to_vote {
            if reputation < min_rep {
                return 0.0;
            }
        }

        let category_rules = self.proposal_categories
            .get(&context.proposal_type)
            .cloned()
            .unwrap_or_default();

        // Calculate weight based on reputation
        let base_weight = if reputation <= 0 { 
            0.0 
        } else {
            1.0 + (reputation as f64 * category_rules.reputation_multiplier.unwrap_or(0.0))
        };

        // Apply maximum weight cap if configured
        if let Some(max_weight) = self.max_vote_weight {
            base_weight.min(max_weight)
        } else {
            base_weight
        }
    }

    fn is_proposal_approved(&self, votes: &HashMap<String, Vote>, context: &VotingContext) -> bool {
        let total_weights: f64 = votes.keys()
            .map(|voter| self.calculate_vote_weight(voter, context))
            .sum();

        let approval_weights: f64 = votes.iter()
            .filter(|(_, vote)| vote.approve)
            .map(|(voter, _)| self.calculate_vote_weight(voter, context))
            .sum();

        // Get category-specific rules
        let category_rules = self.proposal_categories
            .get(&context.proposal_type)
            .cloned()
            .unwrap_or_default();

        // Check quorum
        let quorum = category_rules.quorum_requirement.max(self.quorum_requirement);
        let participation_rate = votes.len() as f64 / context.total_members as f64;
        
        if participation_rate < quorum {
            return false;
        }

        // Check approval threshold
        let approval_rate = if total_weights > 0.0 {
            approval_weights / total_weights
        } else {
            0.0
        };

        approval_rate >= category_rules.approval_threshold
    }

    fn get_quorum_requirement(&self, context: &VotingContext) -> f64 {
        self.proposal_categories
            .get(&context.proposal_type)
            .map(|rules| rules.quorum_requirement)
            .unwrap_or(self.quorum_requirement)
    }
}

// Helper to create common voting rule configurations
pub fn create_voting_rules(cooperative_id: String) -> CustomVotingRules {
    let mut proposal_categories = HashMap::new();
    
    // Standard proposals - simple majority
    proposal_categories.insert("standard".to_string(), ProposalRules {
        category: "standard".to_string(),
        quorum_requirement: 0.5,
        approval_threshold: 0.5,
        reputation_multiplier: None,
    });
    
    // Resource allocation - higher requirements
    proposal_categories.insert("resource".to_string(), ProposalRules {
        category: "resource".to_string(),
        quorum_requirement: 0.6,
        approval_threshold: 0.6,
        reputation_multiplier: None,
    });
    
    // Critical changes - supermajority required
    proposal_categories.insert("critical".to_string(), ProposalRules {
        category: "critical".to_string(),
        quorum_requirement: 0.75,
        approval_threshold: 0.75,
        reputation_multiplier: None,
    });

    CustomVotingRules {
        cooperative_id,
        use_reputation_weighting: false,
        min_reputation_to_vote: None,
        max_vote_weight: None,
        quorum_requirement: 0.5,
        special_majority_requirement: None,
        proposal_categories,
    }
}