use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::vm::VMError;
use std::sync::atomic::Ordering;

/// Types of memory segment for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum MemorySegment {
    /// Private memory for a single cooperative
    Cooperative,
    /// Shared memory between federation members
    Federation,
    /// Temporary computation space
    Scratch,
    /// Persistent storage space
    Persistent,
}

/// Operations for managing memory within the VM
pub enum MemoryOperation {
    /// Allocate new memory space
    Allocate {
        size: u64,
        segment_type: MemorySegment,
        federation_id: Option<String>,
        persistent: bool,
    },
    
    /// Free allocated memory
    Free {
        address: String,
        segment_type: MemorySegment,
    },
    
    /// Create shared memory segment for federation
    CreateSharedSegment {
        federation_id: String,
        size: u64,
        access_control: Vec<String>,
    },
    
    /// Join existing shared memory segment
    JoinSharedSegment {
        federation_id: String,
        segment_id: String,
    },
    
    /// Copy data between memory segments
    CopyMemory {
        source: String,
        destination: String,
        size: u64,
    },
    
    /// Move data between memory segments
    MoveMemory {
        source: String,
        destination: String,
        size: u64,
    },
    
    /// Resize existing memory allocation
    Resize {
        address: String,
        new_size: u64,
    },
    
    /// Get memory segment information
    GetMemoryInfo {
        segment_id: String,
    },
    
    /// Clear memory segment
    ClearMemory {
        segment_id: String,
    },
    
    /// Mark memory region for garbage collection
    MarkForCollection {
        address: String,
    },
    
    /// Run garbage collection
    CollectGarbage,
    
    /// Pin memory to prevent garbage collection
    PinMemory {
        address: String,
    },
    
    /// Unpin previously pinned memory
    UnpinMemory {
        address: String,
    },
    
    /// Compact memory to reduce fragmentation
    CompactMemory {
        segment_type: MemorySegment,
    },
}

impl Operation for MemoryOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            MemoryOperation::Allocate { size, segment_type, federation_id, persistent } => {
                ensure_permissions(&["memory.allocate".to_string()], &state.permissions)?;
                
                // Check if we have enough available memory
                let current_usage = state.memory.len() as u64;
                if current_usage + size > state.memory_limit {
                    return Err(VMError::OutOfMemory);
                }
                
                // Generate unique address
                let address = format!("{}:{}",
                    match segment_type {
                        MemorySegment::Cooperative => "coop",
                        MemorySegment::Federation => "fed", 
                        MemorySegment::Scratch => "scratch",
                        MemorySegment::Persistent => "persist",
                    },
                    state.memory_address_counter.fetch_add(1, Ordering::SeqCst)
                );

                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                event_data.insert("size".to_string(), size.to_string());
                event_data.insert("segment_type".to_string(), format!("{:?}", segment_type));
                if let Some(fed_id) = federation_id {
                    event_data.insert("federation_id".to_string(), fed_id.clone());
                }
                event_data.insert("persistent".to_string(), persistent.to_string());
                
                emit_event(state, "MemoryAllocated".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::Free { address, segment_type } => {
                ensure_permissions(&["memory.free".to_string()], &state.permissions)?;
                
                if !state.memory.contains_key(address) {
                    return Err(VMError::InvalidMemoryAddress);
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                event_data.insert("segment_type".to_string(), format!("{:?}", segment_type));
                
                emit_event(state, "MemoryFreed".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::CreateSharedSegment { federation_id, size, access_control } => {
                ensure_permissions(&["memory.share".to_string()], &state.permissions)?;
                
                let segment_id = format!("shared:{}:{}", federation_id, 
                    state.memory_address_counter.fetch_add(1, Ordering::SeqCst));
                
                let mut event_data = HashMap::new();
                event_data.insert("segment_id".to_string(), segment_id);
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("size".to_string(), size.to_string());
                event_data.insert("access_control".to_string(), access_control.join(","));
                
                emit_event(state, "SharedMemoryCreated".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::JoinSharedSegment { federation_id, segment_id } => {
                ensure_permissions(&["memory.share".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("federation_id".to_string(), federation_id.clone());
                event_data.insert("segment_id".to_string(), segment_id.clone());
                
                emit_event(state, "SharedMemoryJoined".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::CopyMemory { source, destination, size } => {
                ensure_permissions(&["memory.copy".to_string()], &state.permissions)?;
                
                if !state.memory.contains_key(source) || !state.memory.contains_key(destination) {
                    return Err(VMError::InvalidMemoryAddress);
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("source".to_string(), source.clone());
                event_data.insert("destination".to_string(), destination.clone());
                event_data.insert("size".to_string(), size.to_string());
                
                emit_event(state, "MemoryCopied".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::MoveMemory { source, destination, size } => {
                ensure_permissions(&["memory.move".to_string()], &state.permissions)?;
                
                if !state.memory.contains_key(source) || !state.memory.contains_key(destination) {
                    return Err(VMError::InvalidMemoryAddress);
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("source".to_string(), source.clone());
                event_data.insert("destination".to_string(), destination.clone());
                event_data.insert("size".to_string(), size.to_string());
                
                emit_event(state, "MemoryMoved".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::Resize { address, new_size } => {
                ensure_permissions(&["memory.resize".to_string()], &state.permissions)?;
                
                if !state.memory.contains_key(address) {
                    return Err(VMError::InvalidMemoryAddress);
                }
                
                // Check if we have enough memory for the resize
                let current_usage = state.memory.len() as u64;
                if current_usage + new_size > state.memory_limit {
                    return Err(VMError::OutOfMemory);
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                event_data.insert("new_size".to_string(), new_size.to_string());
                
                emit_event(state, "MemoryResized".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::GetMemoryInfo { segment_id } => {
                let mut event_data = HashMap::new();
                event_data.insert("segment_id".to_string(), segment_id.clone());
                
                emit_event(state, "MemoryInfoQueried".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::ClearMemory { segment_id } => {
                ensure_permissions(&["memory.clear".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("segment_id".to_string(), segment_id.clone());
                
                emit_event(state, "MemoryCleared".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::MarkForCollection { address } => {
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                
                emit_event(state, "MemoryMarkedForCollection".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::CollectGarbage => {
                ensure_permissions(&["memory.gc".to_string()], &state.permissions)?;
                
                emit_event(state, "GarbageCollectionRun".to_string(), HashMap::new());
                Ok(())
            },

            MemoryOperation::PinMemory { address } | MemoryOperation::UnpinMemory { address } => {
                ensure_permissions(&["memory.pin".to_string()], &state.permissions)?;
                
                let event_type = match self {
                    MemoryOperation::PinMemory { .. } => "MemoryPinned",
                    _ => "MemoryUnpinned",
                };
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                
                emit_event(state, event_type.to_string(), event_data);
                Ok(())
            },

            MemoryOperation::CompactMemory { segment_type } => {
                ensure_permissions(&["memory.compact".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("segment_type".to_string(), format!("{:?}", segment_type));
                
                emit_event(state, "MemoryCompacted".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            MemoryOperation::Allocate { size, .. } => 10 + (size / 1024),
            MemoryOperation::Free { .. } => 5,
            MemoryOperation::CreateSharedSegment { size, .. } => 20 + (size / 1024),
            MemoryOperation::JoinSharedSegment { .. } => 10,
            MemoryOperation::CopyMemory { size, .. } => 5 + (size / 1024),
            MemoryOperation::MoveMemory { size, .. } => 5 + (size / 1024),
            MemoryOperation::Resize { new_size, .. } => 10 + (new_size / 1024),
            MemoryOperation::GetMemoryInfo { .. } => 2,
            MemoryOperation::ClearMemory { .. } => 5,
            MemoryOperation::MarkForCollection { .. } => 2,
            MemoryOperation::CollectGarbage => 50,
            MemoryOperation::PinMemory { .. } => 5,
            MemoryOperation::UnpinMemory { .. } => 5,
            MemoryOperation::CompactMemory { .. } => 30,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            MemoryOperation::Allocate { .. } => vec!["memory.allocate".to_string()],
            MemoryOperation::Free { .. } => vec!["memory.free".to_string()],
            MemoryOperation::CreateSharedSegment { .. } | MemoryOperation::JoinSharedSegment { .. } => 
                vec!["memory.share".to_string()],
            MemoryOperation::CopyMemory { .. } => vec!["memory.copy".to_string()],
            MemoryOperation::MoveMemory { .. } => vec!["memory.move".to_string()],
            MemoryOperation::Resize { .. } => vec!["memory.resize".to_string()],
            MemoryOperation::ClearMemory { .. } => vec!["memory.clear".to_string()],
            MemoryOperation::CollectGarbage => vec!["memory.gc".to_string()],
            MemoryOperation::PinMemory { .. } | MemoryOperation::UnpinMemory { .. } => 
                vec!["memory.pin".to_string()],
            MemoryOperation::CompactMemory { .. } => vec!["memory.compact".to_string()],
            _ => vec![],
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
                "memory.allocate".to_string(),
                "memory.free".to_string(),
                "memory.share".to_string(),
                "memory.copy".to_string(),
                "memory.move".to_string(),
            ],
            memory_limit: 1024 * 1024, // 1MB
            memory_address_counter: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_allocate_memory() {
        let mut state = setup_test_state();
        let op = MemoryOperation::Allocate { 
            size: 1024,
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryAllocated");
    }

    #[test]
    fn test_out_of_memory() {
        let mut state = setup_test_state();
        state.memory_limit = 1024; // Set small memory limit
        
        let op = MemoryOperation::Allocate {
            size: 2048,
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        
        assert!(matches!(op.execute(&mut state), Err(VMError::OutOfMemory)));
    }

    #[test]
    fn test_shared_memory() {
        let mut state = setup_test_state();
        let op = MemoryOperation::CreateSharedSegment {
            federation_id: "fed1".to_string(),
            size: 2048,
            access_control: vec!["coop1".to_string(), "coop2".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "SharedMemoryCreated");
    }

    #[test]
    fn test_copy_memory() {
        let mut state = setup_test_state();
        // Set up source and destination in memory
        state.memory.insert("source".to_string(), 42);
        state.memory.insert("dest".to_string(), 0);
        
        let op = MemoryOperation::CopyMemory {
            source: "source".to_string(),
            destination: "dest".to_string(),
            size: 8,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryCopied");
    }

    #[test]
    fn test_garbage_collection() {
        let mut state = setup_test_state();
        state.permissions.push("memory.gc".to_string());
        
        // Mark some memory for collection
        let mark_op = MemoryOperation::MarkForCollection {
            address: "unused".to_string(),
        };
        assert!(mark_op.execute(&mut state).is_ok());
        
        // Run garbage collection
        let gc_op = MemoryOperation::CollectGarbage;
        assert!(gc_op.execute(&mut state).is_ok());
        assert_eq!(state.events[1].event_type, "GarbageCollectionRun");
    }

    #[test]
    fn test_memory_pin_unpin() {
        let mut state = setup_test_state();
        state.permissions.push("memory.pin".to_string());
        
        let pin_op = MemoryOperation::PinMemory {
            address: "important".to_string(),
        };
        assert!(pin_op.execute(&mut state).is_ok());
        
        let unpin_op = MemoryOperation::UnpinMemory {
            address: "important".to_string(),
        };
        assert!(unpin_op.execute(&mut state).is_ok());
        
        assert_eq!(state.events[0].event_type, "MemoryPinned");
        assert_eq!(state.events[1].event_type, "MemoryUnpinned");
    }

    #[test]
    fn test_memory_compaction() {
        let mut state = setup_test_state();
        state.permissions.push("memory.compact".to_string());
        
        let op = MemoryOperation::CompactMemory {
            segment_type: MemorySegment::Cooperative,
        };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryCompacted");
    }

    #[test]
    fn test_invalid_memory_address() {
        let mut state = setup_test_state();
        let op = MemoryOperation::Free {
            address: "nonexistent".to_string(),
            segment_type: MemorySegment::Cooperative,
        };
        
        assert!(matches!(op.execute(&mut state), Err(VMError::InvalidMemoryAddress)));
    }

    #[test]
    fn test_memory_resize() {
        let mut state = setup_test_state();
        state.permissions.push("memory.resize".to_string());
        
        // First allocate some memory
        state.memory.insert("test_segment".to_string(), 0);
        
        let op = MemoryOperation::Resize {
            address: "test_segment".to_string(),
            new_size: 2048,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryResized");
    }

    #[test]
    fn test_permission_checks() {
        let mut state = setup_test_state();
        state.permissions.clear(); // Remove all permissions
        
        let op = MemoryOperation::Allocate {
            size: 1024,
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        
        assert!(matches!(op.execute(&mut state), Err(VMError::InsufficientPermissions)));
    }

    #[test]
    fn test_federation_shared_memory() {
        let mut state = setup_test_state();
        
        // Create shared memory segment
        let create_op = MemoryOperation::CreateSharedSegment {
            federation_id: "fed1".to_string(),
            size: 1024,
            access_control: vec!["coop1".to_string(), "coop2".to_string()],
        };
        assert!(create_op.execute(&mut state).is_ok());
        
        // Join shared memory segment
        let join_op = MemoryOperation::JoinSharedSegment {
            federation_id: "fed1".to_string(),
            segment_id: "shared:fed1:0".to_string(),
        };
        assert!(join_op.execute(&mut state).is_ok());
        
        assert_eq!(state.events[0].event_type, "SharedMemoryCreated");
        assert_eq!(state.events[1].event_type, "SharedMemoryJoined");
    }

    #[test]
    fn test_resource_costs() {
        let alloc_op = MemoryOperation::Allocate {
            size: 1024,
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        assert_eq!(alloc_op.resource_cost(), 11); // 10 + (1024/1024)
        
        let gc_op = MemoryOperation::CollectGarbage;
        assert_eq!(gc_op.resource_cost(), 50);
    }
}
