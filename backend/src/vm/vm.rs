// src/vm/vm.rs

use std::collections::HashMap;
use crate::vm::opcode::OpCode;
use crate::vm::contract::Contract;
use crate::vm::execution_context::ExecutionContext;
use crate::vm::cooperative_metadata::CooperativeMetadata;
use crate::vm::event::Event;
use crate::vm::operations::{
    Operation,
    StackOperation,
    ArithmeticOperation,
    CooperativeOperation,
    DataOperation,
    FederationOperation,
    GovernanceOperation,
    MemoryOperation,
    NetworkOperation,
    RelationshipOperation,
    ReputationOperation,
    SystemOperation,
};
use crate::vm::operations::system::LogLevel;
use crate::vm::operations::relationship::RelationType;

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
        let context = self.execution_context.as_ref().ok_or("No execution context".to_string())?;

        let caller_reputation = self
            .reputation_context
            .get(&context.caller_did)
            .copied()
            .unwrap_or(0);
        if caller_reputation < contract.required_reputation {
            return Err("Insufficient reputation to execute contract".to_string());
        }

        for permission in &contract.permissions {
            if !context.permissions.contains(permission) {
                return Err(format!("Missing permission: {}", permission));
            }
        }

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
            OpCode::Store(key) => Box::new(MemoryOperation::Allocate {
                request: crate::vm::operations::memory::AllocationRequest {
                    size: 64,
                    segment_type: crate::vm::operations::memory::MemorySegment::Scratch,
                    federation_id: None,
                    persistent: false,
                }
            }),
            OpCode::Load(key) => Box::new(MemoryOperation::GetMemoryInfo {
                segment_id: key.clone()
            }),
            
            // Cooperative Operations
            OpCode::CreateCooperative => Box::new(CooperativeOperation::CreateCooperative {
                name: metadata.cooperative_id.clone(),
                description: metadata.purpose.clone(),
                resource_policies: HashMap::new(),
                membership_requirements: vec![],
            }),
            OpCode::JoinCooperative => Box::new(CooperativeOperation::JoinCooperative {
                cooperative_id: metadata.cooperative_id.clone(),
                role: "member".to_string(),
                qualifications: vec![],
            }),
            OpCode::LeaveCooperative => Box::new(CooperativeOperation::LeaveCooperative {
                cooperative_id: metadata.cooperative_id.clone(),
                exit_reason: "User initiated".to_string(),
            }),

            OpCode::RecordContribution { description, impact_story, context, tags } => {
                Box::new(RelationshipOperation::RecordContribution {
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    context: context.clone(),
                    tags: tags.clone(),
                    witnesses: vec![],
                })
            },
            OpCode::RecordMutualAid { description, receiver, impact_story, reciprocity_notes, tags } => {
                Box::new(RelationshipOperation::RecordMutualAid {
                    recipient_did: receiver.clone(),
                    description: description.clone(),
                    impact_story: impact_story.clone(),
                    reciprocity_notes: reciprocity_notes.clone(),
                    tags: tags.clone(),
                })
            },
            OpCode::UpdateRelationship { member_two, relationship_type, story, interaction } => {
                Box::new(RelationshipOperation::UpdateRelationship {
                    member_did: member_two.clone(),
                    relationship_type: RelationType::Custom(relationship_type.clone()),
                    story: story.clone(),
                    interaction: interaction.clone(),
                })
            },
            OpCode::AddEndorsement { to_did, content, context, skills } => {
                Box::new(RelationshipOperation::AddEndorsement {
                    member_did: to_did.clone(),
                    endorsement_type: crate::vm::operations::relationship::EndorsementType::Skill,
                    content: content.clone(),
                    context: context.clone(),
                    skills: skills.clone(),
                })
            },

            OpCode::Log(msg) => Box::new(SystemOperation::Log {
                message: msg.clone(),
                level: LogLevel::Info,
                metadata: HashMap::new(),
            }),
            OpCode::Halt => Box::new(SystemOperation::Halt),
            OpCode::Nop => Box::new(SystemOperation::Log {
                message: "No operation".to_string(),
                level: LogLevel::Debug,
                metadata: HashMap::new(),
            }),
        };

        operation.execute(self).map_err(|e| e.to_string())
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

    pub fn get_instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_stack_operations() {
        let mut vm = VM::new(1000, HashMap::new());
        let metadata = CooperativeMetadata::default();

        vm.execute_instruction(&OpCode::Push(42), &metadata).unwrap();
        assert_eq!(vm.get_stack(), &vec![42]);

        vm.execute_instruction(&OpCode::Dup, &metadata).unwrap();
        assert_eq!(vm.get_stack(), &vec![42, 42]);

        vm.execute_instruction(&OpCode::Pop, &metadata).unwrap();
        assert_eq!(vm.get_stack(), &vec![42]);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = VM::new(1000, HashMap::new());
        let metadata = CooperativeMetadata::default();

        vm.execute_instruction(&OpCode::Push(10), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Push(5), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Add, &metadata).unwrap();
        assert_eq!(vm.get_stack(), &vec![15]);
    }

    #[test]
    fn test_memory_operations() {
        let mut vm = VM::new(1000, HashMap::new());
        let metadata = CooperativeMetadata::default();

        vm.execute_instruction(&OpCode::Push(42), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Store("test".to_string()), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Load("test".to_string()), &metadata).unwrap();
    }
}