//! Relationship System for Inter-Cooperative Network
//!
//! This module implements the core relationship tracking functionality for ICN,
//! focusing on human connections, mutual aid, and cooperative bonds rather than
//! purely transactional interactions.
//!
//! The system tracks:
//! - Contributions with their impact stories and peer verification
//! - Mutual aid interactions and reciprocity
//! - Ongoing relationships between members and cooperatives
//! - Endorsements and skill recognition
//!
//! Key principles:
//! - Emphasize qualitative relationships over quantitative metrics
//! - Support story-based impact documentation
//! - Enable peer recognition and verification
//! - Facilitate mutual aid coordination

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};

mod types;
pub use types::RelationshipType;

/// Records a concrete contribution made to the cooperative community.
/// Focuses on capturing both the action and its impact through storytelling.
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
    
    /// Context or category of the contribution (e.g., "education", "food-security")
    pub context: String,
    
    /// DIDs of members who witnessed or verified the contribution
    pub witnesses: Vec<String>,
    
    /// Feedback and endorsements from other members
    pub feedback: Vec<Feedback>,
    
    /// Tags for categorizing and finding related contributions
    pub tags: Vec<String>,
}

/// Qualitative feedback on a contribution from community members
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

/// Categories of endorsements that can be given
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

/// Documents mutual aid and resource sharing between members
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

/// Represents an ongoing relationship between members
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

/// Records individual interactions within a relationship
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

/// Categories of interactions between members
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

/// Endorsement of skills or qualities
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

/// Notes about a relationship
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

/// Visibility levels for relationship notes
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
}

impl RelationshipSystem {
    /// Creates a new relationship system
    pub fn new() -> Self {
        RelationshipSystem {
            contributions: Vec::new(),
            mutual_aid: Vec::new(),
            relationships: HashMap::new(),
            security_trust_scores: HashMap::new(),
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
        if !self.is_valid_member(&interaction.provider_did) || 
           !self.is_valid_member(&interaction.receiver_did) {
            return Err("Invalid member DID".to_string());
        }

        // Update relationship if it exists
        self.update_or_create_relationship(
            &interaction.provider_did,
            &interaction.receiver_did,
            &interaction.description,
        );

        // Record the interaction
        self.mutual_aid.push(interaction);
        Ok(())
    }

    /// Creates or updates a relationship between members
    pub fn update_relationship(&mut self, relationship: Relationship) -> Result<(), String> {
        let key = self.make_relationship_key(
            &relationship.member_one,
            &relationship.member_two
        );
        
        // Validate both members exist
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
        self.contributions
            .iter()
            .filter(|c| c.contributor_did == did)
            .collect()
    }

    /// Gets mutual aid history for a member
    pub fn get_mutual_aid_history(&self, did: &str) -> Vec<&MutualAidInteraction> {
        self.mutual_aid
            .iter()
            .filter(|m| m.provider_did == did || m.receiver_did == did)
            .collect()
    }

    /// Gets all relationships for a member
    pub fn get_member_relationships(&self, did: &str) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| r.member_one == did || r.member_two == did)
            .collect()
    }

    // Internal helper methods

    /// Validates that a member exists in the system
    fn is_valid_member(&self, _did: &str) -> bool {
        // In a real implementation, this would check against your identity system
        true // Simplified for example
    }

    /// Creates a consistent key for relationships regardless of member order
    fn make_relationship_key(&self, member_one: &str, member_two: &str) -> (String, String) {
        // Ensure consistent ordering
        if member_one < member_two {
            (member_one.to_string(), member_two.to_string())
        } else {
            (member_two.to_string(), member_one.to_string())
        }
    }

    /// Updates internal security score
    fn update_security_score(&mut self, did: &str, amount: i64) {
        let score = self.security_trust_scores.entry(did.to_string()).or_insert(0);
        *score += amount;
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
            relationship.interactions.push(Interaction {
                date: Utc::now(),
                description: context.to_string(),
                impact: None,
                interaction_type: InteractionType::ResourceExchange,
            });
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

// Implement the energy awareness trait
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