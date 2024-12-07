// src/vm/operations/data.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_stack_size, ensure_permissions, emit_event};
use crate::vm::VMError;

/// Operations for handling structured data within the VM
pub enum DataOperation {
    /// Create a new data structure of specified type
    CreateStruct {
        name: String,
        fields: Vec<String>,
    },
    
    /// Set a field value in a structure
    SetField {
        struct_name: String,
        field_name: String,
    },
    
    /// Get a field value from a structure
    GetField {
        struct_name: String,
        field_name: String,
    },
    
    /// Create a new array with initial size
    CreateArray {
        size: usize,
    },
    
    /// Get value at array index
    GetArrayValue {
        index: usize,
    },
    
    /// Set value at array index
    SetArrayValue {
        index: usize,
    },
    
    /// Get array length
    GetArrayLength,
    
    /// Create a new hashmap
    CreateMap,
    
    /// Set key-value pair in map
    SetMapValue,
    
    /// Get value by key from map
    GetMapValue,
    
    /// Check if key exists in map
    HasMapKey,
    
    /// Delete key-value pair from map
    DeleteMapValue,
    
    /// Serialize data structure to bytes
    Serialize,
    
    /// Deserialize bytes to data structure
    Deserialize {
        target_type: String,
    },
}

impl Operation for DataOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            DataOperation::CreateStruct { name, fields } => {
                ensure_permissions(&["data.create".to_string()], &state.permissions)?;
                
                // Create structure data with initialized fields
                let struct_data: HashMap<String, i64> = fields.iter()
                    .map(|field| (field.clone(), 0i64))
                    .collect();

                // Store the structure definition and reference
                state.memory.insert(format!("struct_def:{}", name), fields.len() as i64);
                for (field, value) in &struct_data {
                    state.memory.insert(format!("struct:{}:{}", name, field), *value);
                }
                
                let mut event_data = HashMap::new();
                event_data.insert("struct_name".to_string(), name.clone());
                event_data.insert("field_count".to_string(), fields.len().to_string());
                
                emit_event(state, "StructCreated".to_string(), event_data);
                Ok(())
            },
            
            DataOperation::SetField { struct_name, field_name } => {
                ensure_stack_size(&state.stack, 1)?;
                let value = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                
                let key = format!("struct:{}:{}", struct_name, field_name);
                if !state.memory.contains_key(&format!("struct_def:{}", struct_name)) {
                    return Err(VMError::Custom("Structure not found".to_string()));
                }
                
                state.memory.insert(key, value);
                
                let mut event_data = HashMap::new();
                event_data.insert("struct_name".to_string(), struct_name.clone());
                event_data.insert("field_name".to_string(), field_name.clone());
                event_data.insert("value".to_string(), value.to_string());
                
                emit_event(state, "FieldUpdated".to_string(), event_data);
                Ok(())
            },
            
            DataOperation::GetField { struct_name, field_name } => {
                let key = format!("struct:{}:{}", struct_name, field_name);
                let value = state.memory.get(&key)
                    .copied()
                    .ok_or_else(|| VMError::Custom("Field not found".to_string()))?;
                
                state.stack.push(value);
                Ok(())
            },
            
            DataOperation::CreateArray { size } => {
                ensure_permissions(&["data.create".to_string()], &state.permissions)?;
                
                // Initialize array with zeros
                for i in 0..*size {
                    state.memory.insert(format!("array:{}", i), 0);
                }
                
                state.memory.insert("array:length".to_string(), *size as i64);
                
                let mut event_data = HashMap::new();
                event_data.insert("size".to_string(), size.to_string());
                
                emit_event(state, "ArrayCreated".to_string(), event_data);
                Ok(())
            },
            
            DataOperation::GetArrayValue { index } => {
                let length = state.memory.get("array:length")
                    .copied()
                    .unwrap_or(0) as usize;
                    
                if *index >= length {
                    return Err(VMError::Custom("Array index out of bounds".to_string()));
                }
                
                let key = format!("array:{}", index);
                let value = state.memory.get(&key)
                    .copied()
                    .unwrap_or(0);
                
                state.stack.push(value);
                Ok(())
            },
            
            DataOperation::SetArrayValue { index } => {
                ensure_stack_size(&state.stack, 1)?;
                
                let length = state.memory.get("array:length")
                    .copied()
                    .unwrap_or(0) as usize;
                    
                if *index >= length {
                    return Err(VMError::Custom("Array index out of bounds".to_string()));
                }
                
                let value = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                let key = format!("array:{}", index);
                state.memory.insert(key, value);
                
                let mut event_data = HashMap::new();
                event_data.insert("index".to_string(), index.to_string());
                event_data.insert("value".to_string(), value.to_string());
                
                emit_event(state, "ArrayValueUpdated".to_string(), event_data);
                Ok(())
            },
            
            DataOperation::GetArrayLength => {
                let length = state.memory.get("array:length")
                    .copied()
                    .unwrap_or(0);
                
                state.stack.push(length);
                Ok(())
            },
            
            DataOperation::CreateMap => {
                ensure_permissions(&["data.create".to_string()], &state.permissions)?;
                state.memory.insert("map:size".to_string(), 0);
                
                emit_event(state, "MapCreated".to_string(), HashMap::new());
                Ok(())
            },
            
            DataOperation::SetMapValue => {
                ensure_stack_size(&state.stack, 2)?;
                ensure_permissions(&["data.write".to_string()], &state.permissions)?;
                
                let value = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                let key = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                
                let map_key = format!("map:{}:value", key);
                state.memory.insert(map_key, value);
                
                // Update map size
                let size = state.memory.get("map:size")
                    .copied()
                    .unwrap_or(0);
                state.memory.insert("map:size".to_string(), size + 1);
                
                let mut event_data = HashMap::new();
                event_data.insert("key".to_string(), key.to_string());
                event_data.insert("value".to_string(), value.to_string());
                
                emit_event(state, "MapValueSet".to_string(), event_data);
                Ok(())
            },
            
            DataOperation::GetMapValue => {
                ensure_stack_size(&state.stack, 1)?;
                
                let key = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                let map_key = format!("map:{}:value", key);
                
                let value = state.memory.get(&map_key)
                    .copied()
                    .unwrap_or(0);
                
                state.stack.push(value);
                Ok(())
            },
            
            DataOperation::HasMapKey => {
                ensure_stack_size(&state.stack, 1)?;
                
                let key = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                let map_key = format!("map:{}:value", key);
                
                let exists = state.memory.contains_key(&map_key);
                state.stack.push(if exists { 1 } else { 0 });
                
                Ok(())
            },
            
            DataOperation::DeleteMapValue => {
                ensure_stack_size(&state.stack, 1)?;
                ensure_permissions(&["data.write".to_string()], &state.permissions)?;
                
                let key = state.stack.pop().ok_or(VMError::StackUnderflow)?;
                let map_key = format!("map:{}:value", key);
                
                if state.memory.remove(&map_key).is_some() {
                    let size = state.memory.get("map:size")
                        .copied()
                        .unwrap_or(0);
                    state.memory.insert("map:size".to_string(), size - 1);
                    
                    let mut event_data = HashMap::new();
                    event_data.insert("key".to_string(), key.to_string());
                    
                    emit_event(state, "MapValueDeleted".to_string(), event_data);
                }
                
                Ok(())
            },
            
            DataOperation::Serialize | DataOperation::Deserialize { .. } => {
                // These operations would typically involve more complex serialization logic
                // For now, we just emit an event
                let mut event_data = HashMap::new();
                event_data.insert("operation".to_string(), "serialization".to_string());
                
                emit_event(state, "DataOperation".to_string(), event_data);
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            DataOperation::CreateStruct { fields, .. } => 10 + (fields.len() as u64),
            DataOperation::SetField { .. } => 5,
            DataOperation::GetField { .. } => 2,
            DataOperation::CreateArray { size } => 10 + (*size as u64),
            DataOperation::GetArrayValue { .. } => 2,
            DataOperation::SetArrayValue { .. } => 5,
            DataOperation::GetArrayLength => 1,
            DataOperation::CreateMap => 10,
            DataOperation::SetMapValue => 5,
            DataOperation::GetMapValue => 2,
            DataOperation::HasMapKey => 2,
            DataOperation::DeleteMapValue => 5,
            DataOperation::Serialize => 20,
            DataOperation::Deserialize { .. } => 20,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        match self {
            DataOperation::CreateStruct { .. } |
            DataOperation::CreateArray { .. } |
            DataOperation::CreateMap => vec!["data.create".to_string()],
            
            DataOperation::SetField { .. } |
            DataOperation::SetArrayValue { .. } |
            DataOperation::SetMapValue |
            DataOperation::DeleteMapValue => vec!["data.write".to_string()],
            
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_state() -> VMState {
        let mut state = VMState::default();
        state.permissions = vec!["data.create".to_string(), "data.write".to_string()];
        state
    }

    #[test]
    fn test_create_struct() {
        let mut state = setup_test_state();
        let op = DataOperation::CreateStruct {
            name: "test".to_string(),
            fields: vec!["field1".to_string(), "field2".to_string()],
        };
        
        assert!(op.execute(&mut state).is_ok());
        assert!(state.memory.contains_key("struct_def:test"));
    }

    #[test]
    fn test_array_operations() {
        let mut state = setup_test_state();
        
        // Create array
        let create_op = DataOperation::CreateArray { size: 3 };
        assert!(create_op.execute(&mut state).is_ok());
        
        // Set value
        state.stack.push(42);
        let set_op = DataOperation::SetArrayValue { index: 1 };
        assert!(set_op.execute(&mut state).is_ok());
        
        // Get value
        let get_op = DataOperation::GetArrayValue { index: 1 };
        assert!(get_op.execute(&mut state).is_ok());
        assert_eq!(state.stack.pop().unwrap(), 42);
    }

    #[test]
    fn test_map_operations() {
        let mut state = setup_test_state();
        
        // Create map
        let create_op = DataOperation::CreateMap;
        assert!(create_op.execute(&mut state).is_ok());
        
        // Set value
        state.stack.push(1); // key
        state.stack.push(42); // value
        let set_op = DataOperation::SetMapValue;
        assert!(set_op.execute(&mut state).is_ok());
        
        // Get value
        state.stack.push(1); // key
        let get_op = DataOperation::GetMapValue;
        assert!(get_op.execute(&mut state).is_ok());
        assert_eq!(state.stack.pop().unwrap(), 42);
    }

    #[test]
    fn test_array_bounds() {
        let mut state = setup_test_state();
        
        // Create small array
        let create_op = DataOperation::CreateArray { size: 2 };
        assert!(create_op.execute(&mut state).is_ok());
        
        // Try to access invalid index
        let get_op = DataOperation::GetArrayValue { index: 5 };
        assert!(matches!(
            get_op.execute(&mut state),
            Err(VMError::Custom(_))
        ));
    }

    #[test]
    fn test_permissions() {
        let mut state = setup_test_state();
        state.permissions.clear(); // Remove all permissions
        
        let op = DataOperation::CreateStruct {
            name: "test".to_string(),
            fields: vec!["field1".to_string()],
        };
        
        assert!(matches!(
            op.execute(&mut state),
            Err(VMError::InsufficientPermissions)
        ));
    }
}