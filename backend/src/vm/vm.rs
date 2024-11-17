// src/vm/vm.rs

use std::collections::HashMap;
use crate::vm::opcode::OpCode;
use crate::vm::operations::Operation;
use crate::vm::operations::{
    StackOperation,
    ArithmeticOperation,
    SystemOperation,
    RelationshipOperation,
    MemoryOperation,
};
use crate::vm::{Contract, ExecutionContext, VMState, VMResult, VMError};
use crate::relationship::RelationshipType;

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
            // Stack Operations
            OpCode::Push(val) => StackOperation::Push(*val).execute(&mut self.state),
            OpCode::Pop => StackOperation::Pop.execute(&mut self.state),
            OpCode::Dup => StackOperation::Dup.execute(&mut self.state),
            OpCode::Swap => StackOperation::Swap.execute(&mut self.state),
            
            // Arithmetic Operations
            OpCode::Add => ArithmeticOperation::Add.execute(&mut self.state),
            OpCode::Sub => ArithmeticOperation::Sub.execute(&mut self.state),
            OpCode::Mul => ArithmeticOperation::Mul.execute(&mut self.state),
            OpCode::Div => ArithmeticOperation::Div.execute(&mut self.state),
            OpCode::Mod => ArithmeticOperation::Mod.execute(&mut self.state),

            // Memory Operations
            OpCode::Store(key) => MemoryOperation::Allocate {
                request: crate::vm::operations::memory::AllocationRequest {
                    size: 64,
                    segment_type: crate::vm::operations::memory::MemorySegment::Scratch,
                    federation_id: None,
                    persistent: false,
                }
            }.execute(&mut self.state),

            // Relationship Operations
            OpCode::RecordContribution { description, impact_story, context, tags } => {
                RelationshipOperation::RecordContribution {
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    context: context.clone(),
                    tags: tags.clone(),
                    witnesses: vec![],
                }.execute(&mut self.state)
            },

            OpCode::RecordMutualAid { description, receiver, impact_story, reciprocity_notes } => {
                RelationshipOperation::RecordMutualAid {
                    recipient_did: receiver.clone(),
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    reciprocity_notes: reciprocity_notes.clone(),
                    tags: vec![],
                }.execute(&mut self.state)
            },

            OpCode::UpdateRelationship { member_two, relationship_type, story } => {
                RelationshipOperation::UpdateRelationship {
                    member_did: member_two.clone(),
                    relationship_type: RelationshipType::new(&relationship_type),
                    story: story.clone(),
                    strength_indicators: vec![],
                }.execute(&mut self.state)
            },

            // System Operations
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_vm() -> VM {
        let mut reputation_context = HashMap::new();
        reputation_context.insert("test_did".to_string(), 100);
        VM::new(1000, reputation_context)
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = setup_test_vm();
        
        assert!(vm.execute_instruction(&OpCode::Push(42)).is_ok());
        assert_eq!(vm.state.stack, vec![42]);

        assert!(vm.execute_instruction(&OpCode::Dup).is_ok());
        assert_eq!(vm.state.stack, vec![42, 42]);

        assert!(vm.execute_instruction(&OpCode::Pop).is_ok());
        assert_eq!(vm.state.stack, vec![42]);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = setup_test_vm();
        
        assert!(vm.execute_instruction(&OpCode::Push(10)).is_ok());
        assert!(vm.execute_instruction(&OpCode::Push(5)).is_ok());
        assert!(vm.execute_instruction(&OpCode::Add).is_ok());
        
        assert_eq!(vm.state.stack, vec![15]);
    }
}