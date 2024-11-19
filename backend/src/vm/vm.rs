use std::collections::HashMap;
use crate::vm::opcode::OpCode;
use crate::vm::operations::{Operation, VMState};
use crate::vm::{Contract, ExecutionContext, VMError, VMResult}; 
use crate::vm::operations::{
    StackOperation,
    ArithmeticOperation,
    SystemOperation,
    RelationshipOperation,
};
use std::sync::atomic::AtomicU64;

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
        let state = VMState {
            stack: Vec::new(),
            memory: HashMap::new(),
            events: Vec::new(),
            instruction_pointer: 0,
            reputation_context,
            caller_did: String::new(),
            block_number: 1,
            timestamp: 1000,
            permissions: vec![],
            memory_limit: 1024 * 1024, // 1MB default limit
            memory_address_counter: AtomicU64::new(0),
        };
        
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
                if let Some(value) = self.state.stack.pop() {
                    self.state.memory.insert(key.clone(), value);
                }
                Ok(())
            },

            OpCode::Load(key) => {
                if let Some(&value) = self.state.memory.get(key) {
                    self.state.stack.push(value);
                    Ok(())
                } else {
                    Err(VMError::InvalidMemoryAccess)
                }
            },

            OpCode::RecordContribution { description, impact_story, context, tags } => {
                RelationshipOperation::RecordContribution {
                    description: description.clone(),
                    impact_story: impact_story.clone(), 
                    context: context.clone(),
                    tags: tags.clone(),
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
        let reputation = self.state.reputation_context.get(&self.state.caller_did)
            .copied()
            .unwrap_or(0);
            
        if reputation < contract.required_reputation {
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
    pub fn get_events(&self) -> &[crate::vm::event::Event] {
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
}
