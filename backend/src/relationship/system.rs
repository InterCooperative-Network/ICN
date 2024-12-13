use std::collections::{HashMap, HashSet};
use chrono::Utc;

use super::models::{
    Contribution,
    MutualAidInteraction,
    Relationship,
    Interaction,
    InteractionType,
    Endorsement,
};
use crate::relationship::types::RelationshipType;
use crate::monitoring::energy::{EnergyAware, EnergyMonitor};

/// System for managing cooperative relationships.
pub struct RelationshipSystem {
    contributions: Vec<Contribution>,
    mutual_aid: Vec<MutualAidInteraction>,
    relationships: HashMap<(String, String), Relationship>,
    security_trust_scores: HashMap<String, i64>,
    valid_members: HashSet<String>,
}

impl RelationshipSystem {
    /// Creates a new RelationshipSystem.
    pub fn new() -> Self {
        RelationshipSystem {
            contributions: Vec::new(),
            mutual_aid: Vec::new(),
            relationships: HashMap::new(),
            security_trust_scores: HashMap::new(),
            valid_members: HashSet::new(),
        }
    }

    /// Records a contribution with validation and impact.
    pub fn record_contribution(&mut self, contribution: Contribution) -> Result<(), String> {
        if !self.is_valid_member(&contribution.contributor_did) {
            return Err("Contributor not found".to_string());
        }

        self.update_security_score(&contribution.contributor_did, 1);
        self.contributions.push(contribution);
        Ok(())
    }

    /// Records mutual aid interactions with validation.
    pub fn record_mutual_aid(&mut self, interaction: MutualAidInteraction) -> Result<(), String> {
        if !self.is_valid_member(&interaction.provider_did) || !self.is_valid_member(&interaction.receiver_did) {
            return Err("Invalid member DID".to_string());
        }

        self.update_or_create_relationship(
            &interaction.provider_did,
            &interaction.receiver_did,
            &interaction.description,
        );

        self.mutual_aid.push(interaction);
        Ok(())
    }

    /// Updates or creates a relationship between members.
    pub fn update_relationship(&mut self, relationship: Relationship) -> Result<(), String> {
        let key = self.make_relationship_key(&relationship.member_one, &relationship.member_two);

        if !self.is_valid_member(&relationship.member_one) || !self.is_valid_member(&relationship.member_two) {
            return Err("Invalid member DID".to_string());
        }

        self.relationships.insert(key, relationship);
        Ok(())
    }

    /// Adds an endorsement to an existing relationship.
    pub fn add_endorsement(
        &mut self,
        member_one: &str,
        member_two: &str,
        endorsement: Endorsement,
    ) -> Result<(), String> {
        let key = self.make_relationship_key(member_one, member_two);

        if let Some(relationship) = self.relationships.get_mut(&key) {
            relationship.mutual_endorsements.push(endorsement);
            Ok(())
        } else {
            Err("Relationship not found".to_string())
        }
    }

    /// Gets contribution history for a member.
    pub fn get_member_contributions(&self, did: &str) -> Vec<&Contribution> {
        self.contributions.iter().filter(|c| c.contributor_did == did).collect()
    }

    /// Gets mutual aid history for a member.
    pub fn get_mutual_aid_history(&self, did: &str) -> Vec<&MutualAidInteraction> {
        self.mutual_aid.iter().filter(|m| m.provider_did == did || m.receiver_did == did).collect()
    }

    /// Gets all relationships for a member.
    pub fn get_member_relationships(&self, did: &str) -> Vec<&Relationship> {
        self.relationships.values().filter(|r| r.member_one == did || r.member_two == did).collect()
    }

    /// Registers a valid member DID.
    pub fn register_member(&mut self, did: String) {
        self.valid_members.insert(did);
    }

    // --- Internal Methods ---

    /// Validates if a member exists.
    fn is_valid_member(&self, did: &str) -> bool {
        self.valid_members.contains(did)
    }

    /// Creates a key for relationships.
    fn make_relationship_key(&self, member_one: &str, member_two: &str) -> (String, String) {
        if member_one < member_two {
            (member_one.to_string(), member_two.to_string())
        } else {
            (member_two.to_string(), member_one.to_string())
        }
    }

    /// Updates a member's security trust score.
    fn update_security_score(&mut self, did: &str, amount: i64) {
        *self.security_trust_scores.entry(did.to_string()).or_insert(0) += amount;
    }

    /// Updates or creates a relationship for an interaction.
    fn update_or_create_relationship(&mut self, member_one: &str, member_two: &str, context: &str) {
        let key = self.make_relationship_key(member_one, member_two);

        if let Some(relationship) = self.relationships.get_mut(&key) {
            relationship.interactions.push(Interaction {
                date: Utc::now(),
                description: context.to_string(),
                impact: None,
                interaction_type: InteractionType::ResourceExchange,
            });
        } else {
            self.relationships.insert(
                key,
                Relationship {
                    member_one: member_one.to_string(),
                    member_two: member_two.to_string(),
                    relationship_type: RelationshipType::MutualAid,
                    started: Utc::now(),
                    story: format!("Started relationship with mutual aid: {}", context),
                    interactions: vec![Interaction {
                        date: Utc::now(),
                        description: context.to_string(),
                        impact: None,
                        interaction_type: InteractionType::ResourceExchange,
                    }],
                    mutual_endorsements: Vec::new(),
                    notes: Vec::new(),
                },
            );
        }
    }
}

impl EnergyAware for RelationshipSystem {
    fn record_energy_metrics(&self, monitor: &EnergyMonitor) {
        monitor.record_instruction();
        monitor.record_storage_operation((self.relationships.len() * std::mem::size_of::<Relationship>()) as u64);
        monitor.record_memory_operation((self.contributions.len() * std::mem::size_of::<Contribution>()) as u64);
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

    // ... [previous tests remain the same] ...
}
