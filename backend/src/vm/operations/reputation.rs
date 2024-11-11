// src/vm/operations/reputation.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_stack_size, ensure_permissions, emit_event};
use crate::vm::VMError;

/// Operations for managing reputation within the system
pub enum ReputationOperation {
    /// Update a member's reputation by a specific amount
    UpdateReputation {
        amount: i64,
        reason: String,
        context: String,
    },
    
    /// Get the current reputation of an account
    GetReputation {
        target_did: String,
    },
    
    /// Burn (remove) reputation from an account
    BurnReputation {
        amount: i64,
        reason: String,
    },
    
    /// Mint new reputation (requires special permissions)
    MintReputation {
        target_did: String,
        amount: i64,
        reason: String,
    },
    
    /// Record contribution impact on reputation
    RecordContributionImpact {
        contribution_id: String,
        impact_score: i64,
        context: String,
    },
    
    /// Calculate voting power based on reputation
    CalculateVotingPower {
        context: String,
    },
    
    /// Decay reputation over time
    ApplyReputationDecay {
        decay_rate: f64,
        minimum_reputation: i64,
    },
    
    /// Get reputation history for an account
    GetReputationHistory {
        target_did: String,
        context: Option<String>,
    },
}

impl Operation for ReputationOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            ReputationOperation::UpdateReputation { amount, reason, context } => {
                // Check if caller has permission to update reputation
                ensure_permissions(&["reputation.update".to_string()], &state.permissions)?;
                
                // Get current reputation
                let current_rep = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                
                // Update reputation
                let new_rep = current_rep + amount;
                state.reputation_context.insert(state.caller_did.clone(), new_rep);
                
                // Emit reputation update event
                let mut event_data = HashMap::new();
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("reason".to_string(), reason.clone());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("new_total".to_string(), new_rep.to_string());
                
                emit_event(state, "ReputationUpdated".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::GetReputation { target_did } => {
                let reputation = state.reputation_context
                    .get(target_did)
                    .copied()
                    .unwrap_or(0);
                    
                state.stack.push(reputation);
                Ok(())
            },
            
            ReputationOperation::BurnReputation { amount, reason } => {
                ensure_permissions(&["reputation.burn".to_string()], &state.permissions)?;
                
                let current_rep = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                    
                if current_rep < *amount {
                    return Err(VMError::InsufficientReputation);
                }
                
                let new_rep = current_rep - amount;
                state.reputation_context.insert(state.caller_did.clone(), new_rep);
                
                let mut event_data = HashMap::new();
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("reason".to_string(), reason.clone());
                event_data.insert("new_total".to_string(), new_rep.to_string());
                
                emit_event(state, "ReputationBurned".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::MintReputation { target_did, amount, reason } => {
                ensure_permissions(&["reputation.mint".to_string()], &state.permissions)?;
                
                let current_rep = state.reputation_context
                    .get(target_did)
                    .copied()
                    .unwrap_or(0);
                    
                let new_rep = current_rep + amount;
                state.reputation_context.insert(target_did.clone(), new_rep);
                
                let mut event_data = HashMap::new();
                event_data.insert("target_did".to_string(), target_did.clone());
                event_data.insert("amount".to_string(), amount.to_string());
                event_data.insert("reason".to_string(), reason.clone());
                event_data.insert("new_total".to_string(), new_rep.to_string());
                
                emit_event(state, "ReputationMinted".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::RecordContributionImpact { contribution_id, impact_score, context } => {
                ensure_permissions(&["contribution.record".to_string()], &state.permissions)?;
                
                // Calculate reputation change based on impact score
                let reputation_change = impact_score.max(-50).min(50); // Limit impact
                
                let current_rep = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                    
                let new_rep = current_rep + reputation_change;
                state.reputation_context.insert(state.caller_did.clone(), new_rep);
                
                let mut event_data = HashMap::new();
                event_data.insert("contribution_id".to_string(), contribution_id.clone());
                event_data.insert("impact_score".to_string(), impact_score.to_string());
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("reputation_change".to_string(), reputation_change.to_string());
                
                emit_event(state, "ContributionImpactRecorded".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::CalculateVotingPower { context } => {
                let reputation = state.reputation_context
                    .get(&state.caller_did)
                    .copied()
                    .unwrap_or(0);
                    
                // Calculate voting power as a function of reputation
                // Square root function to prevent excessive concentration of power
                let voting_power = (reputation as f64).sqrt() as i64;
                
                state.stack.push(voting_power);
                
                let mut event_data = HashMap::new();
                event_data.insert("context".to_string(), context.clone());
                event_data.insert("reputation".to_string(), reputation.to_string());
                event_data.insert("voting_power".to_string(), voting_power.to_string());
                
                emit_event(state, "VotingPowerCalculated".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::ApplyReputationDecay { decay_rate, minimum_reputation } => {
                ensure_permissions(&["reputation.decay".to_string()], &state.permissions)?;
                
                let mut decayed_accounts = Vec::new();
                
                for (did, rep) in state.reputation_context.iter_mut() {
                    let old_rep = *rep;
                    let new_rep = ((*rep as f64) * (1.0 - decay_rate)) as i64;
                    *rep = new_rep.max(*minimum_reputation);
                    
                    if old_rep != *rep {
                        decayed_accounts.push((did.clone(), old_rep, *rep));
                    }
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("decay_rate".to_string(), decay_rate.to_string());
                event_data.insert("accounts_affected".to_string(), decayed_accounts.len().to_string());
                
                emit_event(state, "ReputationDecayApplied".to_string(), event_data);
                Ok(())
            },
            
            ReputationOperation::GetReputationHistory { target_did, context: _ } => {
                // In a real implementation, this would query historical reputation data
                // For now, we just return the current reputation
                let reputation = state.reputation_context
                    .get(target_did)
                    .copied()
                    .unwrap_or(0);
                    
                state.stack.push(reputation);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            ReputationOperation::UpdateReputation { .. } => 5,
            ReputationOperation::GetReputation { .. } => 1,
            ReputationOperation::BurnReputation { .. } => 5,
            ReputationOperation::MintReputation { .. } => 10,
            ReputationOperation::RecordContributionImpact { .. } => 7,
            ReputationOperation::CalculateVotingPower { .. } => 3,
            ReputationOperation::ApplyReputationDecay { .. } => 15,
            ReputationOperation::GetReputationHistory { .. } => 2,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            ReputationOperation::UpdateReputation { .. } => vec!["reputation.update".to_string()],
            ReputationOperation::BurnReputation { .. } => vec!["reputation.burn".to_string()],
            ReputationOperation::MintReputation { .. } => vec!["reputation.mint".to_string()],
            ReputationOperation::RecordContributionImpact { .. } => vec!["contribution.record".to_string()],
            ReputationOperation::ApplyReputationDecay { .. } => vec!["reputation.decay".to_string()],
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
                "reputation.update".to_string(),
                "reputation.mint".to_string(),
                "reputation.burn".to_string(),
                "reputation.decay".to_string(),
            ],
        };
        
        // Set initial reputation
        state.reputation_context.insert(state.caller_did.clone(), 100);
        state
    }

    #[test]
    fn test_update_reputation() {
        let mut state = setup_test_state();
        let op = ReputationOperation::UpdateReputation {
            amount: 50,
            reason: "test".to_string(),
            context: "testing".to_string(),
        };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(
            state.reputation_context.get(&state.caller_did).copied().unwrap(),
            150
        );
    }

    #[test]
    fn test_burn_reputation() {
        let mut state = setup_test_state();
        let op = ReputationOperation::BurnReputation {
            amount: 30,
            reason: "test".to_string(),
        };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(
            state.reputation_context.get(&state.caller_did).copied().unwrap(),
            70
        );
    }

    #[test]
    fn test_calculate_voting_power() {
        let mut state = setup_test_state();
        let op = ReputationOperation::CalculateVotingPower {
            context: "voting".to_string(),
        };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack.pop().unwrap(), 10); // sqrt(100) = 10
    }

    #[test]
    fn test_reputation_decay() {
        let mut state = setup_test_state();
        let op = ReputationOperation::ApplyReputationDecay {
            decay_rate: 0.1,
            minimum_reputation: 0,
        };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(
            state.reputation_context.get(&state.caller_did).copied().unwrap(),
            90  // 100 * (1 - 0.1) = 90
        );
    }
}