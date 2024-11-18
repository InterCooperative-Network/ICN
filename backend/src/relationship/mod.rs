//! Relationship System for Inter-Cooperative Network
//!
//! This module implements the core relationship tracking functionality for ICN,
//! focusing on human connections, mutual aid, and cooperative bonds rather than
//! purely transactional interactions.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};

mod types;
pub use types::RelationshipType;

/// Records a concrete contribution made to the cooperative community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// DID of the contributing member
    pub contributor_did: String,
    
    /// Brief description of the contribution
    pub description: String,
    
    /// Detailed story about the contribution's impact on the community
    pub impact_story: String,
    
    /// When the contribution occurred
    pub date: DateTime<Utc>,
    
    /// Context or category of the contribution
    pub context: String,
    
    /// DIDs of members who witnessed or verified the contribution
    pub witnesses: Vec<String>,
    
    /// Feedback and endorsements from other members
    pub feedback: Vec<Feedback>,
    
    /// Tags for categorizing and finding related contributions
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// DID of the member providing feedback
    pub from_did: String,
    
    /// The actual feedback content
    pub content: String,
    
    /// When the feedback was given
    pub date: DateTime<Utc>,
    
    /// Type of endorsement this feedback represents
    pub endorsement_type: EndorsementType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndorsementType {
    /// Confirms the contribution occurred as described
    Verification,
    
    /// Speaks to the contribution's effect on the community
    Impact,
    
    /// Endorses the contributor's character and reliability
    Character,
    
    /// Validates specific skills demonstrated
    Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidInteraction {
    /// When the interaction occurred
    pub date: DateTime<Utc>,
    
    /// DID of the member providing aid
    pub provider_did: String,
    
    /// DID of the member receiving aid
    pub receiver_did: String,
    
    /// Description of what was shared or exchanged
    pub description: String,
    
    /// Story about how this aid impacted the community
    pub impact_story: Option<String>,
    
    /// Notes about reciprocity and relationship building
    pub reciprocity_notes: Option<String>,
    
    /// Tags for categorizing types of mutual aid
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// DID of the first member
    pub member_one: String,
    
    /// DID of the second member
    pub member_two: String,
    
    /// Type of relationship
    pub relationship_type: RelationshipType,
    
    /// When the relationship began
    pub started: DateTime<Utc>,
    
    /// Story of how the relationship formed and evolved
    pub story: String,
    
    /// History of interactions between members
    pub interactions: Vec<Interaction>,
    
    /// Endorsements members have given each other
    pub mutual_endorsements: Vec<Endorsement>,
    
    /// Notes about the relationship
    pub notes: Vec<RelationshipNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// When the interaction occurred
    pub date: DateTime<Utc>,
    
    /// Description of what happened
    pub description: String,
    
    /// Impact or outcome of the interaction
    pub impact: Option<String>,
    
    /// Type of interaction
    pub interaction_type: InteractionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    /// Working together
    Collaboration,
    
    /// Providing support
    Support,
    
    /// Exchanging resources
    ResourceExchange,
    
    /// Sharing knowledge
    KnowledgeSharing,
    
    /// Working through challenges
    ConflictResolution,
    
    /// Other types of interactions
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endorsement {
    /// DID of the endorsing member
    pub from_did: String,
    
    /// Content of the endorsement
    pub content: String,
    
    /// When the endorsement was given
    pub date: DateTime<Utc>,
    
    /// Context in which the skills were demonstrated
    pub context: String,
    
    /// Specific skills being endorsed
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNote {
    /// DID of the note author
    pub author_did: String,
    
    /// Content of the note
    pub content: String,
    
    /// When the note was written
    pub date: DateTime<Utc>,
    
    /// Who can see this note
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    /// Visible to everyone
    Public,
    
    /// Only visible to those in the relationship
    RelationshipParticipants,
    
    /// Visible to all cooperative members
    CooperativeMembers,
    
    /// Only visible to the author
    Private,
}

/// Main system for managing cooperative relationships
pub struct RelationshipSystem {
    /// Record of all contributions
    contributions: Vec<Contribution>,
    
    /// Record of mutual aid interactions
    mutual_aid: Vec<MutualAidInteraction>,
    
    /// Active relationships between members
    relationships: HashMap<(String, String), Relationship>,
    
    /// Internal trust scores for security validation
    /// Not exposed to users - used only for system security
    security_trust_scores: HashMap<String, i64>,
    
    /// Cached member validation info
    valid_members: HashSet<String>,
}

impl RelationshipSystem {
    /// Creates a new relationship system
    pub fn new() -> Self {
        RelationshipSystem {
            contributions: Vec::new(),
            mutual_aid: Vec::new(),
            relationships: HashMap::new(),
            security_trust_scores: HashMap::new(),
            valid_members: HashSet::new(),
        }
    }

    /// Records a new contribution with its story and impact
    pub fn record_contribution(&mut self, contribution: Contribution) -> Result<(), String> {
        if !self.is_valid_member(&contribution.contributor_did) {
            return Err("Contributor not found".to_string());
        }

        // Update internal security score (not exposed to users)
        self.update_security_score(&contribution.contributor_did, 1);
        
        // Record the contribution
        self.contributions.push(contribution);
        Ok(())
    }

    /// Records mutual aid interaction between members
    pub fn record_mutual_aid(&mut self, interaction: MutualAidInteraction) -> Result<(), String> {
        // Validate both members
        if !self.is_valid_member(&interaction.provider_did) || 
           !self.is_valid_member(&interaction.receiver_did) {
            return Err("Invalid member DID".to_string());
        }

        // Update relationship
        self.update_or_create_relationship(
            &interaction.provider_did,
            &interaction.receiver_did,
            &interaction.description,
        );

        // Record interaction
        self.mutual_aid.push(interaction);
        Ok(())
    }

    /// Creates or updates a relationship between members
    pub fn update_relationship(&mut self, relationship: Relationship) -> Result<(), String> {
        // Make relationship key consistent
        let key = self.make_relationship_key(
            &relationship.member_one,
            &relationship.member_two
        );
        
        // Validate members
        if !self.is_valid_member(&relationship.member_one) || 
           !self.is_valid_member(&relationship.member_two) {
            return Err("Invalid member DID".to_string());
        }

        self.relationships.insert(key, relationship);
        Ok(())
    }

    /// Adds an endorsement to an existing relationship
    pub fn add_endorsement(
        &mut self,
        member_one: &str,
        member_two: &str,
        endorsement: Endorsement
    ) -> Result<(), String> {
        let key = self.make_relationship_key(member_one, member_two);
        
        if let Some(relationship) = self.relationships.get_mut(&key) {
            relationship.mutual_endorsements.push(endorsement);
            Ok(())
        } else {
            Err("Relationship not found".to_string())
        }
    }

    /// Gets member's contribution history with impact stories
    pub fn get_member_contributions(&self, did: &str) -> Vec<&Contribution> {
        self.contributions.iter()
            .filter(|c| c.contributor_did == did)
            .collect()
    }

    /// Gets mutual aid history for a member
    pub fn get_mutual_aid_history(&self, did: &str) -> Vec<&MutualAidInteraction> {
        self.mutual_aid.iter()
            .filter(|m| m.provider_did == did || m.receiver_did == did)
            .collect()
    }

    /// Gets all relationships for a member
    pub fn get_member_relationships(&self, did: &str) -> Vec<&Relationship> {
        self.relationships.values()
            .filter(|r| r.member_one == did || r.member_two == did)
            .collect()
    }

    /// Register a valid member DID
    pub fn register_member(&mut self, did: String) {
        self.valid_members.insert(did);
    }

    // Internal helper methods

    /// Validates that a member exists in the system
    fn is_valid_member(&self, did: &str) -> bool {
        self.valid_members.contains(did)
    }

    /// Creates a consistent key for relationships regardless of member order
    fn make_relationship_key(&self, member_one: &str, member_two: &str) -> (String, String) {
        if member_one < member_two {
            (member_one.to_string(), member_two.to_string())
        } else {
            (member_two.to_string(), member_one.to_string())
        }
    }

    /// Updates internal security score
    fn update_security_score(&mut self, did: &str, amount: i64) {
        *self.security_trust_scores.entry(did.to_string()).or_insert(0) += amount;
    }

    /// Creates or updates a relationship based on an interaction
    fn update_or_create_relationship(
        &mut self,
        member_one: &str,
        member_two: &str,
        context: &str
    ) {
        let key = self.make_relationship_key(member_one, member_two);
        
        if let Some(relationship) = self.relationships.get_mut(&key) {
            let interaction = Interaction {
                date: Utc::now(),
                description: context.to_string(),
                impact: None,
                interaction_type: InteractionType::ResourceExchange,
            };
            relationship.interactions.push(interaction);
        } else {
            let new_relationship = Relationship {
                member_one: member_one.to_string(),
                member_two: member_two.to_string(),
                relationship_type: RelationshipType::MutualAid,
                started: Utc::now(),
                story: format!("Relationship started with mutual aid: {}", context),
                interactions: vec![Interaction {
                    date: Utc::now(),
                    description: context.to_string(),
                    impact: None,
                    interaction_type: InteractionType::ResourceExchange,
                }],
                mutual_endorsements: Vec::new(),
                notes: Vec::new(),
            };
            self.relationships.insert(key, new_relationship);
        }
    }
}

impl EnergyAware for RelationshipSystem {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        // Record basic operations
        monitor.record_instruction();
        
        // Record storage based on relationship count
        let storage_size = (self.relationships.len() * std::mem::size_of::<Relationship>()) as u64;
        monitor.record_storage_operation(storage_size);
        
        // Record contribution operations
        let contributions_size = (self.contributions.len() * std::mem::size_of::<Contribution>()) as u64;
        monitor.record_memory_operation(contributions_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_system() -> RelationshipSystem {
        let mut system = RelationshipSystem::new();
        system.register_member("test_did".to_string());
        system.register_member("test_did2".to_string());
        system
    }

    #[test]
    fn test_record_contribution() {
        let mut system = setup_test_system();
        
        let contribution = Contribution {
            contributor_did: "test_did".to_string(),
            description: "Test contribution".to_string(),
            impact_story: "Made an impact".to_string(),
            date: Utc::now(),
            context: "test".to_string(),
            witnesses: vec![],
            feedback: vec![],
            tags: vec!["test".to_string()],
        };

        assert!(system.record_contribution(contribution).is_ok());
    }

    #[test]
    fn test_mutual_aid() {
        let mut system = setup_test_system();
        
        let interaction = MutualAidInteraction {
            date: Utc::now(),
            provider_did: "test_did".to_string(),
            receiver_did: "test_did2".to_string(),
            description: "Helped with project".to_string(),
            impact_story: Some("Great collaboration".to_string()),
            reciprocity_notes: None,
            tags: vec!["help".to_string()],
        };

        assert!(system.record_mutual_aid(interaction).is_ok());
    }

    #[test]
    fn test_invalid_member() {
        let mut system = setup_test_system();
        
        let contribution = Contribution {
            contributor_did: "invalid_did".to_string(),
            description: "Test".to_string(),
            impact_story: "Test".to_string(),
            date: Utc::now(),
            context: "test".to_string(),
            witnesses: vec![],
            feedback: vec![],
            tags: vec![],
        };

        assert!(system.record_contribution(contribution).is_err());
    }
}