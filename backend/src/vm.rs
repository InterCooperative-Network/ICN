use std::sync::Arc;
use icn_types::RuntimeError;
use icn_core::vm::{OpCode, Contract, CooperativeMetadata, VM as CoreVM};
use std::error::Error;
use std::collections::HashMap;

pub mod opcode {
    #[derive(Debug, PartialEq, Clone)]
    pub enum OpCode {
        // Stack operations
        Push(i32),
        Pop,
        Dup,
        Swap,
        
        // Math operations
        Add,
        Sub,
        Mul,
        Div,
        
        // Memory operations
        Load(String),
        Store(String),
        
        // Control flow
        Jump(usize),
        JumpIf(usize),
        Call(String),
        Return,
        
        // Cooperative operations
        ShareResource(String),
        RequestResource(String),
        ReleaseResource(String),
        
        // Federation operations
        JoinFederation(String),
        LeaveFederation(String),
        VoteOnProposal(String),
        
        // Reputation operations
        GetReputation(String),
        IncreaseReputation(String, i32),
        DecreaseReputation(String, i32),
    }
}

pub mod cooperative_metadata {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResourceImpact {
        pub cpu: f64,
        pub memory: f64,
        pub storage: f64,
        pub bandwidth: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CooperativeMetadata {
        pub name: String,
        pub version: String,
        pub author: String,
        pub description: String,
        pub resource_impact: ResourceImpact,
        pub permissions: Vec<String>,
        pub dependencies: HashMap<String, String>, // name -> version
    }
    
    impl CooperativeMetadata {
        pub fn new(
            name: String,
            version: String,
            author: String,
            description: String,
        ) -> Self {
            Self {
                name,
                version,
                author,
                description,
                resource_impact: ResourceImpact {
                    cpu: 0.0,
                    memory: 0.0,
                    storage: 0.0,
                    bandwidth: 0.0,
                },
                permissions: Vec::new(),
                dependencies: HashMap::new(),
            }
        }
        
        pub fn with_resource_impact(mut self, impact: ResourceImpact) -> Self {
            self.resource_impact = impact;
            self
        }
        
        pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
            self.permissions = permissions;
            self
        }
        
        pub fn add_dependency(mut self, name: String, version: String) -> Self {
            self.dependencies.insert(name, version);
            self
        }
    }
}

pub struct VM {
    pub code: Vec<opcode::OpCode>,
    pub stack: Vec<i32>,
    pub memory: HashMap<String, i32>,
    pub ip: usize,
    pub metadata: cooperative_metadata::CooperativeMetadata,
}

impl VM {
    pub fn new(code: Vec<opcode::OpCode>, metadata: cooperative_metadata::CooperativeMetadata) -> Self {
        Self {
            code,
            stack: Vec::new(),
            memory: HashMap::new(),
            ip: 0,
            metadata,
        }
    }
    
    pub fn execute(&mut self) -> Result<i32, Box<dyn Error>> {
        while self.ip < self.code.len() {
            match &self.code[self.ip].clone() {
                opcode::OpCode::Push(val) => {
                    self.stack.push(*val);
                },
                opcode::OpCode::Pop => {
                    self.stack.pop().ok_or("Stack underflow")?;
                },
                opcode::OpCode::Dup => {
                    let val = *self.stack.last().ok_or("Stack underflow")?;
                    self.stack.push(val);
                },
                opcode::OpCode::Swap => {
                    let len = self.stack.len();
                    if len < 2 {
                        return Err("Stack underflow".into());
                    }
                    self.stack.swap(len - 1, len - 2);
                },
                opcode::OpCode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a + b);
                },
                opcode::OpCode::Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a - b);
                },
                opcode::OpCode::Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a * b);
                },
                opcode::OpCode::Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    if b == 0 {
                        return Err("Division by zero".into());
                    }
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a / b);
                },
                opcode::OpCode::Load(var) => {
                    let val = *self.memory.get(var).unwrap_or(&0);
                    self.stack.push(val);
                },
                opcode::OpCode::Store(var) => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    self.memory.insert(var.clone(), val);
                },
                opcode::OpCode::Jump(pos) => {
                    self.ip = *pos;
                    continue; // Skip the ip increment at the end
                },
                opcode::OpCode::JumpIf(pos) => {
                    let condition = self.stack.pop().ok_or("Stack underflow")?;
                    if condition != 0 {
                        self.ip = *pos;
                        continue; // Skip the ip increment at the end
                    }
                },
                opcode::OpCode::Call(_) => {
                    // In a real VM, this would call another program
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::Return => {
                    break;
                },
                opcode::OpCode::ShareResource(_) => {
                    // In a real VM, this would actually share resources
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::RequestResource(_) => {
                    // In a real VM, this would request resources
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::ReleaseResource(_) => {
                    // In a real VM, this would release resources
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::JoinFederation(_) => {
                    // In a real VM, this would join a federation
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::LeaveFederation(_) => {
                    // In a real VM, this would leave a federation
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::VoteOnProposal(_) => {
                    // In a real VM, this would vote on a proposal
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::GetReputation(_) => {
                    // In a real VM, this would get the reputation
                    // For this implementation, we just push 0
                    self.stack.push(0);
                },
                opcode::OpCode::IncreaseReputation(_, _) => {
                    // In a real VM, this would increase reputation
                    // For this implementation, we just do nothing
                },
                opcode::OpCode::DecreaseReputation(_, _) => {
                    // In a real VM, this would decrease reputation
                    // For this implementation, we just do nothing
                },
            }
            
            self.ip += 1;
        }
        
        // Return the top of the stack, or 0 if empty
        Ok(self.stack.last().copied().unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::opcode::OpCode;
    use super::cooperative_metadata::{CooperativeMetadata, ResourceImpact};

    #[test]
    fn test_simple_program() {
        // Create a metadata for the VM
        let metadata = CooperativeMetadata::new(
            "TestProgram".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
            "A test program".to_string(),
        );
        
        // Create a simple program that calculates (5 + 3) * 2
        let program = vec![
            OpCode::Push(5),
            OpCode::Push(3),
            OpCode::Add,     // Stack now contains [8]
            OpCode::Push(2),
            OpCode::Mul,     // Stack now contains [16]
        ];
        
        let mut vm = VM::new(program, metadata);
        let result = vm.execute();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 16);
    }
    
    #[test]
    fn test_memory_operations() {
        // Create a metadata for the VM
        let metadata = CooperativeMetadata::new(
            "MemoryTest".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
            "Tests memory operations".to_string(),
        );
        
        // Create a program that stores and loads values from memory
        let program = vec![
            OpCode::Push(42),
            OpCode::Store("answer".to_string()),  // Store 42 in "answer"
            OpCode::Push(10),                     // Push something else to ensure we're loading
            OpCode::Load("answer".to_string()),   // Load "answer" (42)
        ];
        
        let mut vm = VM::new(program, metadata);
        let result = vm.execute();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_control_flow() {
        // Create a metadata for the VM
        let metadata = CooperativeMetadata::new(
            "ControlFlowTest".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
            "Tests control flow".to_string(),
        );
        
        // Create a program with conditional jumps
        let program = vec![
            OpCode::Push(1),                  // Condition true
            OpCode::JumpIf(3),                // Jump to position 3 if condition is true
            OpCode::Push(0),                  // This should be skipped
            OpCode::Push(42),                 // This is where we jump to
            OpCode::Push(0),                  // Condition false
            OpCode::JumpIf(7),                // This jump should NOT happen
            OpCode::Push(100),                // This should be executed
        ];
        
        let mut vm = VM::new(program, metadata);
        let result = vm.execute();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }
}
