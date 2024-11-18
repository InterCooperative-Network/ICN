// src/vm/operations/relationship.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::relationship::RelationshipType;

#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    Collaboration,
    Mentorship,
    MutualAid,
    Custom(String),
}

/// Operations for managing relationships between members
pub enum RelationshipOperation {
    /// Record a contribution with impact story
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    
    /// Record mutual aid interaction
    RecordMutualAid {
        receiver_did: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
    
    /// Update relationship between members
    UpdateRelationship {
        member_two: String,
        relationship_type: String,
        story: String,
        interaction: Option<String>,
    },
    
    /// Add endorsement to relationship
    AddEndorsement {
        to_did: String,
        content: String,
        context: String,
        skills: Vec<String>,
    },
}

impl Operation for RelationshipOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            RelationshipOperation::RecordContribution { 
                description,
                impact_story,
                context,
                tags 
            } => {
                ensure_permissions(&["contribution.record".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("impact_story".to_string(), impact_story.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("tags".to_string(), tags.join(","));
                
                emit_event(state, "ContributionRecorded".to_string(), event_data);
                Ok(())
            },

            RelationshipOperation::RecordMutualAid { 
                receiver_did,
                description,
                impact_story,
                reciprocity_notes,
                tags 
            } => {
                ensure_permissions(&["mutual_aid.record".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("receiver_did".to_string(), receiver_did.clone());
                event_data.insert("description".to_string(), description.clone());
                
                if let Some(impact) = impact_story {
                    event_data.insert("impact_story".to_string(), impact.clone());
                }
                
                if let Some(notes) = reciprocity_notes {
                    event_data.insert("reciprocity_notes".to_string(), notes.clone());
                }
                
                event_data.insert("tags".to_string(), tags.join(","));
                
                emit_event(state, "MutualAidRecorded".to_string(), event_data);
                Ok(())
            },

            RelationshipOperation::UpdateRelationship { 
                member_two,
                relationship_type,
                story,
                interaction 
            } => {
                ensure_permissions(&["relationship.update".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("member_two".to_string(), member_two.clone());
                event_data.insert("relationship_type".to_string(), relationship_type.clone());
                event_data.insert("story".to_string(), story.clone());
                
                if let Some(interaction_data) = interaction {
                    event_data.insert("interaction".to_string(), interaction_data.clone());
                }
                
                emit_event(state, "RelationshipUpdated".to_string(), event_data);
                Ok(())
            },

            RelationshipOperation::AddEndorsement { 
                to_did,
                content,
                context,
                skills 
            } => {
                ensure_permissions(&["endorsement.add".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("to_did".to_string(), to_did.clone());
                event_data.insert("content".to_string(), content.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("skills".to_string(), skills.join(","));
                
                emit_event(state, "EndorsementAdded".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            RelationshipOperation::RecordContribution { .. } => 100,
            RelationshipOperation::RecordMutualAid { .. } => 80,
            RelationshipOperation::UpdateRelationship { .. } => 60,
            RelationshipOperation::AddEndorsement { .. } => 50,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            RelationshipOperation::RecordContribution { .. } => vec!["contribution.record".to_string()],
            RelationshipOperation::RecordMutualAid { .. } => vec!["mutual_aid.record".to_string()],
            RelationshipOperation::UpdateRelationship { .. } => vec!["relationship.update".to_string()],
            RelationshipOperation::AddEndorsement { .. } => vec!["endorsement.add".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    fn setup_test_state() -> VMState {
        VMState {
            stack: Vec::new(),
            memory: HashMap::new(),
            events: Vec::new(),
            instruction_pointer: 0,
            reputation_context: HashMap::new(),
            caller_did: "test_caller".to_string(),
            block_number: 1,
            timestamp: 1000,
            permissions: vec![
                "contribution.record".to_string(),
                "mutual_aid.record".to_string(),
            ],
            memory_limit: 1024 * 1024, // 1MB
            memory_address_counter: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_record_contribution() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::RecordContribution {
            description: "Test contribution".to_string(),
            impact_story: "Made a difference".to_string(),
            context: "Testing".to_string(),
            tags: vec!["test".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "ContributionRecorded");
    }

    #[test]
    fn test_record_mutual_aid() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::RecordMutualAid {
            receiver_did: "receiver".to_string(),
            description: "Help provided".to_string(),
            impact_story: Some("Positive impact".to_string()),
            reciprocity_notes: None,
            tags: vec!["help".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MutualAidRecorded");
    }

    #[test]
    fn test_insufficient_permissions() {
        let mut state = setup_test_state();
        state.permissions.clear();
        
        let op = RelationshipOperation::RecordContribution {
            description: "Test".to_string(),
            impact_story: "Test".to_string(),
            context: "Test".to_string(),
            tags: vec![],
        };
        
        assert!(op.execute(&mut state).is_err());
    }

    #[test]
    fn test_update_relationship() {
        let mut state = setup_test_state();
        state.permissions.push("relationship.update".to_string());

        let op = RelationshipOperation::UpdateRelationship {
            member_two: "other_member".to_string(),
            relationship_type: "collaboration".to_string(),
            story: "Working together".to_string(),
            interaction: Some("Initial meeting".to_string()),
        };

        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "RelationshipUpdated");
    }

    #[test]
    fn test_add_endorsement() {
        let mut state = setup_test_state();
        state.permissions.push("endorsement.add".to_string());

        let op = RelationshipOperation::AddEndorsement {
            to_did: "endorsed_member".to_string(),
            content: "Great collaboration!".to_string(),
            context: "Project work".to_string(),
            skills: vec!["teamwork".to_string(), "communication".to_string()],
        };

        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "EndorsementAdded");
    }
}