// src/vm/operations/stack.rs

use std::collections::HashMap;
use super::{Operation, VMState, VMResult, ensure_stack_size};
use crate::vm::VMError;
use std::sync::atomic::AtomicU64;

/// Stack manipulation operations
pub enum StackOperation {
    /// Push a value onto the stack
    Push(i64),
    
    /// Pop a value from the stack
    Pop,
    
    /// Duplicate the top value on the stack
    Dup,
    
    /// Swap the top two values on the stack
    Swap,
    
    /// Duplicate the nth value on the stack (n=0 is top)
    DupN(usize),
    
    /// Rotate the top n values on the stack
    Rotate(usize),
    
    /// Clear the entire stack
    Clear,
}

impl Operation for StackOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            StackOperation::Push(value) => {
                if state.stack.len() >= 1024 {  // Stack size limit
                    return Err(VMError::StackOverflow);
                }
                state.stack.push(*value);
                Ok(())
            },
            
            StackOperation::Pop => {
                ensure_stack_size(&state.stack, 1)?;
                state.stack.pop()
                    .ok_or(VMError::StackUnderflow)
                    .map(|_| ())
            },
            
            StackOperation::Dup => {
                ensure_stack_size(&state.stack, 1)?;
                let value = *state.stack.last()
                    .ok_or(VMError::StackUnderflow)?;
                state.stack.push(value);
                Ok(())
            },
            
            StackOperation::Swap => {
                ensure_stack_size(&state.stack, 2)?;
                let len = state.stack.len();
                state.stack.swap(len - 1, len - 2);
                Ok(())
            },
            
            StackOperation::DupN(n) => {
                ensure_stack_size(&state.stack, *n + 1)?;
                let len = state.stack.len();
                let value = state.stack[len - 1 - n];
                state.stack.push(value);
                Ok(())
            },
            
            StackOperation::Rotate(n) => {
                let n = *n;
                if n == 0 {
                    return Ok(());
                }
                ensure_stack_size(&state.stack, n)?;
                let len = state.stack.len();
                let temp = state.stack[len - 1];
                for i in (len - n..len).rev() {
                    state.stack[i] = state.stack[i - 1];
                }
                state.stack[len - n] = temp;
                Ok(())
            },
            
            StackOperation::Clear => {
                state.stack.clear();
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            StackOperation::Push(_) => 1,
            StackOperation::Pop => 1,
            StackOperation::Dup => 1,
            StackOperation::Swap => 2,
            StackOperation::DupN(n) => 1 + (*n as u64),
            StackOperation::Rotate(n) => 1 + (*n as u64),
            StackOperation::Clear => 1,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        // Stack operations don't require special permissions
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            permissions: vec![],
            memory_limit: 1024 * 1024, // 1MB default limit
            memory_address_counter: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_push() {
        let mut state = setup_test_state();
        let op = StackOperation::Push(42);
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![42]);
    }

    #[test]
    fn test_pop() {
        let mut state = setup_test_state();
        state.stack.push(42);
        let op = StackOperation::Pop;
        assert!(op.execute(&mut state).is_ok());
        assert!(state.stack.is_empty());
    }

    #[test]
    fn test_pop_empty() {
        let mut state = setup_test_state();
        let op = StackOperation::Pop;
        assert!(matches!(op.execute(&mut state), Err(VMError::StackUnderflow)));
    }

    #[test]
    fn test_dup() {
        let mut state = setup_test_state();
        state.stack.push(42);
        let op = StackOperation::Dup;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![42, 42]);
    }

    #[test]
    fn test_swap() {
        let mut state = setup_test_state();
        state.stack.extend([1, 2]);
        let op = StackOperation::Swap;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![2, 1]);
    }

    #[test]
    fn test_dupn() {
        let mut state = setup_test_state();
        state.stack.extend([1, 2, 3]);
        let op = StackOperation::DupN(2);
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![1, 2, 3, 1]);
    }

    #[test]
    fn test_rotate() {
        let mut state = setup_test_state();
        state.stack.extend([1, 2, 3]);
        let op = StackOperation::Rotate(3);
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![2, 3, 1]);
    }

    #[test]
    fn test_clear() {
        let mut state = setup_test_state();
        state.stack.extend([1, 2, 3]);
        let op = StackOperation::Clear;
        assert!(op.execute(&mut state).is_ok());
        assert!(state.stack.is_empty());
    }
}