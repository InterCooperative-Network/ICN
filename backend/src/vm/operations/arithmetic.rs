// src/vm/operations/arithmetic.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use super::{Operation, VMState, VMResult, ensure_stack_size};
use crate::vm::VMError;

/// Arithmetic operations for the VM
#[derive(Debug, Clone)]
pub enum ArithmeticOperation {
    /// Add the top two values
    Add,
    /// Subtract the top two values
    Sub,    
    /// Multiply the top two values
    Mul,    
    /// Divide the top two values
    Div,    
    /// Compute the modulo of the top two values
    Mod,    
    /// Increment the top value
    Increment,
    /// Decrement the top value
    Decrement,    
    /// Compute the absolute value
    Abs,    
    /// Negate the top value
    Negate,    
    /// Compute the minimum of two values
    Min,    
    /// Compute the maximum of two values
    Max,
}

impl Operation for ArithmeticOperation {
    fn execute(&self, state: &mut VMState) -> VMResult<()> {
        match self {
            ArithmeticOperation::Add => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                state.stack.push(a + b);
                Ok(())
            },
            
            ArithmeticOperation::Sub => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                state.stack.push(a - b);
                Ok(())
            },
            
            ArithmeticOperation::Mul => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                state.stack.push(a * b);
                Ok(())
            },
            
            ArithmeticOperation::Div => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                state.stack.push(a / b);
                Ok(())
            },
            
            ArithmeticOperation::Mod => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                state.stack.push(a % b);
                Ok(())
            },
            
            ArithmeticOperation::Increment => {
                ensure_stack_size(&state.stack, 1)?;
                let a = state.stack.pop().unwrap();
                state.stack.push(a + 1);
                Ok(())
            },
            
            ArithmeticOperation::Decrement => {
                ensure_stack_size(&state.stack, 1)?;
                let a = state.stack.pop().unwrap();
                state.stack.push(a - 1);
                Ok(())
            },
            
            ArithmeticOperation::Abs => {
                ensure_stack_size(&state.stack, 1)?;
                let a = state.stack.pop().unwrap();
                state.stack.push(a.abs());
                Ok(())
            },
            
            ArithmeticOperation::Negate => {
                ensure_stack_size(&state.stack, 1)?;
                let a = state.stack.pop().unwrap();
                state.stack.push(-a);
                Ok(())
            },
            
            ArithmeticOperation::Min => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                state.stack.push(a.min(b));
                Ok(())
            },
            
            ArithmeticOperation::Max => {
                ensure_stack_size(&state.stack, 2)?;
                let b = state.stack.pop().unwrap();
                let a = state.stack.pop().unwrap();
                state.stack.push(a.max(b));
                Ok(())
            },
        }
    }

    fn resource_cost(&self) -> u64 {
        match self {
            ArithmeticOperation::Add => 2,
            ArithmeticOperation::Sub => 2,
            ArithmeticOperation::Mul => 3,
            ArithmeticOperation::Div => 3,
            ArithmeticOperation::Mod => 3,
            ArithmeticOperation::Increment => 1,
            ArithmeticOperation::Decrement => 1,
            ArithmeticOperation::Abs => 1,
            ArithmeticOperation::Negate => 1,
            ArithmeticOperation::Min => 2,
            ArithmeticOperation::Max => 2,
        }
    }

    fn required_permissions(&self) -> Vec<String> {
        // Arithmetic operations don't require special permissions
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::Event;

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
            memory_limit: 1024 * 1024, // 1MB
            memory_address_counter: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_stack_operations() {
        let mut state = setup_test_state();
        
        // Test Add
        state.stack.extend([5, 3]);
        let add_op = ArithmeticOperation::Add;
        assert!(add_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![8]);

        // Test Subtract
        state.stack.extend([10]);
        let sub_op = ArithmeticOperation::Sub;
        assert!(sub_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![-2]); // 8 - 10 = -2

        // Test Multiply
        state.stack.extend([3]);
        let mul_op = ArithmeticOperation::Mul;
        assert!(mul_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![-6]); // -2 * 3 = -6
    }

    #[test]
    fn test_div_by_zero() {
        let mut state = setup_test_state();
        state.stack.extend([5, 0]);
        let op = ArithmeticOperation::Div;
        assert!(matches!(op.execute(&mut state), Err(VMError::DivisionByZero)));
    }

    #[test]
    fn test_advanced_operations() {
        let mut state = setup_test_state();

        // Test Abs
        state.stack.push(-5);
        let abs_op = ArithmeticOperation::Abs;
        assert!(abs_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5]);

        // Test Min/Max
        state.stack.extend([10, 3]);
        let min_op = ArithmeticOperation::Min;
        assert!(min_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5, 3]);

        state.stack.extend([8]);
        let max_op = ArithmeticOperation::Max;
        assert!(max_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5, 8]);
    }

    #[test]
    fn test_increment_decrement() {
        let mut state = setup_test_state();
        
        // Test Increment
        state.stack.push(5);
        let inc_op = ArithmeticOperation::Increment;
        assert!(inc_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![6]);

        // Test Decrement
        let dec_op = ArithmeticOperation::Decrement;
        assert!(dec_op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5]);
    }

    #[test]
    fn test_resource_costs() {
        let add_op = ArithmeticOperation::Add;
        let mul_op = ArithmeticOperation::Mul;
        let inc_op = ArithmeticOperation::Increment;

        assert_eq!(add_op.resource_cost(), 2);
        assert_eq!(mul_op.resource_cost(), 3);
        assert_eq!(inc_op.resource_cost(), 1);
    }

    #[test]
    fn test_stack_underflow() {
        let mut state = setup_test_state();
        let op = ArithmeticOperation::Add;
        
        // Empty stack
        assert!(matches!(op.execute(&mut state), Err(VMError::StackUnderflow)));

        // Single element (need two for add)
        state.stack.push(5);
        assert!(matches!(op.execute(&mut state), Err(VMError::StackUnderflow)));
    }
}