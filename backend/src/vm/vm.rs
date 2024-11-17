// src/vm/vm.rs

use std::collections::HashMap;
use super::opcode::OpCode;
use super::contract::Contract;
use super::execution_context::ExecutionContext;
use super::cooperative_metadata::CooperativeMetadata;
use super::event::Event;
use super::VMError;
use super::operations::{
    Operation,
    stack::StackOperation,
    arithmetic::ArithmeticOperation,
    cooperative::CooperativeOperation,
    data::DataOperation,
    federation::FederationOperation,
    governance::GovernanceOperation,
    memory::MemoryOperation,
    network::NetworkOperation,
    relationship::RelationshipOperation,
    reputation::ReputationOperation,
    system::SystemOperation,
};

pub struct VM {
    stack: Vec<i64>,
    memory: HashMap<String, i64>,
    execution_context: Option<ExecutionContext>,
    events: Vec<Event>,
    logs: Vec<String>,
    pub reputation_context: HashMap<String, i64>,
    instruction_limit: usize,
    instruction_pointer: usize,
}

impl VM {
    pub fn new(instruction_limit: usize, reputation_context: HashMap<String, i64>) -> Self {
        VM {
            stack: Vec::new(),
            memory: HashMap::new(),
            execution_context: None,
            events: Vec::new(),
            logs: Vec::new(),
            reputation_context,
            instruction_limit,
            instruction_pointer: 0,
        }
    }

    pub fn get_reputation_context(&self) -> &HashMap<String, i64> {
        &self.reputation_context
    }

    pub fn set_execution_context(&mut self, context: ExecutionContext) {
        self.execution_context = Some(context);
    }

    pub fn execute_contract(&mut self, contract: &Contract) -> Result<(), String> {
        // Check execution context
        let context = self.execution_context.as_ref().ok_or("No execution context".to_string())?;

        // Check caller reputation
        let caller_reputation = self
            .reputation_context
            .get(&context.caller_did)
            .copied()
            .unwrap_or(0);
        if caller_reputation < contract.required_reputation {
            return Err("Insufficient reputation to execute contract".to_string());
        }

        // Check permissions
        for permission in &contract.permissions {
            if !context.permissions.contains(permission) {
                return Err(format!("Missing permission: {}", permission));
            }
        }

        // Execute instructions
        let code_len = contract.code.len();
        self.instruction_pointer = 0;

        while self.instruction_pointer < code_len {
            if self.instruction_pointer >= self.instruction_limit {
                return Err("Instruction limit exceeded".to_string());
            }

            let op = &contract.code[self.instruction_pointer];
            self.execute_instruction(op, &contract.cooperative_metadata)?;

            self.instruction_pointer += 1;
        }

        Ok(())
    }

    pub fn execute_instruction(
        &mut self,
        op: &OpCode,
        metadata: &CooperativeMetadata,
    ) -> Result<(), String> {
        let operation: Box<dyn Operation> = match op {
            // Stack Operations
            OpCode::Push(val) => Box::new(StackOperation::Push(*val)),
            OpCode::Pop => Box::new(StackOperation::Pop),
            OpCode::Dup => Box::new(StackOperation::Dup),
            OpCode::Swap => Box::new(StackOperation::Swap),
            
            // Arithmetic Operations
            OpCode::Add => Box::new(ArithmeticOperation::Add),
            OpCode::Sub => Box::new(ArithmeticOperation::Sub),
            OpCode::Mul => Box::new(ArithmeticOperation::Mul),
            OpCode::Div => Box::new(ArithmeticOperation::Div),
            OpCode::Mod => Box::new(ArithmeticOperation::Mod),

            // Memory Operations
            OpCode::Store(key) => Box::new(MemoryOperation::Store(key.clone())),
            OpCode::Load(key) => Box::new(MemoryOperation::Load(key.clone())),
            
            // Cooperative Operations
            OpCode::CreateCooperative => Box::new(CooperativeOperation::CreateCooperative),
            OpCode::JoinCooperative => Box::new(CooperativeOperation::JoinCooperative),
            OpCode::LeaveCooperative => Box::new(CooperativeOperation::LeaveCooperative),

            // Relationship Operations
            OpCode::RecordContribution { description, impact_story, context, tags } => {
                Box::new(RelationshipOperation::RecordContribution {
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    context: context.clone(),
                    tags: tags.clone(),
                })
            },
            OpCode::RecordMutualAid { description, receiver, impact_story, reciprocity_notes, tags } => {
                Box::new(RelationshipOperation::RecordMutualAid {
                    description: description.clone(),
                    receiver: receiver.clone(),
                    impact_story: impact_story.clone(),
                    reciprocity_notes: reciprocity_notes.clone(),
                    tags: tags.clone(),
                })
            },
            OpCode::UpdateRelationship { member_two, relationship_type, story, interaction } => {
                Box::new(RelationshipOperation::UpdateRelationship {
                    member_two: member_two.clone(),
                    relationship_type: relationship_type.clone(),
                    story: story.clone(),
                    interaction: interaction.clone(),
                })
            },
            OpCode::AddEndorsement { to_did, content, context, skills } => {
                Box::new(RelationshipOperation::AddEndorsement {
                    to_did: to_did.clone(),
                    content: content.clone(),
                    context: context.clone(),
                    skills: skills.clone(),
                })
            },

            // System Operations
            OpCode::Log(msg) => Box::new(SystemOperation::Log {
                message: msg.clone(),
                level: super::operations::system::LogLevel::Info,
                metadata: HashMap::new(),
            }),
            OpCode::Halt => Box::new(SystemOperation::Halt),

            OpCode::Nop => Box::new(SystemOperation::Nop),
            _ => return Err("Operation not implemented".to_string()),
        };

        // Execute the operation using the Operation trait
        operation.execute(self).map_err(|e| e.to_string())
    }

    fn emit_event(&mut self, event_type: &str, cooperative_id: String, data: HashMap<String, String>) {
        if let Some(context) = &self.execution_context {
            let event = Event {
                event_type: event_type.to_string(),
                cooperative_id,
                data,
                timestamp: context.timestamp,
            };
            self.events.push(event);
        }
    }

    pub fn get_logs(&self) -> &Vec<String> {
        &self.logs
    }

    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    pub fn get_stack(&self) -> &Vec<i64> {
        &self.stack
    }

    pub fn get_memory(&self) -> &HashMap<String, i64> {
        &self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_vm() -> VM {
        let mut reputation_context = HashMap::new();
        reputation_context.insert("test_caller".to_string(), 100);
        VM::new(1000, reputation_context)
    }

    #[test]
    fn test_basic_stack_operations() {
        let mut vm = setup_test_vm();
        vm.execute_instruction(&OpCode::Push(42), &CooperativeMetadata::default()).unwrap();
        assert_eq!(vm.get_stack(), &vec![42]);

        vm.execute_instruction(&OpCode::Dup, &CooperativeMetadata::default()).unwrap();
        assert_eq!(vm.get_stack(), &vec![42, 42]);

        vm.execute_instruction(&OpCode::Pop, &CooperativeMetadata::default()).unwrap();
        assert_eq!(vm.get_stack(), &vec![42]);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = setup_test_vm();
        vm.execute_instruction(&OpCode::Push(10), &CooperativeMetadata::default()).unwrap();
        vm.execute_instruction(&OpCode::Push(5), &CooperativeMetadata::default()).unwrap();
        vm.execute_instruction(&OpCode::Add, &CooperativeMetadata::default()).unwrap();
        assert_eq!(vm.get_stack(), &vec![15]);
    }

    #[test]
    fn test_memory_operations() {
        let mut vm = setup_test_vm();
        vm.execute_instruction(&OpCode::Push(42), &CooperativeMetadata::default()).unwrap();
        vm.execute_instruction(&OpCode::Store("test".to_string()), &CooperativeMetadata::default()).unwrap();
        vm.execute_instruction(&OpCode::Load("test".to_string()), &CooperativeMetadata::default()).unwrap();
        assert_eq!(vm.get_stack(), &vec![42]);
    }
}