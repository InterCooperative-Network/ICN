use std::sync::Arc;
use icn_types::RuntimeError;
use icn_core::vm::{OpCode, Contract, CooperativeMetadata, VM as CoreVM};
use std::error::Error;
use std::collections::HashMap;
use icn_core::vm::{VM, VMConfig};

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

pub struct BackendVM {
    inner: VM
}

impl BackendVM {
    pub fn new(config: VMConfig) -> Self {
        Self {
            inner: VM::new(config)
        }
    }

    pub fn execute(&mut self, contract_id: &str, method: &str, args: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
        self.inner.execute(contract_id, method, args)
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
        
        let mut vm = BackendVM::new(VMConfig { code: program, metadata });
        let result = vm.execute("contract_id", "method", vec![]);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![16]);
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
        
        let mut vm = BackendVM::new(VMConfig { code: program, metadata });
        let result = vm.execute("contract_id", "method", vec![]);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![42]);
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
        
        let mut vm = BackendVM::new(VMConfig { code: program, metadata });
        let result = vm.execute("contract_id", "method", vec![]);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![100]);
    }
}
