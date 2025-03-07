use crate::vm::contract::Contract;
use crate::vm::opcode::OpCode;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    InvalidOpCode,
    DivisionByZero,
    ExecutionLimit,
    MemoryExceeded,
}

impl Error for VMError {}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::InvalidOpCode => write!(f, "Invalid opcode"),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::ExecutionLimit => write!(f, "Execution limit reached"),
            VMError::MemoryExceeded => write!(f, "Memory limit exceeded"),
        }
    }
}

pub struct VM {
    stack: VecDeque<i64>,
    max_stack_size: usize,
    max_instructions: u64,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            max_stack_size: 1024,
            max_instructions: 10000,
        }
    }
    
    // Execute a contract and return the final result (value on top of stack)
    pub fn execute(&self, contract: &Contract) -> Result<i64, Box<dyn Error>> {
        let mut stack = VecDeque::with_capacity(self.max_stack_size);
        let mut instruction_count = 0;
        
        for op in &contract.code {
            instruction_count += 1;
            if instruction_count > self.max_instructions {
                return Err(Box::new(VMError::ExecutionLimit));
            }
            
            match op {
                OpCode::Push(val) => {
                    if stack.len() >= self.max_stack_size {
                        return Err(Box::new(VMError::MemoryExceeded));
                    }
                    stack.push_back(*val);
                },
                OpCode::Pop => {
                    if stack.is_empty() {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    stack.pop_back();
                },
                OpCode::Add => {
                    if stack.len() < 2 {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    let b = stack.pop_back().unwrap();
                    let a = stack.pop_back().unwrap();
                    stack.push_back(a + b);
                },
                OpCode::Sub => {
                    if stack.len() < 2 {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    let b = stack.pop_back().unwrap();
                    let a = stack.pop_back().unwrap();
                    stack.push_back(a - b);
                },
                OpCode::Mul => {
                    if stack.len() < 2 {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    let b = stack.pop_back().unwrap();
                    let a = stack.pop_back().unwrap();
                    stack.push_back(a * b);
                },
                OpCode::Div => {
                    if stack.len() < 2 {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    let b = stack.pop_back().unwrap();
                    if b == 0 {
                        return Err(Box::new(VMError::DivisionByZero));
                    }
                    let a = stack.pop_back().unwrap();
                    stack.push_back(a / b);
                },
                OpCode::Mod => {
                    if stack.len() < 2 {
                        return Err(Box::new(VMError::StackUnderflow));
                    }
                    let b = stack.pop_back().unwrap();
                    if b == 0 {
                        return Err(Box::new(VMError::DivisionByZero));
                    }
                    let a = stack.pop_back().unwrap();
                    stack.push_back(a % b);
                },
                // For simplicity, we're implementing only the opcodes needed for our test
                _ => {
                    return Err(Box::new(VMError::InvalidOpCode));
                }
            }
        }
        
        // Return the top value on the stack, or 0 if stack is empty
        Ok(stack.pop_back().unwrap_or(0))
    }
}