// src/vm/operations/relationship.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, ensure_reputation, emit_event};
use crate::vm::VMError;

/// Operations for managing relationships and social connections
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
        recipient_did: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
    
    /// Update relationship status
    UpdateRelationship {
        member_did: String,
        relationship_type: RelationType,
        story: String,
        strength_indicators: Vec<StrengthIndicator>,
    },
    
    /// Add endorsement for skills or character
    AddEndorsement {
        member_did: String,
        endorsement_type: EndorsementType,
        content: String,
        context: String,
        skills: Vec<String>,
    },
    
    /// Record interaction or collaboration
    RecordInteraction {
        member_did: String,
        interaction_type: InteractionType,
        description: String,
        impact: Option<String>,
        outcomes: Vec<String>,
    },
    
    /// Add witness to contribution
    AddWitness {
        contribution_id: String,
        witness_did: String,
        attestation: String,
    },
    
    /// Provide feedback on contribution
    AddFeedback {
        contribution_id: String,
        feedback_type: FeedbackType,
        content: String,
        impact_rating: Option<u8>,
    },
    
    /// Create knowledge sharing record
    RecordKnowledgeSharing {
        recipients: Vec<String>,
        knowledge_type: String,
        description: String,
        outcomes: Vec<String>,
    },
    
    /// Record conflict resolution
    RecordConflictResolution {
        participants: Vec<String>,
        description: String,
        resolution: String,
        learnings: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum RelationType {
    Collaboration,
    Mentorship,
    PeerSupport,
    ResourceSharing,
    KnowledgeExchange,
    ConflictResolution,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum EndorsementType {
    Skill,
    Character,
    Contribution,
    Reliability,
    Cooperation,
    Leadership,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum InteractionType {
    Collaboration,
    Support,
    Learning,
    ResourceExchange,
    ConflictResolution,
    FeedbackSession,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum FeedbackType {
    Verification,
    Impact,
    Suggestion,
    Recognition,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct StrengthIndicator {
    pub indicator_type: String,
    pub value: u8,
    pub context: String,
}

impl Operation for RelationshipOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            RelationshipOperation::RecordContribution { 
                description,
                impact_story,
                context,
                tags,
                witnesses,
            } => {
                ensure_permissions(&["contribution.record".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("impact_story".to_string(), impact_story.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("tags".to_string(), tags.join(","));
                event_data.insert("witness_count".to_string(), witnesses.len().to_string());
                
                emit_event(state, "ContributionRecorded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::RecordMutualAid {
                recipient_did,
                description,
                impact_story,
                reciprocity_notes,
                tags,
            } => {
                ensure_permissions(&["mutual_aid.record".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("recipient_did".to_string(), recipient_did.clone());
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
                member_did,
                relationship_type,
                story,
                strength_indicators,
            } => {
                let mut event_data = HashMap::new();
                event_data.insert("member_did".to_string(), member_did.clone());
                event_data.insert("relationship_type".to_string(), format!("{:?}", relationship_type));
                event_data.insert("story".to_string(), story.clone());
                event_data.insert("indicator_count".to_string(), strength_indicators.len().to_string());
                
                emit_event(state, "RelationshipUpdated".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::AddEndorsement {
                member_did,
                endorsement_type,
                content,
                context,
                skills,
            } => {
                ensure_permissions(&["endorsement.create".to_string()], &state.permissions)?;
                
                // Require some reputation to make endorsements
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                ensure_reputation(50, reputation)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("member_did".to_string(), member_did.clone());
                event_data.insert("endorsement_type".to_string(), format!("{:?}", endorsement_type));
                event_data.insert("content".to_string(), content.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("skills".to_string(), skills.join(","));
                
                emit_event(state, "EndorsementAdded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::RecordInteraction {
                member_did,
                interaction_type,
                description,
                impact,
                outcomes,
            } => {
                let mut event_data = HashMap::new();
                event_data.insert("member_did".to_string(), member_did.clone());
                event_data.insert("interaction_type".to_string(), format!("{:?}", interaction_type));
                event_data.insert("description".to_string(), description.clone());
                if let Some(imp) = impact {
                    event_data.insert("impact".to_string(), imp.clone());
                }
                event_data.insert("outcomes".to_string(), outcomes.join(","));
                
                emit_event(state, "InteractionRecorded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::AddWitness {
                contribution_id,
                witness_did,
                attestation,
            } => {
                ensure_permissions(&["witness.add".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("contribution_id".to_string(), contribution_id.clone());
                event_data.insert("witness_did".to_string(), witness_did.clone());
                event_data.insert("attestation".to_string(), attestation.clone());
                
                emit_event(state, "WitnessAdded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::AddFeedback {
                contribution_id,
                feedback_type,
                content,
                impact_rating,
            } => {
                ensure_permissions(&["feedback.add".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("contribution_id".to_string(), contribution_id.clone());
                event_data.insert("feedback_type".to_string(), format!("{:?}", feedback_type));
                event_data.insert("content".to_string(), content.clone());
                if let Some(rating) = impact_rating {
                    event_data.insert("impact_rating".to_string(), rating.to_string());
                }
                
                emit_event(state, "FeedbackAdded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::RecordKnowledgeSharing {
                recipients,
                knowledge_type,
                description,
                outcomes,
            } => {
                let mut event_data = HashMap::new();
                event_data.insert("recipients".to_string(), recipients.join(","));
                event_data.insert("knowledge_type".to_string(), knowledge_type.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("outcomes".to_string(), outcomes.join(","));
                
                emit_event(state, "KnowledgeSharingRecorded".to_string(), event_data);
                Ok(())
            },
            
            RelationshipOperation::RecordConflictResolution {
                participants,
                description,
                resolution,
                learnings,
            } => {
                ensure_permissions(&["conflict.resolve".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("participants".to_string(), participants.join(","));
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("resolution".to_string(), resolution.clone());
                event_data.insert("learnings".to_string(), learnings.join(","));
                
                emit_event(state, "ConflictResolutionRecorded".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            RelationshipOperation::RecordContribution { .. } => 100,
            RelationshipOperation::RecordMutualAid { .. } => 80,
            RelationshipOperation::UpdateRelationship { .. } => 50,
            RelationshipOperation::AddEndorsement { .. } => 70,
            RelationshipOperation::RecordInteraction { .. } => 40,
            RelationshipOperation::AddWitness { .. } => 30,
            RelationshipOperation::AddFeedback { .. } => 40,
            RelationshipOperation::RecordKnowledgeSharing { .. } => 60,
            RelationshipOperation::RecordConflictResolution { .. } => 90,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            RelationshipOperation::RecordContribution { .. } => vec!["contribution.record".to_string()],
            RelationshipOperation::RecordMutualAid { .. } => vec!["mutual_aid.record".to_string()],
            RelationshipOperation::AddEndorsement { .. } => vec!["endorsement.create".to_string()],
            RelationshipOperation::AddWitness { .. } => vec!["witness.add".to_string()],
            RelationshipOperation::AddFeedback { .. } => vec!["feedback.add".to_string()],
            RelationshipOperation::RecordConflictResolution { .. } => vec!["conflict.resolve".to_string()],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_state() -> VMState {
        let mut state = VMState {
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
                "endorsement.create".to_string(),
                "witness.add".to_string(),
            ],
        };
        
        state.reputation_context.insert(state.caller_did.clone(), 100);
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
            recipient_did: "recipient".to_string(),
            description: "Helped with project".to_string(),
            impact_story: Some("Positive impact".to_string()),
            reciprocity_notes: None,
            tags: vec!["help".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MutualAidRecorded");
    }

    #[test]
    fn test_add_endorsement() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::AddEndorsement {
            member_did: "member".to_string(),
            endorsement_type: EndorsementType::Skill,
            content: "Great skills".to_string(),
            context: "Project work".to_string(),
            skills: vec!["coding".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "EndorsementAdded");
    }

    #[test]
    fn test_record_conflict_resolution() {
        let mut state = setup_test_state();
        state.permissions.push("conflict.resolve".to_string());
        
        let op = RelationshipOperation::RecordConflictResolution {
            participants: vec!["member1".to_string(), "member2".to_string()],
            description: "Resource allocation dispute".to_string(),
            resolution: "Agreed to share".to_string(),
            learnings: vec!["Better communication needed".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "ConflictResolutionRecorded");
    }

    #[test]
    fn test_add_feedback() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::AddFeedback {
            contribution_id: "contribution1".to_string(),
            feedback_type: FeedbackType::Impact,
            content: "This contribution had a significant positive effect.".to_string(),
            impact_rating: Some(8),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FeedbackAdded");
        assert_eq!(state.events[0].data.get("content").unwrap(), "This contribution had a significant positive effect.");
        assert_eq!(state.events[0].data.get("impact_rating").unwrap(), "8");
    }

    #[test]
    fn test_update_relationship() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::UpdateRelationship {
            member_did: "member1".to_string(),
            relationship_type: RelationType::Mentorship,
            story: "Guided member1 through project challenges.".to_string(),
            strength_indicators: vec![
                StrengthIndicator {
                    indicator_type: "Trust".to_string(),
                    value: 9,
                    context: "Mentorship during project".to_string(),
                },
                StrengthIndicator {
                    indicator_type: "Reliability".to_string(),
                    value: 8,
                    context: "Consistently available for support".to_string(),
                },
            ],
        };

        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "RelationshipUpdated");
        assert_eq!(state.events[0].data.get("member_did").unwrap(), "member1");
        assert_eq!(state.events[0].data.get("relationship_type").unwrap(), "Mentorship");
    }

    #[test]
    fn test_record_interaction() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::RecordInteraction {
            member_did: "member2".to_string(),
            interaction_type: InteractionType::Collaboration,
            description: "Collaborated on implementing a new module.".to_string(),
            impact: Some("Improved overall efficiency.".to_string()),
            outcomes: vec!["Module completed".to_string(), "Team learned new skills".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "InteractionRecorded");
        assert_eq!(state.events[0].data.get("member_did").unwrap(), "member2");
        assert_eq!(state.events[0].data.get("interaction_type").unwrap(), "Collaboration");
        assert!(state.events[0].data.get("outcomes").unwrap().contains("Module completed"));
    }

    #[test]
    fn test_record_knowledge_sharing() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::RecordKnowledgeSharing {
            recipients: vec!["member1".to_string(), "member3".to_string()],
            knowledge_type: "Technical Knowledge".to_string(),
            description: "Provided an overview of advanced Rust techniques.".to_string(),
            outcomes: vec!["Improved coding practices".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "KnowledgeSharingRecorded");
        assert_eq!(state.events[0].data.get("recipients").unwrap(), "member1,member3");
        assert_eq!(state.events[0].data.get("knowledge_type").unwrap(), "Technical Knowledge");
    }

    #[test]
    fn test_add_witness() {
        let mut state = setup_test_state();
        let op = RelationshipOperation::AddWitness {
            contribution_id: "contribution123".to_string(),
            witness_did: "witness1".to_string(),
            attestation: "I witnessed this contribution.".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "WitnessAdded");
        assert_eq!(state.events[0].data.get("contribution_id").unwrap(), "contribution123");
        assert_eq!(state.events[0].data.get("witness_did").unwrap(), "witness1");
    }

    #[test]
    fn test_record_conflict_resolution() {
        let mut state = setup_test_state();
        state.permissions.push("conflict.resolve".to_string());

        let op = RelationshipOperation::RecordConflictResolution {
            participants: vec!["member1".to_string(), "member2".to_string()],
            description: "Disagreement over project timeline.".to_string(),
            resolution: "Compromised on a new timeline.".to_string(),
            learnings: vec!["Importance of early communication".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "ConflictResolutionRecorded");
        assert_eq!(state.events[0].data.get("participants").unwrap(), "member1,member2");
        assert_eq!(state.events[0].data.get("resolution").unwrap(), "Compromised on a new timeline.");
    }
}
