// src/vm/operations/governance.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::vm::VMError;

/// Operations for governance functionality
pub enum GovernanceOperation {
    /// Create a new proposal
    CreateProposal {
        title: String,
        description: String,
        proposal_type: ProposalType,
        duration: u64,
        required_reputation: i64,
    },

    /// Cast a vote on a proposal
    CastVote {
        proposal_id: String,
        approve: bool,
        comment: Option<String>,
    },

    /// Delegate voting power to another member
    DelegateVotes {
        delegate_to: String,
        scope: DelegationScope,
        duration: Option<u64>,
    },

    /// Execute an approved proposal
    ExecuteProposal {
        proposal_id: String,
    },

    /// Cancel a proposal (only by creator or governance admin)
    CancelProposal {
        proposal_id: String,
        reason: String,
    },

    /// Update quorum requirements
    UpdateQuorum {
        new_quorum: f64,
        proposal_type: Option<ProposalType>,
    },

    /// Extend voting period for a proposal
    ExtendVotingPeriod {
        proposal_id: String,
        additional_time: u64,
    },

    /// Get proposal details
    GetProposalDetails {
        proposal_id: String,
    },

    /// Get voting statistics
    GetVotingStats {
        proposal_id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalType {
    ResourceAllocation,
    PolicyChange,
    MembershipDecision,
    FederationAction,
    TechnicalChange,
    ReputationAdjustment,
    EmergencyAction,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum DelegationScope {
    All,
    ProposalType(ProposalType),
    SpecificProposal(String),
    Domain(String),
}

impl Operation for GovernanceOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            GovernanceOperation::CreateProposal {
                title,
                description,
                proposal_type,
                duration,
                required_reputation,
            } => {
                ensure_permissions(&["governance.create_proposal".to_string()], &state.permissions)?;

                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);

                if reputation < *required_reputation {
                    return Err(VMError::InsufficientReputation);
                }

                let mut event_data = HashMap::new();
                event_data.insert("title".to_string(), title.clone());
                event_data.insert("description".to_string(), description.clone());
                event_data.insert("proposal_type".to_string(), format!("{:?}", proposal_type));
                event_data.insert("duration".to_string(), duration.to_string());
                event_data.insert("required_reputation".to_string(), required_reputation.to_string());

                emit_event(state, "ProposalCreated".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::CastVote { proposal_id, approve, comment } => {
                ensure_permissions(&["governance.vote".to_string()], &state.permissions)?;

                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);

                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("approve".to_string(), approve.to_string());
                event_data.insert("voter_reputation".to_string(), reputation.to_string());
                
                if let Some(comment) = comment {
                    event_data.insert("comment".to_string(), comment.clone());
                }

                emit_event(state, "VoteCast".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::DelegateVotes { delegate_to, scope, duration } => {
                ensure_permissions(&["governance.delegate".to_string()], &state.permissions)?;

                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);

                if reputation < 100 {  // Minimum reputation required for delegation
                    return Err(VMError::InsufficientReputation);
                }

                let mut event_data = HashMap::new();
                event_data.insert("delegate_to".to_string(), delegate_to.clone());
                event_data.insert("scope".to_string(), format!("{:?}", scope));
                
                if let Some(dur) = duration {
                    event_data.insert("duration".to_string(), dur.to_string());
                }

                emit_event(state, "VotesDelegated".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::ExecuteProposal { proposal_id } => {
                ensure_permissions(&["governance.execute".to_string()], &state.permissions)?;

                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("executor".to_string(), state.caller_did.clone());

                emit_event(state, "ProposalExecuted".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::CancelProposal { proposal_id, reason } => {
                ensure_permissions(&["governance.cancel".to_string()], &state.permissions)?;

                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("reason".to_string(), reason.clone());
                event_data.insert("cancelled_by".to_string(), state.caller_did.clone());

                emit_event(state, "ProposalCancelled".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::UpdateQuorum { new_quorum, proposal_type } => {
                ensure_permissions(&["governance.update_quorum".to_string()], &state.permissions)?;

                if *new_quorum <= 0.0 || *new_quorum > 1.0 {
                    return Err(VMError::Custom("Invalid quorum value".to_string()));
                }

                let mut event_data = HashMap::new();
                event_data.insert("new_quorum".to_string(), new_quorum.to_string());
                
                if let Some(pt) = proposal_type {
                    event_data.insert("proposal_type".to_string(), format!("{:?}", pt));
                }

                emit_event(state, "QuorumUpdated".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::ExtendVotingPeriod { proposal_id, additional_time } => {
                ensure_permissions(&["governance.extend_voting".to_string()], &state.permissions)?;

                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("additional_time".to_string(), additional_time.to_string());

                emit_event(state, "VotingPeriodExtended".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::GetProposalDetails { proposal_id } => {
                // In a real implementation, this would query the proposal storage
                // For now, we just emit an event for tracking
                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("queried_by".to_string(), state.caller_did.clone());

                emit_event(state, "ProposalDetailsQueried".to_string(), event_data);
                Ok(())
            }

            GovernanceOperation::GetVotingStats { proposal_id } => {
                // Similarly, this would query voting statistics in a real implementation
                let mut event_data = HashMap::new();
                event_data.insert("proposal_id".to_string(), proposal_id.clone());
                event_data.insert("queried_by".to_string(), state.caller_did.clone());

                emit_event(state, "VotingStatsQueried".to_string(), event_data);
                Ok(())
            }
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            GovernanceOperation::CreateProposal { .. } => 100,
            GovernanceOperation::CastVote { .. } => 20,
            GovernanceOperation::DelegateVotes { .. } => 50,
            GovernanceOperation::ExecuteProposal { .. } => 200,
            GovernanceOperation::CancelProposal { .. } => 75,
            GovernanceOperation::UpdateQuorum { .. } => 150,
            GovernanceOperation::ExtendVotingPeriod { .. } => 30,
            GovernanceOperation::GetProposalDetails { .. } => 5,
            GovernanceOperation::GetVotingStats { .. } => 5,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            GovernanceOperation::CreateProposal { .. } => vec!["governance.create_proposal".to_string()],
            GovernanceOperation::CastVote { .. } => vec!["governance.vote".to_string()],
            GovernanceOperation::DelegateVotes { .. } => vec!["governance.delegate".to_string()],
            GovernanceOperation::ExecuteProposal { .. } => vec!["governance.execute".to_string()],
            GovernanceOperation::CancelProposal { .. } => vec!["governance.cancel".to_string()],
            GovernanceOperation::UpdateQuorum { .. } => vec!["governance.update_quorum".to_string()],
            GovernanceOperation::ExtendVotingPeriod { .. } => vec!["governance.extend_voting".to_string()],
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
                "governance.create_proposal".to_string(),
                "governance.vote".to_string(),
                "governance.execute".to_string(),
            ],
        };
        
        state.reputation_context.insert(state.caller_did.clone(), 200);
        state
    }

    #[test]
    fn test_create_proposal() {
        let mut state = setup_test_state();
        let op = GovernanceOperation::CreateProposal {
            title: "Test Proposal".to_string(),
            description: "Test Description".to_string(),
            proposal_type: ProposalType::PolicyChange,
            duration: 86400,
            required_reputation: 100,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].event_type, "ProposalCreated");
    }

    #[test]
    fn test_cast_vote() {
        let mut state = setup_test_state();
        let op = GovernanceOperation::CastVote {
            proposal_id: "test_proposal".to_string(),
            approve: true,
            comment: Some("Support this proposal".to_string()),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].event_type, "VoteCast");
    }

    #[test]
    fn test_insufficient_reputation() {
        let mut state = setup_test_state();
        state.reputation_context.insert(state.caller_did.clone(), 50);
        
        let op = GovernanceOperation::CreateProposal {
            title: "Test Proposal".to_string(),
            description: "Test Description".to_string(),
            proposal_type: ProposalType::PolicyChange,
            duration: 86400,
            required_reputation: 100,
        };
        
        assert!(matches!(op.execute(&mut state), Err(VMError::InsufficientReputation)));
    }

    #[test]
    fn test_update_quorum() {
        let mut state = setup_test_state();
        state.permissions.push("governance.update_quorum".to_string());
        
        let op = GovernanceOperation::UpdateQuorum {
            new_quorum: 0.75,
            proposal_type: Some(ProposalType::PolicyChange),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].event_type, "QuorumUpdated");
    }
}