// src/vm/operations/federation.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, ensure_reputation, emit_event};

/// Types of federation that can be formed
#[derive(Debug, Clone, PartialEq)]
pub enum FederationType {
    /// Federation between cooperatives for resource sharing
    CooperativeFederation,
    /// Federation between communities for civic coordination
    CommunityFederation,
    /// Hybrid federation involving both cooperatives and communities
    HybridFederation,
}

/// Federation agreement terms and conditions
#[derive(Debug, Clone)]
pub struct FederationTerms {
    /// Type of federation being formed
    federation_type: FederationType,
    /// Minimum reputation required to participate
    min_reputation: i64,
    /// Resource sharing policies
    resource_policies: Vec<String>,
    /// Governance rules for federation
    governance_rules: Vec<String>,
    /// Duration of federation agreement
    duration_days: u64,
}

/// Operations for managing federation between cooperatives and communities
pub enum FederationOperation {
    /// Initiate a new federation
    InitiateFederation {
        federation_type: FederationType,
        partner_id: String,
        terms: FederationTerms,
    },
    
    /// Join an existing federation
    JoinFederation {
        federation_id: String,
        commitment: Vec<String>,
    },
    
    /// Leave a federation
    LeaveFederation {
        federation_id: String,
        reason: String,
    },
    
    /// Propose action within federation
    ProposeAction {
        federation_id: String,
        action_type: String,
        description: String,
        resources: HashMap<String, u64>,
    },
    
    /// Vote on federation proposal
    VoteOnProposal {
        federation_id: String,
        proposal_id: String,
        approve: bool,
        notes: Option<String>,
    },
    
    /// Share resources within federation
    ShareResources {
        federation_id: String,
        resource_type: String,
        amount: u64,
        recipient_id: String,
    },
    
    /// Synchronize federation state
    SyncFederationState {
        federation_id: String,
    },
    
    /// Update federation terms
    UpdateFederationTerms {
        federation_id: String,
        new_terms: FederationTerms,
    },
    
    /// Add cross-federation endorsement
    AddEndorsement {
        federation_id: String,
        target_id: String,
        endorsement_type: String,
        content: String,
    },
    
    /// Resolve federation dispute
    ResolveDispute {
        federation_id: String,
        dispute_id: String,
        resolution: String,
        mediators: Vec<String>,
    },
    
    /// Coordinate joint action across federation
    CoordinateAction {
        federation_id: String,
        action_type: String,
        participants: Vec<String>,
        resources: HashMap<String, u64>,
        timeline: String,
    },
}

impl Operation for FederationOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            FederationOperation::InitiateFederation { federation_type, partner_id, terms } => {
                // Verify permissions based on federation type
                let required_permission = match federation_type {
                    FederationType::CooperativeFederation => "cooperative.federate",
                    FederationType::CommunityFederation => "community.federate",
                    FederationType::HybridFederation => "hybrid.federate",
                };
                
                ensure_permissions(&[required_permission.to_string()], &state.permissions)?;
                ensure_reputation(terms.min_reputation, state.reputation_context.get(&state.caller_did).copied().unwrap_or(0))?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_type".to_string(), format!("{:?}", federation_type));
                event_data.insert("partner_id".to_string(), partner_id.clone());
                event_data.insert("min_reputation".to_string(), terms.min_reputation.to_string());
                
                emit_event(state, "FederationInitiated".to_string(), event_data);
                Ok(())
            },

            FederationOperation::JoinFederation { federation_id, commitment } => {
                ensure_permissions(&["federation.join".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("member_did".to_string(), state.caller_did.clone());
                event_data.insert("commitment".to_string(), commitment.join(", "));
                
                emit_event(state, "FederationJoined".to_string(), event_data);
                Ok(())
            },

            FederationOperation::LeaveFederation { federation_id, reason } => {
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("member_did".to_string(), state.caller_did.clone());
                event_data.insert("reason".to_string(), reason.clone());
                
                emit_event(state, "FederationLeft".to_string(), event_data);
                Ok(())
            },

            FederationOperation::ProposeAction { federation_id, action_type, description, resources } => {
                ensure_permissions(&["federation.propose".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("action_type".to_string(), action_type.clone());
                event_data.insert("description".to_string(), description.clone());
                
                for (resource, amount) in resources {
                    event_data.insert(format!("resource_{}", resource), amount.to_string());
                }
                
                emit_event(state, "FederationActionProposed".to_string(), event_data);
                Ok(())
            },

            FederationOperation::VoteOnProposal { federation_id, proposal_id, approve, notes } => {
                ensure_permissions(&["federation.vote".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("approve".to_string(), approve.to_string());
                if let Some(note) = notes {
                    event_data.insert("notes".to_string(), note.clone());
                }
                
                emit_event(state, "FederationVoteCast".to_string(), event_data);
                Ok(())
            },

            FederationOperation::ShareResources { federation_id, resource_type, amount, recipient_id } => {
                ensure_permissions(&["federation.share_resources".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("resource_type".to_string(), resource_type.clone());
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("recipient_id".to_string(), recipient_id.clone());
                
                emit_event(state, "FederationResourceShared".to_string(), event_data);
                Ok(())
            },

            FederationOperation::SyncFederationState { federation_id } => {
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("sync_time".to_string(), state.timestamp.to_string());
                
                emit_event(state, "FederationStateSynced".to_string(), event_data);
                Ok(())
            },

            FederationOperation::UpdateFederationTerms { federation_id, new_terms } => {
                ensure_permissions(&["federation.update_terms".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("federation_type".to_string(), format!("{:?}", new_terms.federation_type));
                event_data.insert("min_reputation".to_string(), new_terms.min_reputation.to_string());
                
                emit_event(state, "FederationTermsUpdated".to_string(), event_data);
                Ok(())
            },

            FederationOperation::AddEndorsement { federation_id, target_id, endorsement_type, content } => {
                ensure_permissions(&["federation.endorse".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("target_id".to_string(), target_id.clone());
                event_data.insert("endorsement_type".to_string(), endorsement_type.clone());
                event_data.insert("content".to_string(), content.clone());
                
                emit_event(state, "FederationEndorsementAdded".to_string(), event_data);
                Ok(())
            },

            FederationOperation::ResolveDispute { federation_id, dispute_id, resolution, mediators } => {
                ensure_permissions(&["federation.mediate".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("dispute_id".to_string(), dispute_id.clone());
                event_data.insert("resolution".to_string(), resolution.clone());
                event_data.insert("mediators".to_string(), mediators.join(", "));
                
                emit_event(state, "FederationDisputeResolved".to_string(), event_data);
                Ok(())
            },

            FederationOperation::CoordinateAction { federation_id, action_type, participants, resources, timeline } => {
                ensure_permissions(&["federation.coordinate".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("action_type".to_string(), action_type.clone());
                event_data.insert("participants".to_string(), participants.join(", "));
                event_data.insert("timeline".to_string(), timeline.clone());
                
                for (resource, amount) in resources {
                    event_data.insert(format!("resource_{}", resource), amount.to_string());
                }
                
                emit_event(state, "FederationActionCoordinated".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            FederationOperation::InitiateFederation { .. } => 1000, // High cost for federation creation
            FederationOperation::JoinFederation { .. } => 500,
            FederationOperation::LeaveFederation { .. } => 200,
            FederationOperation::ProposeAction { .. } => 300,
            FederationOperation::VoteOnProposal { .. } => 100,
            FederationOperation::ShareResources { .. } => 400,
            FederationOperation::SyncFederationState { .. } => 200,
            FederationOperation::UpdateFederationTerms { .. } => 500,
            FederationOperation::AddEndorsement { .. } => 100,
            FederationOperation::ResolveDispute { .. } => 400,
            FederationOperation::CoordinateAction { .. } => 600,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            FederationOperation::InitiateFederation { federation_type, .. } => {
                match federation_type {
                    FederationType::CooperativeFederation => vec!["cooperative.federate".to_string()],
                    FederationType::CommunityFederation => vec!["community.federate".to_string()],
                    FederationType::HybridFederation => vec!["hybrid.federate".to_string()],
                }
            },
            FederationOperation::JoinFederation { .. } => vec!["federation.join".to_string()],
            FederationOperation::ProposeAction { .. } => vec!["federation.propose".to_string()],
            FederationOperation::VoteOnProposal { .. } => vec!["federation.vote".to_string()],
            FederationOperation::ShareResources { .. } => vec!["federation.share_resources".to_string()],
            FederationOperation::UpdateFederationTerms { .. } => vec!["federation.update_terms".to_string()],
            FederationOperation::AddEndorsement { .. } => vec!["federation.endorse".to_string()],
            FederationOperation::ResolveDispute { .. } => vec!["federation.mediate".to_string()],
            FederationOperation::CoordinateAction { .. } => vec!["federation.coordinate".to_string()],
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
                "cooperative.federate".to_string(),
                "community.federate".to_string(),
                "federation.join".to_string(),
                "federation.propose".to_string(),
                "federation.vote".to_string(),
                "federation.share_resources".to_string(),
            ],
        };
        
        state.reputation_context.insert(state.caller_did.clone(), 1000);
        state
    }

    #[test]
    fn test_initiate_cooperative_federation() {
        let mut state = setup_test_state();
        let terms = FederationTerms {
            federation_type: FederationType::CooperativeFederation,
            min_reputation: 500,
            resource_policies: vec!["policy1".to_string()],
            governance_rules: vec!["rule1".to_string()],
            duration_days: 365,
        };
        
        let op = FederationOperation::InitiateFederation {
            federation_type: FederationType::CooperativeFederation,
            partner_id: "partner_coop".to_string(),
            terms,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationInitiated");
    }

    #[test]
    fn test_join_federation() {
        let mut state = setup_test_state();
        let op = FederationOperation::JoinFederation {
            federation_id: "fed1".to_string(),
            commitment: vec!["commit1".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationJoined");
    }

    #[test]
    fn test_share_resources() {
        let mut state = setup_test_state();
        let op = FederationOperation::ShareResources {
            federation_id: "fed1".to_string(),
            resource_type: "computing".to_string(),
            amount: 1000,
            recipient_id: "recipient_coop".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationResourceShared");
    }

    #[test]
    fn test_resolve_dispute() {
        let mut state = setup_test_state();
        state.permissions.push("federation.mediate".to_string());
        
        let op = FederationOperation::ResolveDispute {
            federation_id: "fed1".to_string(),
            dispute_id: "dispute1".to_string(),
            resolution: "Mutual agreement reached".to_string(),
            mediators: vec!["mediator1".to_string(), "mediator2".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationDisputeResolved");
    }

    #[test]
    fn test_coordinate_action() {
        let mut state = setup_test_state();
        state.permissions.push("federation.coordinate".to_string());
        
        let mut resources = HashMap::new();
        resources.insert("computing".to_string(), 500);
        resources.insert("storage".to_string(), 1000);
        
        let op = FederationOperation::CoordinateAction {
            federation_id: "fed1".to_string(),
            action_type: "joint_project".to_string(),
            participants: vec!["coop1".to_string(), "coop2".to_string()],
            resources,
            timeline: "2024-Q2".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationActionCoordinated");
    }

    #[test]
    fn test_insufficient_reputation() {
        let mut state = setup_test_state();
        state.reputation_context.insert(state.caller_did.clone(), 100); // Set low reputation
        
        let terms = FederationTerms {
            federation_type: FederationType::CooperativeFederation,
            min_reputation: 500,
            resource_policies: vec![],
            governance_rules: vec![],
            duration_days: 365,
        };
        
        let op = FederationOperation::InitiateFederation {
            federation_type: FederationType::CooperativeFederation,
            partner_id: "partner_coop".to_string(),
            terms,
        };
        
        assert!(matches!(op.execute(&mut state), Err(VMError::InsufficientReputation)));
    }

    #[test]
    fn test_hybrid_federation() {
        let mut state = setup_test_state();
        state.permissions.push("hybrid.federate".to_string());
        
        let terms = FederationTerms {
            federation_type: FederationType::HybridFederation,
            min_reputation: 500,
            resource_policies: vec!["policy1".to_string()],
            governance_rules: vec!["rule1".to_string()],
            duration_days: 365,
        };
        
        let op = FederationOperation::InitiateFederation {
            federation_type: FederationType::HybridFederation,
            partner_id: "community1".to_string(),
            terms,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "FederationInitiated");
    }
}