use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::vm::VMError;

/// System-level operations for the VM
pub enum SystemOperation {
    /// Log a message to system logs
    Log {
        message: String,
        level: LogLevel,
        metadata: HashMap<String, String>,
    },

    /// Halt VM execution
    Halt,

    /// Emit a custom event
    EmitEvent {
        event_type: String,
        data: HashMap<String, String>,
    },

    /// Get current block number
    GetBlockNumber,

    /// Get current timestamp
    GetTimestamp,

    /// Get caller's DID
    GetCaller,

    /// Record energy metrics
    RecordEnergyMetrics {
        operation_type: String,
        energy_used: u64,
        duration_ms: u64,
    },

    /// Get system statistics
    GetSystemStats {
        stat_types: Vec<StatType>,
    },

    /// Check system health
    CheckHealth {
        components: Vec<String>,
    },

    /// Update system parameters
    UpdateParameter {
        parameter: SystemParameter,
        value: String,
    },
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub enum StatType {
    MemoryUsage,
    CpuLoad,
    NetworkActivity,
    StorageUsage,
    TransactionCount,
    ActiveNodes,
    EnergyMetrics,
}

#[derive(Debug, Clone)]
pub enum SystemParameter {
    MaxBlockSize,
    MaxTransactionsPerBlock,
    MinimumResourceCost,
    ReputationDecayRate,
    NetworkTimeout,
    ConsensusThreshold,
}

impl Operation for SystemOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            SystemOperation::Log { message, level, metadata } => {
                let mut event_data = HashMap::new();
                event_data.insert("message".to_string(), message.clone());
                event_data.insert("level".to_string(), format!("{:?}", level));
                
                for (key, value) in metadata {
                    event_data.insert(format!("metadata_{}", key), value.clone());
                }
                
                emit_event(state, "SystemLog".to_string(), event_data);
                Ok(())
            },

            SystemOperation::Halt => {
                emit_event(
                    state,
                    "SystemHalt".to_string(),
                    HashMap::new(),
                );
                Err(VMError::Custom("Execution halted".to_string()))
            },

            SystemOperation::EmitEvent { event_type, data } => {
                emit_event(state, event_type.clone(), data.clone());
                Ok(())
            },

            SystemOperation::GetBlockNumber => {
                state.stack.push(state.block_number as i64);
                Ok(())
            },

            SystemOperation::GetTimestamp => {
                state.stack.push(state.timestamp as i64);
                Ok(())
            },

            SystemOperation::GetCaller => {
                let caller_hash = state.caller_did.as_bytes().iter()
                    .fold(0i64, |acc, &b| acc + b as i64);
                state.stack.push(caller_hash);
                Ok(())
            },

            SystemOperation::RecordEnergyMetrics { operation_type, energy_used, duration_ms } => {
                ensure_permissions(&["system.metrics".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("operation_type".to_string(), operation_type.clone());
                event_data.insert("energy_used".to_string(), energy_used.to_string());
                event_data.insert("duration_ms".to_string(), duration_ms.to_string());
                
                emit_event(state, "EnergyMetricsRecorded".to_string(), event_data);
                Ok(())
            },

            SystemOperation::GetSystemStats { stat_types } => {
                ensure_permissions(&["system.stats".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("requested_stats".to_string(), 
                    stat_types.iter()
                        .map(|st| format!("{:?}", st))
                        .collect::<Vec<String>>()
                        .join(",")
                );
                
                // In a real implementation, we would collect actual system stats here
                for stat_type in stat_types {
                    match stat_type {
                        StatType::MemoryUsage => event_data.insert("memory_usage".to_string(), "1000".to_string()),
                        StatType::CpuLoad => event_data.insert("cpu_load".to_string(), "50".to_string()),
                        StatType::NetworkActivity => event_data.insert("network_activity".to_string(), "100".to_string()),
                        StatType::StorageUsage => event_data.insert("storage_usage".to_string(), "5000".to_string()),
                        StatType::TransactionCount => event_data.insert("transaction_count".to_string(), "1000".to_string()),
                        StatType::ActiveNodes => event_data.insert("active_nodes".to_string(), "10".to_string()),
                        StatType::EnergyMetrics => event_data.insert("energy_metrics".to_string(), "500".to_string()),
                    };
                }
                
                emit_event(state, "SystemStatsQueried".to_string(), event_data);
                Ok(())
            },

            SystemOperation::CheckHealth { components } => {
                ensure_permissions(&["system.health".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("components".to_string(), components.join(","));
                
                // In a real implementation, we would do actual health checks here
                event_data.insert("status".to_string(), "healthy".to_string());
                
                emit_event(state, "HealthCheckPerformed".to_string(), event_data);
                Ok(())
            },

            SystemOperation::UpdateParameter { parameter, value } => {
                ensure_permissions(&["system.admin".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("parameter".to_string(), format!("{:?}", parameter));
                event_data.insert("value".to_string(), value.clone());
                
                emit_event(state, "SystemParameterUpdated".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            SystemOperation::Log { .. } => 5,
            SystemOperation::Halt => 1,
            SystemOperation::EmitEvent { .. } => 10,
            SystemOperation::GetBlockNumber => 1,
            SystemOperation::GetTimestamp => 1,
            SystemOperation::GetCaller => 1,
            SystemOperation::RecordEnergyMetrics { .. } => 20,
            SystemOperation::GetSystemStats { stat_types } => 10 * (stat_types.len() as u64),
            SystemOperation::CheckHealth { components } => 15 * (components.len() as u64),
            SystemOperation::UpdateParameter { .. } => 50,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            SystemOperation::RecordEnergyMetrics { .. } => vec!["system.metrics".to_string()],
            SystemOperation::GetSystemStats { .. } => vec!["system.stats".to_string()],
            SystemOperation::CheckHealth { .. } => vec!["system.health".to_string()],
            SystemOperation::UpdateParameter { .. } => vec!["system.admin".to_string()],
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
                "system.metrics".to_string(),
                "system.stats".to_string(),
                "system.health".to_string(),
                "system.admin".to_string(),
            ],
        };
        state.reputation_context.insert(state.caller_did.clone(), 100);
        state
    }

    #[test]
    fn test_system_log() {
        let mut state = setup_test_state();
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        
        let op = SystemOperation::Log {
            message: "Test log".to_string(),
            level: LogLevel::Info,
            metadata,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "SystemLog");
    }

    #[test]
    fn test_emit_event() {
        let mut state = setup_test_state();
        let mut data = HashMap::new();
        data.insert("test_key".to_string(), "test_value".to_string());
        
        let op = SystemOperation::EmitEvent {
            event_type: "TestEvent".to_string(),
            data,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "TestEvent");
    }

    #[test]
    fn test_get_system_stats() {
        let mut state = setup_test_state();
        let op = SystemOperation::GetSystemStats {
            stat_types: vec![StatType::MemoryUsage, StatType::CpuLoad],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "SystemStatsQueried");
    }

    #[test]
    fn test_update_parameter() {
        let mut state = setup_test_state();
        let op = SystemOperation::UpdateParameter {
            parameter: SystemParameter::MaxBlockSize,
            value: "1000".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "SystemParameterUpdated");
    }

    #[test]
    fn test_halt() {
        let mut state = setup_test_state();
        let op = SystemOperation::Halt;
        
        assert!(matches!(op.execute(&mut state), Err(VMError::Custom(_))));
    }
}
