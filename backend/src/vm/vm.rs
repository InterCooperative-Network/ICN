// src/vm/vm.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::vm::opcode::OpCode;
use crate::vm::operations::{Operation, VMResult};
use crate::vm::operations::{
    StackOperation,
    ArithmeticOperation,
    SystemOperation,
    RelationshipOperation,
    MemoryOperation,
};
use crate::vm::{Contract, ExecutionContext, VMState, VMError, Event};
use crate::vm::operations::relationship::{RelationshipType};

/// Virtual Machine implementation for executing cooperative operations
pub struct VM {
    /// Current state of the virtual machine
    state: VMState,
    /// Maximum number of instructions that can be executed
    instruction_limit: usize,
    /// Current instruction pointer
    instruction_pointer: usize,
}

impl VM {
    /// Creates a new VM instance
    pub fn new(instruction_limit: usize, reputation_context: HashMap<String, i64>) -> Self {
        let mut state = VMState::new(String::new(), 1, 1000);
        state.reputation_context = reputation_context;
        VM {
            state,
            instruction_limit,
            instruction_pointer: 0,
        }
    }

    /// Sets the execution context for the VM
    pub fn set_execution_context(&mut self, context: ExecutionContext) {
        self.state.caller_did = context.caller_did;
        self.state.block_number = context.block_number;
        self.state.timestamp = context.timestamp;
        self.state.permissions = context.permissions;
    }

    /// Executes a smart contract
    pub fn execute_contract(&mut self, contract: &Contract) -> VMResult<()> {
        // Validate contract
        if !self.validate_contract(contract)? {
            return Err(VMError::ValidationError);
        }

        // Reset instruction pointer
        self.instruction_pointer = 0;

        // Execute each instruction
        while self.instruction_pointer < contract.code.len() {
            if self.instruction_pointer >= self.instruction_limit {
                return Err(VMError::ExecutionLimitExceeded);
            }

            let op = &contract.code[self.instruction_pointer];
            self.execute_instruction(op)?;

            self.instruction_pointer += 1;
        }

        Ok(())
    }

    /// Executes a single instruction
    pub fn execute_instruction(&mut self, op: &OpCode) -> VMResult<()> {
        match op {
            OpCode::Push(val) => StackOperation::Push(*val).execute(&mut self.state),
            OpCode::Pop => StackOperation::Pop.execute(&mut self.state),
            OpCode::Dup => StackOperation::Dup.execute(&mut self.state),
            OpCode::Swap => StackOperation::Swap.execute(&mut self.state),
            
            OpCode::Add => ArithmeticOperation::Add.execute(&mut self.state),
            OpCode::Sub => ArithmeticOperation::Sub.execute(&mut self.state),
            OpCode::Mul => ArithmeticOperation::Mul.execute(&mut self.state),
            OpCode::Div => ArithmeticOperation::Div.execute(&mut self.state),
            OpCode::Mod => ArithmeticOperation::Mod.execute(&mut self.state),

            OpCode::Store(key) => {
                let request = crate::vm::operations::memory::AllocationRequest {
                    size: 64,
                    segment_type: crate::vm::operations::memory::MemorySegment::Scratch,
                    federation_id: None,
                    persistent: false,
                };
                MemoryOperation::Allocate { request }.execute(&mut self.state)
            },

            OpCode::RecordContribution { description, impact_story, context, tags } => {
                RelationshipOperation::RecordContribution {
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    context: context.clone(),
                    tags: tags.clone(),
                }.execute(&mut self.state)
            },

            OpCode::RecordMutualAid { description, receiver, impact_story, reciprocity_notes } => {
                RelationshipOperation::RecordMutualAid {
                    description: description.clone(),
                    receiver_did: receiver.clone(),
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    reciprocity_notes: reciprocity_notes.clone(),
                    tags: vec![],
                }.execute(&mut self.state)
            },

            OpCode::UpdateRelationship { member_two, relationship_type, story } => {
                RelationshipOperation::UpdateRelationship {
                    member_two: member_two.clone(),
                    relationship_type: relationship_type.clone(),
                    story: story.clone(),
                    interaction: None,
                }.execute(&mut self.state)
            },

            OpCode::Log(msg) => SystemOperation::Log {
                message: msg.clone(),
                level: crate::vm::operations::system::LogLevel::Info,
                metadata: HashMap::new(),
            }.execute(&mut self.state),

            OpCode::Halt => SystemOperation::Halt.execute(&mut self.state),
            OpCode::Nop => Ok(()),
            
            _ => Err(VMError::InvalidOperand),
        }
    }

    /// Validates a contract before execution
    fn validate_contract(&self, contract: &Contract) -> VMResult<bool> {
        // Check reputation requirement
        if self.state.get_reputation() < contract.required_reputation {
            return Ok(false);
        }

        // Check permissions
        for permission in &contract.permissions {
            if !self.state.permissions.contains(permission) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Gets the current VM state
    pub fn get_state(&self) -> &VMState {
        &self.state
    }

    /// Gets events from the current execution
    pub fn get_events(&self) -> &[Event] {
        &self.state.events
    }

    /// Gets the current reputation context
    pub fn get_reputation_context(&self) -> &HashMap<String, i64> {
        &self.state.reputation_context
    }

    /// Gets number of instructions executed
    pub fn get_instruction_count(&self) -> usize {
        self.instruction_pointer
    }

    /// Gets the memory stack
    pub fn get_stack(&self) -> &[i64] {
        &self.state.stack
    }

    /// Gets the memory heap
    pub fn get_memory(&self) -> &HashMap<String, i64> {
        &self.state.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::Event;

    fn setup_test_contract() -> Contract {
        Contract {
            id: "test".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(20),
                OpCode::Add,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: Default::default(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    #[test]
    fn test_contract_execution() {
        let mut reputation_context = HashMap::new();
        reputation_context.insert("test_caller".to_string(), 100);
        
        let mut vm = VM::new(1000, reputation_context);
        let context = ExecutionContext {
            caller_did: "test_caller".to_string(),
            cooperative_id: "test_coop".to_string(),
            timestamp: 1000,
            block_number: 1,
            reputation_score: 100,
            permissions: vec![],
        };
        
        vm.set_execution_context(context);
        
        let contract = setup_test_contract();
        assert!(vm.execute_contract(&contract).is_ok());
        assert_eq!(vm.get_stack(), &[30]); // 10 + 20 = 30
    }

    #[test]
    fn test_instruction_limit() {
        let mut vm = VM::new(2, HashMap::new());
        let contract = Contract {
            id: "test".to_string(),
            code: vec![
                OpCode::Push(1),
                OpCode::Push(2),
                OpCode::Add, // This should exceed the limit
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: Default::default(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        assert!(matches!(
            vm.execute_contract(&contract),
            Err(VMError::ExecutionLimitExceeded)
        ));
    }

    #[test]
    fn test_relationship_operations() {
        let mut vm = VM::new(1000, HashMap::new());
        let context = ExecutionContext {
            caller_did: "test_caller".to_string(),
            cooperative_id: "test_coop".to_string(),
            timestamp: 1000,
            block_number: 1,
            reputation_score: 100,
            permissions: vec!["relationship.update".to_string()],
        };
        vm.set_execution_context(context);

        let contract = Contract {
            id: "test".to_string(),
            code: vec![
                OpCode::UpdateRelationship {
                    member_two: "other_member".to_string(),
                    relationship_type: "Collaboration".to_string(),
                    story: "Working together".to_string(),
                },
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: Default::default(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        assert!(vm.execute_contract(&contract).is_ok());
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = VM::new(1000, HashMap::new());
        let contract = Contract {
            id: "test".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(5),
                OpCode::Sub,
                OpCode::Push(2),
                OpCode::Mul,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: Default::default(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        assert!(vm.execute_contract(&contract).is_ok());
        assert_eq!(vm.get_stack(), &[10]); // (10 - 5) * 2 = 10
    }
}