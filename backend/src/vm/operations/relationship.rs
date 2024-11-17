// src/vm/operations/relationship.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::relationship::Contribution;

/// Operations for managing relationships between cooperatives and members
pub enum RelationshipOperation {
    /// Record a contribution with impact story
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
        witnesses: Vec<String>,
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
    
    /// Add feedback to contribution
    AddFeedback {
        contribution_id: String,
        endorsement_type: String,
        content: String,
        impact_rating: Option<u8>,
    },
}

impl Operation for RelationshipOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            RelationshipOperation::RecordContribution { 
                description,
                impact_story,
                context,
                tags,
                witnesses 
            } => {
                ensure_permissions(&["contribution.record".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("impact_story".to_string(), impact_story.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("witness_count".to_string(), witnesses.len().to_string());
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
            
            RelationshipOperation::AddFeedback { 
                contribution_id,
                endorsement_type,
                content,
                impact_rating 
            } => {
                ensure_permissions(&["feedback.add".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("contribution_id".to_string(), contribution_id.clone());
                event_data.insert("endorsement_type".to_string(), endorsement_type.clone());
                event_data.insert("content".to_string(), content.clone());
                if let Some(rating) = impact_rating {
                    event_data.insert("impact_rating".to_string(), rating.to_string());
                }
                
                emit_event(state, "FeedbackAdded".to_string(), event_data);
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
            RelationshipOperation::AddFeedback { .. } => 40,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            RelationshipOperation::RecordContribution { .. } => vec!["contribution.record".to_string()],
            RelationshipOperation::RecordMutualAid { .. } => vec!["mutual_aid.record".to_string()],
            RelationshipOperation::UpdateRelationship { .. } => vec!["relationship.update".to_string()],
            RelationshipOperation::AddEndorsement { .. } => vec!["endorsement.add".to_string()],
            RelationshipOperation::AddFeedback { .. } => vec!["feedback.add".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relationship::{Contribution, MutualAidInteraction};

    fn setup_test_state() -> VMState {
        let mut state = VMState::new(
            "test_caller".to_string(),
            1,
            1000
        );
        state.permissions = vec![
            "contribution.record".to_string(),
            "mutual_aid.record".to_string(),
            "endorsement.add".to_string(),
            "feedback.add".to_string(),
        ];
        state
    }

    #[test]
    fn test_record_contribution() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::RecordContribution {
            description: "Test contribution".to_string(),
            impact_story: "Made a difference".to_string(),
            context: "Testing".to_string(),
            tags: vec!["test".to_string()],
            witnesses: vec!["witness1".to_string()],
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
    fn test_update_relationship() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::UpdateRelationship {
            member_two: "other_member".to_string(),
            relationship_type: "Collaboration".to_string(),
            story: "Working together".to_string(),
            interaction: Some("First meeting".to_string()),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "RelationshipUpdated");
    }

    #[test]
    fn test_add_endorsement() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::AddEndorsement {
            to_did: "endorsed_member".to_string(),
            content: "Great collaboration".to_string(),
            context: "Project work".to_string(),
            skills: vec!["teamwork".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "EndorsementAdded");
    }

    #[test]
    fn test_add_feedback() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::AddFeedback {
            contribution_id: "contribution1".to_string(),
            endorsement_type: "Positive".to_string(),
            content: "Excellent work".to_string(),
            impact_rating: Some(5),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FeedbackAdded");
    }

    #[test]
    fn test_missing_permissions() {
        let mut state = setup_test_state();
        state.permissions.clear();
        
        let op = RelationshipOperation::RecordContribution {
            description: "Test".to_string(),
            impact_story: "Test".to_string(),
            context: "Test".to_string(),
            tags: vec![],
            witnesses: vec![],
        };
        
        assert!(op.execute(&mut state).is_err());
    }
}