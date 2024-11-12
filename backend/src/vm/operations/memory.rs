// src/vm/operations/memory.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_permissions, emit_event};
use crate::vm::VMError;

/// Memory segment types for different use cases
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

/// Memory allocation request details
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    /// Size in bytes to allocate
    size: u64,
    /// Type of memory segment
    segment_type: MemorySegment,
    /// Optional federation ID for shared memory
    federation_id: Option<String>,
    /// Whether the memory should persist across VM invocations
    persistent: bool,
}

/// Memory management operations for the VM
pub enum MemoryOperation {
    /// Allocate new memory space
    Allocate {
        request: AllocationRequest,
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
            MemoryOperation::Allocate { request } => {
                ensure_permissions(&["memory.allocate".to_string()], &state.permissions)?;
                
                // Check if we have enough available memory
                let current_usage = state.memory.len() as u64;
                if current_usage + request.size > state.memory_limit {
                    return Err(VMError::OutOfMemory);
                }
                
                // Generate a unique address for this allocation
                let address = format!("{}:{}", 
                    match request.segment_type {
                        MemorySegment::Cooperative => "coop",
                        MemorySegment::Federation => "fed",
                        MemorySegment::Scratch => "scratch",
                        MemorySegment::Persistent => "persist",
                    },
                    state.next_memory_address()
                );
                
                // Record the allocation
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                event_data.insert("size".to_string(), request.size.to_string());
                event_data.insert("segment_type".to_string(), format!("{:?}", request.segment_type));
                if let Some(fed_id) = &request.federation_id {
                    event_data.insert("federation_id".to_string(), fed_id.clone());
                }
                
                emit_event(state, "MemoryAllocated".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::Free { address, segment_type } => {
                ensure_permissions(&["memory.free".to_string()], &state.permissions)?;
                
                // Verify the memory segment exists and belongs to the caller
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
                
                let segment_id = format!("shared:{}:{}", federation_id, state.next_memory_address());
                
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
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                event_data.insert("new_size".to_string(), new_size.to_string());
                
                emit_event(state, "MemoryResized".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::GetMemoryInfo { segment_id } => {
                let mut event_data = HashMap::new();
                event_data.insert("segment_id".to_string(), segment_id.clone());
                
                // In a real implementation, would return actual memory info
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

            MemoryOperation::PinMemory { address } => {
                ensure_permissions(&["memory.pin".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                
                emit_event(state, "MemoryPinned".to_string(), event_data);
                Ok(())
            },

            MemoryOperation::UnpinMemory { address } => {
                ensure_permissions(&["memory.pin".to_string()], &state.permissions)?;
                
                let mut event_data = HashMap::new();
                event_data.insert("address".to_string(), address.clone());
                
                emit_event(state, "MemoryUnpinned".to_string(), event_data);
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
            MemoryOperation::Allocate { request } => 10 + (request.size / 1024), // Base cost plus size-based cost
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
            MemoryOperation::CreateSharedSegment { .. } => vec!["memory.share".to_string()],
            MemoryOperation::JoinSharedSegment { .. } => vec!["memory.share".to_string()],
            MemoryOperation::CopyMemory { .. } => vec!["memory.copy".to_string()],
            MemoryOperation::MoveMemory { .. } => vec!["memory.move".to_string()],
            MemoryOperation::Resize { .. } => vec!["memory.resize".to_string()],
            MemoryOperation::ClearMemory { .. } => vec!["memory.clear".to_string()],
            MemoryOperation::CollectGarbage => vec!["memory.gc".to_string()],
            MemoryOperation::PinMemory { .. } | MemoryOperation::UnpinMemory { .. } => vec!["memory.pin".to_string()],
            MemoryOperation::CompactMemory { .. } => vec!["memory.compact".to_string()],
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
                "memory.allocate".to_string(),
                "memory.free".to_string(),
                "memory.share".to_string(),
                "memory.copy".to_string(),
                "memory.move".to_string(),
            ],
            memory_limit: 1024 * 1024, // 1MB limit for testing
        };
        
        state.reputation_context.insert(state.caller_did.clone(), 100);
        state
    }

    #[test]
    fn test_allocate_memory() {
        let mut state = setup_test_state();
        let request = AllocationRequest {
            size: 1024,
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        
        let op = MemoryOperation::Allocate { request };
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryAllocated");
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
    fn test_out_of_memory() {
        let mut state = setup_test_state();
        state.memory_limit = 1024; // Set small memory limit
        
        let request = AllocationRequest {
            size: 2048, // Try to allocate more than limit
            segment_type: MemorySegment::Cooperative,
            federation_id: None,
            persistent: false,
        };
        
        let op = MemoryOperation::Allocate { request };
        assert!(matches!(op.execute(&mut state), Err(VMError::OutOfMemory)));
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
            segment_id: "shared:fed1:1".to_string(),
        };
        assert!(join_op.execute(&mut state).is_ok());
    }

    #[test]
    fn test_resize_memory() {
        let mut state = setup_test_state();
        state.permissions.push("memory.resize".to_string());
        state.memory.insert("test_segment".to_string(), 0);
        
        let op = MemoryOperation::Resize {
            address: "test_segment".to_string(),
            new_size: 2048,
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.events[0].event_type, "MemoryResized");
    }
}