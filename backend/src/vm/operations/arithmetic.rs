// src/vm/operations/arithmetic.rs

use super::{Operation, VMState, VMResult, ensure_stack_size};
use crate::vm::VMError;

/// Arithmetic operations for the VM
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
        }
    }

    #[test]
    fn test_add() {
        let mut state = setup_test_state();
        state.stack.extend([5, 3]);
        let op = ArithmeticOperation::Add;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![8]);
    }

    #[test]
    fn test_sub() {
        let mut state = setup_test_state();
        state.stack.extend([5, 3]);
        let op = ArithmeticOperation::Sub;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![2]);
    }

    #[test]
    fn test_div_by_zero() {
        let mut state = setup_test_state();
        state.stack.extend([5, 0]);
        let op = ArithmeticOperation::Div;
        assert!(matches!(op.execute(&mut state), Err(VMError::DivisionByZero)));
    }

    #[test]
    fn test_abs() {
        let mut state = setup_test_state();
        state.stack.push(-5);
        let op = ArithmeticOperation::Abs;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5]);
    }

    #[test]
    fn test_min_max() {
        let mut state = setup_test_state();
        state.stack.extend([5, 3]);
        let op = ArithmeticOperation::Min;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![3]);

        state.stack.extend([5, 3]);
        let op = ArithmeticOperation::Max;
        assert!(op.execute(&mut state).is_ok());
        assert_eq!(state.stack, vec![5]);
    }
}