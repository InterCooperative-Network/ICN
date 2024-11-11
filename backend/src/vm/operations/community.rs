// src/vm/operations/community.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, ensure_reputation, emit_event};
use crate::vm::VMError;

/// Operations specific to community management and civic participation
pub enum CommunityOperation {
    /// Create a new community
    CreateCommunity {
        name: String,
        description: String,
        governance_model: String,
    },
    
    /// Join an existing community
    JoinCommunity {
        community_id: String,
        role: String,
    },
    
    /// Leave a community
    LeaveCommunity {
        community_id: String,
    },
    
    /// Create a civic initiative
    CreateInitiative {
        community_id: String,
        title: String,
        description: String,
        category: String,
    },
    
    /// Support an initiative
    SupportInitiative {
        initiative_id: String,
        support_type: String,
    },
    
    /// Record civic participation
    RecordParticipation {
        community_id: String,
        activity_type: String,
        description: String,
        impact: String,
    },
    
    /// Update community guidelines
    UpdateGuidelines {
        community_id: String,
        updates: HashMap<String, String>,
    },
    
    /// Create working group
    CreateWorkingGroup {
        community_id: String,
        name: String,
        purpose: String,
        membership_criteria: String,
    },
    
    /// Coordinate inter-community action
    CoordinateAction {
        communities: Vec<String>,
        action_type: String,
        description: String,
        resources_needed: HashMap<String, u64>,
    },
}

impl Operation for CommunityOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            CommunityOperation::CreateCommunity { name, description, governance_model } => {
                // Verify permissions
                ensure_permissions(&["community.create".to_string()], &state.permissions)?;
                
                // Verify reputation requirements
                ensure_reputation(100, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                // Record the event
                let mut event_data = HashMap::new();
                event_data.insert("name".to_string(), name.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("governance_model".to_string(), governance_model.clone());
                
                emit_event(state, "CommunityCreated".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::JoinCommunity { community_id, role } => {
                // Basic reputation check for joining
                ensure_reputation(10, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("community_id".to_string(), community_id.clone());
                event_data.insert("role".to_string(), role.clone());
                
                emit_event(state, "CommunityJoined".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::CreateInitiative { community_id, title, description, category } => {
                // Ensure member has sufficient reputation for creating initiatives
                ensure_reputation(50, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("community_id".to_string(), community_id.clone());
                event_data.insert("title".to_string(), title.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("category".to_string(), category.clone());
                
                emit_event(state, "InitiativeCreated".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::RecordParticipation { community_id, activity_type, description, impact } => {
                let mut event_data = HashMap::new();
                event_data.insert("community_id".to_string(), community_id.clone());
                event_data.insert("activity_type".to_string(), activity_type.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("impact".to_string(), impact.clone());
                
                emit_event(state, "ParticipationRecorded".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::UpdateGuidelines { community_id, updates } => {
                // Verify permissions for guideline updates
                ensure_permissions(&["community.update_guidelines".to_string()], &state.permissions)?;
                ensure_reputation(200, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("community_id".to_string(), community_id.clone());
                for (key, value) in updates {
                    event_data.insert(format!("update_{}", key), value.clone());
                }
                
                emit_event(state, "GuidelinesUpdated".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::CreateWorkingGroup { community_id, name, purpose, membership_criteria } => {
                ensure_permissions(&["community.create_group".to_string()], &state.permissions)?;
                ensure_reputation(75, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("community_id".to_string(), community_id.clone());
                event_data.insert("name".to_string(), name.clone());
                event_data.insert("purpose".to_string(), purpose.clone());
                event_data.insert("membership_criteria".to_string(), membership_criteria.clone());
                
                emit_event(state, "WorkingGroupCreated".to_string(), event_data);
                Ok(())
            },

            CommunityOperation::CoordinateAction { communities, action_type, description, resources_needed } => {
                ensure_permissions(&["community.coordinate_action".to_string()], &state.permissions)?;
                ensure_reputation(150, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("communities".to_string(), communities.join(","));
                event_data.insert("action_type".to_string(), action_type.clone());
                event_data.insert("description".to_string(), description.clone());
                
                for (resource, amount) in resources_needed {
                    event_data.insert(format!("resource_{}", resource), amount.to_string());
                }
                
                emit_event(state, "InterCommunityActionCreated".to_string(), event_data);
                Ok(())
            },

            _ => Err(VMError::Custom("Operation not implemented".to_string())),
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            CommunityOperation::CreateCommunity { .. } => 200,
            CommunityOperation::JoinCommunity { .. } => 50,
            CommunityOperation::CreateInitiative { .. } => 100,
            CommunityOperation::RecordParticipation { .. } => 30,
            CommunityOperation::UpdateGuidelines { .. } => 150,
            CommunityOperation::CreateWorkingGroup { .. } => 120,
            CommunityOperation::CoordinateAction { .. } => 180,
            _ => 50, // Default cost for other operations
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            CommunityOperation::CreateCommunity { .. } => vec!["community.create".to_string()],
            CommunityOperation::UpdateGuidelines { .. } => vec!["community.update_guidelines".to_string()],
            CommunityOperation::CreateWorkingGroup { .. } => vec!["community.create_group".to_string()],
            CommunityOperation::CoordinateAction { .. } => vec!["community.coordinate_action".to_string()],
            _ => vec![], // Other operations might not require specific permissions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_state() -> VMState {
        let mut state = VMState::default();
        state.caller_did = "did:icn:test".to_string();
        state.reputation_context.insert(state.caller_did.clone(), 200);
        state.permissions = vec![
            "community.create".to_string(),
            "community.update_guidelines".to_string(),
            "community.create_group".to_string(),
        ];
        state
    }

    #[test]
    fn test_create_community() {
        let mut state = setup_test_state();
        let op = CommunityOperation::CreateCommunity {
            name: "Test Community".to_string(),
            description: "A test community".to_string(),
            governance_model: "democratic".to_string(),
        };

        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].event_type, "CommunityCreated");
    }

    // Add more tests for other operations...
}