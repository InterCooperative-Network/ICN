// src/vm/operations/network.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::vm::VMError;

/// Types of network messages that can be sent
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    /// Direct message to another node
    Direct,
    /// Broadcast to all federation members
    FederationBroadcast,
    /// Broadcast to all community members
    CommunityBroadcast,
    /// Network state synchronization
    StateSync,
    /// Resource discovery query
    ResourceDiscovery,
    /// Network health check
    HealthCheck,
}

/// Network message priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Network message content
#[derive(Debug, Clone)]
pub struct NetworkMessage {
    /// Type of message
    message_type: MessageType,
    /// Message priority
    priority: MessagePriority,
    /// Target DID (for direct messages)
    target: Option<String>,
    /// Message content
    content: String,
    /// Additional metadata
    metadata: HashMap<String, String>,
}

/// Operations for managing network communication
pub enum NetworkOperation {
    /// Send a message to another node
    SendMessage {
        message: NetworkMessage,
    },
    
    /// Subscribe to network topics
    Subscribe {
        topics: Vec<String>,
        filter: Option<String>,
    },
    
    /// Unsubscribe from network topics
    Unsubscribe {
        topics: Vec<String>,
    },
    
    /// Discover available resources on the network
    DiscoverResources {
        resource_type: String,
        max_results: u32,
    },
    
    /// Sync state with federation members
    SyncState {
        federation_id: String,
        state_type: String,
    },
    
    /// Join a network partition
    JoinPartition {
        partition_id: String,
        capabilities: Vec<String>,
    },
    
    /// Leave a network partition
    LeavePartition {
        partition_id: String,
    },
    
    /// Broadcast node capabilities
    BroadcastCapabilities {
        capabilities: Vec<String>,
    },
    
    /// Check network connection health
    HealthCheck {
        target: Option<String>,
        timeout_ms: u64,
    },
    
    /// Configure network settings
    ConfigureNetwork {
        settings: HashMap<String, String>,
    },
    
    /// Request network route to target
    RequestRoute {
        target: String,
        max_hops: u32,
    },
    
    /// Update network topology
    UpdateTopology {
        connections: Vec<(String, String)>,
    },
    
    /// Establish secure channel
    EstablishChannel {
        target: String,
        encryption_params: HashMap<String, String>,
    },
}

impl Operation for NetworkOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            NetworkOperation::SendMessage { message } => {
                let permission = match message.message_type {
                    MessageType::Direct => "network.send_direct",
                    MessageType::FederationBroadcast => "network.broadcast_federation",
                    MessageType::CommunityBroadcast => "network.broadcast_community",
                    MessageType::StateSync => "network.sync_state",
                    MessageType::ResourceDiscovery => "network.discover",
                    MessageType::HealthCheck => "network.health_check",
                };
                
                ensure_permissions(&[permission.to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("message_type".to_string(), format!("{:?}", message.message_type));
                event_data.insert("priority".to_string(), format!("{:?}", message.priority));
                if let Some(target) = &message.target {
                    event_data.insert("target".to_string(), target.clone());
                }
                event_data.insert("content_length".to_string(), message.content.len().to_string());
                
                emit_event(state, "MessageSent".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::Subscribe { topics, filter } => {
                ensure_permissions(&["network.subscribe".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("topics".to_string(), topics.join(","));
                if let Some(f) = filter {
                    event_data.insert("filter".to_string(), f.clone());
                }
                
                emit_event(state, "TopicsSubscribed".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::Unsubscribe { topics } => {
                let mut event_data = HashMap::new();
                event_data.insert("topics".to_string(), topics.join(","));
                
                emit_event(state, "TopicsUnsubscribed".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::DiscoverResources { resource_type, max_results } => {
                ensure_permissions(&["network.discover".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("resource_type".to_string(), resource_type.clone());
                event_data.insert("max_results".to_string(), max_results.to_string());
                
                emit_event(state, "ResourceDiscoveryInitiated".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::SyncState { federation_id, state_type } => {
                ensure_permissions(&["network.sync_state".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("state_type".to_string(), state_type.clone());
                
                emit_event(state, "StateSyncInitiated".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::JoinPartition { partition_id, capabilities } => {
                ensure_permissions(&["network.partition".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("partition_id".to_string(), partition_id.clone());
                event_data.insert("capabilities".to_string(), capabilities.join(","));
                
                emit_event(state, "PartitionJoined".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::LeavePartition { partition_id } => {
                ensure_permissions(&["network.partition".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("partition_id".to_string(), partition_id.clone());
                
                emit_event(state, "PartitionLeft".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::BroadcastCapabilities { capabilities } => {
                ensure_permissions(&["network.broadcast".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("capabilities".to_string(), capabilities.join(","));
                
                emit_event(state, "CapabilitiesBroadcast".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::HealthCheck { target, timeout_ms } => {
                let mut event_data = HashMap::new();
                if let Some(t) = target {
                    event_data.insert("target".to_string(), t.clone());
                }
                event_data.insert("timeout_ms".to_string(), timeout_ms.to_string());
                
                emit_event(state, "HealthCheckInitiated".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::ConfigureNetwork { settings } => {
                ensure_permissions(&["network.configure".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                for (key, value) in settings {
                    event_data.insert(key.clone(), value.clone());
                }
                
                emit_event(state, "NetworkConfigured".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::RequestRoute { target, max_hops } => {
                let mut event_data = HashMap::new();
                event_data.insert("target".to_string(), target.clone());
                event_data.insert("max_hops".to_string(), max_hops.to_string());
                
                emit_event(state, "RouteRequested".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::UpdateTopology { connections } => {
                ensure_permissions(&["network.topology".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("connection_count".to_string(), connections.len().to_string());
                
                emit_event(state, "TopologyUpdated".to_string(), event_data);
                Ok(())
            },

            NetworkOperation::EstablishChannel { target, encryption_params } => {
                ensure_permissions(&["network.secure_channel".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("target".to_string(), target.clone());
                for (key, value) in encryption_params {
                    event_data.insert(format!("param_{}", key), value.clone());
                }
                
                emit_event(state, "SecureChannelEstablished".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            NetworkOperation::SendMessage { message } => {
                let base_cost = match message.priority {
                    MessagePriority::Low => 10,
                    MessagePriority::Normal => 20,
                    MessagePriority::High => 40,
                    MessagePriority::Critical => 80,
                };
                base_cost + (message.content.len() as u64 / 100)
            },
            NetworkOperation::Subscribe { topics, .. } => 20 + (topics.len() as u64 * 5),
            NetworkOperation::Unsubscribe { topics } => 10 + (topics.len() as u64 * 2),
            NetworkOperation::DiscoverResources { .. } => 50,
            NetworkOperation::SyncState { .. } => 100,
            NetworkOperation::JoinPartition { .. } => 30,
            NetworkOperation::LeavePartition { .. } => 20,
            NetworkOperation::BroadcastCapabilities { capabilities } => 20 + (capabilities.len() as u64 * 5),
            NetworkOperation::HealthCheck { .. } => 10,
            NetworkOperation::ConfigureNetwork { settings } => 30 + (settings.len() as u64 * 5),
            NetworkOperation::RequestRoute { .. } => 25,
            NetworkOperation::UpdateTopology { connections } => 40 + (connections.len() as u64 * 5),
            NetworkOperation::EstablishChannel { .. } => 60,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            NetworkOperation::SendMessage { message } => {
                match message.message_type {
                    MessageType::Direct => vec!["network.send_direct".to_string()],
                    MessageType::FederationBroadcast => vec!["network.broadcast_federation".to_string()],
                    MessageType::CommunityBroadcast => vec!["network.broadcast_community".to_string()],
                    MessageType::StateSync => vec!["network.sync_state".to_string()],
                    MessageType::ResourceDiscovery => vec!["network.discover".to_string()],
                    MessageType::HealthCheck => vec!["network.health_check".to_string()],
                }
            },
            NetworkOperation::Subscribe { .. } => vec!["network.subscribe".to_string()],
            NetworkOperation::DiscoverResources { .. } => vec!["network.discover".to_string()],
            NetworkOperation::SyncState { .. } => vec!["network.sync_state".to_string()],
            NetworkOperation::JoinPartition { .. } | NetworkOperation::LeavePartition { .. } => 
                vec!["network.partition".to_string()],
            NetworkOperation::BroadcastCapabilities { .. } => vec!["network.broadcast".to_string()],
            NetworkOperation::ConfigureNetwork { .. } => vec!["network.configure".to_string()],
            NetworkOperation::UpdateTopology { .. } => vec!["network.topology".to_string()],
            NetworkOperation::EstablishChannel { .. } => vec!["network.secure_channel".to_string()],
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
                "network.send_direct".to_string(),
                "network.broadcast_federation".to_string(),
                "network.subscribe".to_string(),
                "network.discover".to_string(),
                "network.sync_state".to_string(),
            ],
        };
        
        state.reputation_context.insert(state.caller_did.clone(), 100);
        state
    }

    #[test]
    fn test_send_direct_message() {
        let mut state = setup_test_state();
        let message = NetworkMessage {
            message_type: MessageType::Direct,
            priority: MessagePriority::Normal,
            target: Some("recipient".to_string()),
            content: "Hello".to_string(),
            metadata: HashMap::new(),
        };
        
        let op = NetworkOperation::SendMessage { message };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MessageSent");
    }

    #[test]
    fn test_federation_broadcast() {
        let mut state = setup_test_state();
        let message = NetworkMessage {
            message_type: MessageType::FederationBroadcast,
            priority: MessagePriority::High,
            target: None,
            content: "Federation update".to_string(),
            metadata: HashMap::new(),
        };
        
        let op = NetworkOperation::SendMessage { message };
        assert!(op.execute(&mut state).is_ok());
    }

    #[test]
    fn test_subscribe_topics() {
        let mut state = setup_test_state();
        let op = NetworkOperation::Subscribe {
            topics: vec!["governance".to_string(), "resources".to_string()],
            filter: Some("type = 'update'".to_string()),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "TopicsSubscribed");
    }

    #[test]
    fn test_sync_state() {
        let mut state = setup_test_state();
        let op = NetworkOperation::SyncState {
            federation_id: "fed1".to_string(),
            state_type: "resources".to_string(),
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "StateSyncInitiated");
    }

    #[test]
    fn test_secure_channel() {
        let mut state = setup_test_state();
        state.permissions.push("network.secure_channel".to_string());
        
        let mut encryption_params = HashMap::new();
        encryption_params.insert("algorithm".to_string(), "AES-256-GCM".to_string());
        encryption_params.insert("key_exchange".to_string(), "ECDH".to_string());
        
        let op = NetworkOperation::EstablishChannel {
            target: "peer1".to_string(),
            encryption_params,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "SecureChannelEstablished");
    }

    #[test]
    fn test_resource_discovery() {
        let mut state = setup_test_state();
        let op = NetworkOperation::DiscoverResources {
            resource_type: "computing".to_string(),
            max_results: 10,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "ResourceDiscoveryInitiated");
    }

    #[test]
    fn test_network_partitioning() {
        let mut state = setup_test_state();
        state.permissions.push("network.partition".to_string());
        
        // Test joining partition
        let join_op = NetworkOperation::JoinPartition {
            partition_id: "partition1".to_string(),
            capabilities: vec!["storage".to_string(), "compute".to_string()],
        };
        assert!(join_op.execute(&mut state).is_ok());
        
        // Test leaving partition
        let leave_op = NetworkOperation::LeavePartition {
            partition_id: "partition1".to_string(),
        };
        assert!(leave_op.execute(&mut state).is_ok());
    }

    #[test]
    fn test_topology_update() {
        let mut state = setup_test_state();
        state.permissions.push("network.topology".to_string());
        
        let connections = vec![
            ("node1".to_string(), "node2".to_string()),
            ("node2".to_string(), "node3".to_string()),
        ];
        
        let op = NetworkOperation::UpdateTopology { connections };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "TopologyUpdated");
    }

    #[test]
    fn test_health_check() {
        let mut state = setup_test_state();
        state.permissions.push("network.health_check".to_string());
        
        let op = NetworkOperation::HealthCheck {
            target: Some("peer1".to_string()),
            timeout_ms: 5000,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "HealthCheckInitiated");
    }

    #[test]
    fn test_network_configuration() {
        let mut state = setup_test_state();
        state.permissions.push("network.configure".to_string());
        
        let mut settings = HashMap::new();
        settings.insert("max_connections".to_string(), "100".to_string());
        settings.insert("timeout_ms".to_string(), "5000".to_string());
        settings.insert("encryption_enabled".to_string(), "true".to_string());
        
        let op = NetworkOperation::ConfigureNetwork { settings };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "NetworkConfigured");
    }

    #[test]
    fn test_message_priorities() {
        let mut state = setup_test_state();
        
        // Test different message priorities and their resource costs
        let priorities = vec![
            MessagePriority::Low,
            MessagePriority::Normal,
            MessagePriority::High,
            MessagePriority::Critical,
        ];
        
        for priority in priorities {
            let message = NetworkMessage {
                message_type: MessageType::Direct,
                priority: priority.clone(),
                target: Some("recipient".to_string()),
                content: "Test message".to_string(),
                metadata: HashMap::new(),
            };
            
            let op = NetworkOperation::SendMessage { message };
            assert!(op.execute(&mut state).is_ok());
        }
    }

    #[test]
    fn test_route_request() {
        let mut state = setup_test_state();
        
        let op = NetworkOperation::RequestRoute {
            target: "distant_peer".to_string(),
            max_hops: 5,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "RouteRequested");
    }

    #[test]
    fn test_insufficient_permissions() {
        let mut state = setup_test_state();
        state.permissions.clear(); // Remove all permissions
        
        let message = NetworkMessage {
            message_type: MessageType::Direct,
            priority: MessagePriority::Normal,
            target: Some("recipient".to_string()),
            content: "Hello".to_string(),
            metadata: HashMap::new(),
        };
        
        let op = NetworkOperation::SendMessage { message };
        assert!(matches!(op.execute(&mut state), Err(VMError::InsufficientPermissions)));
    }

    #[test]
    fn test_broadcast_capabilities() {
        let mut state = setup_test_state();
        state.permissions.push("network.broadcast".to_string());
        
        let capabilities = vec![
            "storage:100GB".to_string(),
            "compute:4cores".to_string(),
            "bandwidth:1Gbps".to_string(),
        ];
        
        let op = NetworkOperation::BroadcastCapabilities { capabilities };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "CapabilitiesBroadcast");
    }
}