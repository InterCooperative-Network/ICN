use std::collections::HashMap;
use serde::{Serialize, Deserialize};
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    // Stack Operations
    Push,
    Pop,
    Dup,      
    Swap,     
    
    // Arithmetic Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    
    // Memory Operations
    Store,    
    Load,     
    
    // Control Flow
    Call,     
    Return,   
    Jump,     
    JumpIf,   
    
    // Cooperative Operations
    CooperativeAction,
    CreateCooperative,
    JoinCooperative,
    LeaveCooperative,
    AllocateResource,
    TransferResource,
    UpdateCooperativeMetadata,
    AddCooperativeMember,
    RemoveCooperativeMember,
    SetMemberRole,
    
    // Resource Management
    CreateResource,
    AssignResource,
    RevokeResource,
    ShareResource,
    CheckResourceAvailability,
    
    // Governance Operations
    CreateProposal,
    CastVote,
    DelegateVotes,
    ExecuteProposal,
    CalculateVotingWeight,
    CancelProposal,
    ExtendVotingPeriod,
    UpdateQuorum,
    
    // Reputation Operations
    UpdateReputation,
    GetReputation,
    TransferReputation,
    BurnReputation,
    MintReputation,
    SetReputationContext,
    MergeReputationContexts,
    
    // Identity Operations
    VerifyDID,
    UpdateDIDDocument,
    CreateCredential,
    VerifyCredential,
    RevokeCredential,
    
    // Federation Operations
    InitiateFederation,
    JoinFederation,
    LeaveFederation,
    SyncFederationState,
    ValidateFederationAction,
    
    // Transaction Operations
    CreateTransaction,
    ValidateTransaction,
    SignTransaction,
    BroadcastTransaction,
    
    // System Operations
    Log,      
    Halt,     
    EmitEvent,
    GetBlockNumber,
    GetTimestamp,
    GetCaller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    pub cpu_intensity: u32,
    pub memory_usage: u32,
    pub network_usage: u32,
    pub storage_usage: u32,
    pub bandwidth_usage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeMetadata {
    pub creator_did: String,
    pub cooperative_id: String,
    pub purpose: String,
    pub resource_impact: ResourceImpact,
    pub federation_id: Option<String>,
    pub creation_timestamp: u64,
    pub last_updated: u64,
    pub member_count: u32,
    pub resource_allocation: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub code: Vec<(OpCode, Option<i64>)>,
    pub state: HashMap<String, i64>,
    pub required_reputation: i64,
    pub cooperative_metadata: CooperativeMetadata,
    pub version: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub caller_did: String,
    pub cooperative_id: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub federation_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: String,
    pub cooperative_id: String,
    pub data: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<i64>,
    memory: HashMap<String, i64>,
    program_counter: usize,
    contribution_credits: u64,
    reputation_context: HashMap<String, i64>,
    execution_context: Option<ExecutionContext>,
    logs: Vec<String>,
    events: Vec<Event>,
    state: HashMap<String, Vec<u8>>,
}

impl VM {
    pub fn new(contribution_limit: u64, reputation_context: HashMap<String, i64>) -> Self {
        VM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program_counter: 0,
            contribution_credits: contribution_limit,
            reputation_context,
            execution_context: None,
            logs: Vec::new(),
            events: Vec::new(),
            state: HashMap::new(),
        }
    }

    pub fn set_execution_context(&mut self, context: ExecutionContext) {
        self.execution_context = Some(context);
    }

    pub fn execute_contract(&mut self, contract: &Contract) -> Result<i64, String> {
        // Verify reputation requirements
        if let Some(context) = &self.execution_context {
            let caller_reputation = self.reputation_context
                .get(&context.caller_did)
                .copied()
                .unwrap_or(0);
            
            if caller_reputation < contract.required_reputation {
                return Err("Insufficient reputation to execute contract".to_string());
            }
        }

        while self.program_counter < contract.code.len() {
            let (op, value) = &contract.code[self.program_counter];
            self.execute_instruction(op, *value, &contract.cooperative_metadata)?;
            self.program_counter += 1;

            // Check contribution credits
            if self.contribution_credits == 0 {
                return Err("Out of contribution credits".to_string());
            }
            self.contribution_credits -= 1;
        }

        Ok(self.stack.pop().unwrap_or(0))
    }

    pub fn execute_instruction(&mut self, op: &OpCode, value: Option<i64>, metadata: &CooperativeMetadata) -> Result<(), String> {
        match op {
            // Stack Operations
            OpCode::Push => {
                if let Some(v) = value {
                    self.stack.push(v);
                }
                Ok(())
            }
            OpCode::Pop => {
                self.stack.pop().ok_or("Stack underflow")?;
                Ok(())
            }
            OpCode::Dup => {
                if let Some(&v) = self.stack.last() {
                    self.stack.push(v);
                    Ok(())
                } else {
                    Err("Stack underflow".to_string())
                }
            }
            OpCode::Swap => {
                if self.stack.len() >= 2 {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                    Ok(())
                } else {
                    Err("Stack underflow".to_string())
                }
            }

            // Arithmetic Operations
            OpCode::Add => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a + b);
                Ok(())
            }
            OpCode::Sub => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a - b);
                Ok(())
            }
            OpCode::Mul => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a * b);
                Ok(())
            }
            OpCode::Div => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(a / b);
                Ok(())
            }
            OpCode::Mod => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                if b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                self.stack.push(a % b);
                Ok(())
            }

            // Memory Operations
            OpCode::Store => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let key = self.stack.pop().ok_or("Stack underflow")?;
                self.memory.insert(key.to_string(), value);
                Ok(())
            }
            OpCode::Load => {
                let key = self.stack.pop().ok_or("Stack underflow")?;
                let value = self.memory.get(&key.to_string()).copied().unwrap_or(0);
                self.stack.push(value);
                Ok(())
            }

            // Cooperative Operations
            OpCode::CreateCooperative => self.handle_create_cooperative(metadata),
            OpCode::JoinCooperative => self.handle_join_cooperative(metadata),
            OpCode::AllocateResource => self.handle_allocate_resource(value, metadata),
            OpCode::LeaveCooperative => self.handle_leave_cooperative(metadata),

            // Governance Operations
            OpCode::CreateProposal => self.handle_create_proposal(metadata),
            OpCode::CastVote => self.handle_cast_vote(value, metadata),
            OpCode::CalculateVotingWeight => self.handle_calculate_voting_weight(value, metadata),

            // Federation Operations
            OpCode::InitiateFederation => self.handle_federation_operation(op, metadata),
            OpCode::JoinFederation => self.handle_federation_operation(op, metadata),

            // Resource Operations
            OpCode::CreateResource => self.handle_resource_operation(op, value, metadata),
            OpCode::ShareResource => self.handle_resource_operation(op, value, metadata),

            // System Operations
            OpCode::Log => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.logs.push(format!("Log: {}", value));
                Ok(())
            }
            OpCode::Halt => Ok(()),
            OpCode::EmitEvent => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event("CustomEvent", metadata.cooperative_id.clone(), 
                    HashMap::from([("value".to_string(), value.to_string())]));
                Ok(())
            }

            _ => Err(format!("Opcode {:?} not implemented", op)),
        }
    }

    fn handle_create_cooperative(&mut self, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            self.emit_event(
                "CooperativeCreated",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("creator".to_string(), context.caller_did.clone()),
                    ("purpose".to_string(), metadata.purpose.clone()),
                ])
            );
            Ok(())
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_join_cooperative(&mut self, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            self.emit_event(
                "CooperativeJoined",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("member".to_string(), context.caller_did.clone()),
                ])
            );
            Ok(())
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_leave_cooperative(&mut self, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            self.emit_event(
                "CooperativeLeft",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("member".to_string(), context.caller_did.clone()),
                ])
            );
            Ok(())
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_allocate_resource(&mut self, resource_id: Option<i64>, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(id) = resource_id {
            // Check resource impact
            if metadata.resource_impact.cpu_intensity > 80 || 
               metadata.resource_impact.memory_usage > 80 || 
               metadata.resource_impact.network_usage > 80 {
                return Err("Resource impact too high".to_string());
            }

            self.emit_event(
                "ResourceAllocated",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("resource_id".to_string(), id.to_string()),
                ])
            );
            self.memory.insert(format!("resource_{}", id), 1);
            Ok(())
        } else {
            Err("Missing resource ID".to_string())
        }
    }

    fn handle_create_proposal(&mut self, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            let caller_reputation = self.reputation_context
                .get(&context.caller_did)
                .copied()
                .unwrap_or(0);
            
            if caller_reputation < 100 {
                return Err("Insufficient reputation to create proposal".to_string());
            }

            self.emit_event(
                "ProposalCreated",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("creator".to_string(), context.caller_did.clone()),
                ])
            );
            Ok(())
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_cast_vote(&mut self, vote_value: Option<i64>, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            if let Some(value) = vote_value {
                self.emit_event(
                    "VoteCast",
                    metadata.cooperative_id.clone(),
                    HashMap::from([
                        ("voter".to_string(), context.caller_did.clone()),
                        ("vote".to_string(), value.to_string()),
                    ])
                );
                Ok(())
            } else {
                Err("Missing vote value".to_string())
            }
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_calculate_voting_weight(&mut self, _weight_id: Option<i64>, metadata: &CooperativeMetadata) -> Result<(), String> {
        if let Some(context) = &self.execution_context {
            let reputation = self.reputation_context
                .get(&context.caller_did)
                .copied()
                .unwrap_or(0);
            
            let weight = (reputation as f64 * 0.7 + 100.0) as i64;
            
            self.stack.push(weight);
            self.emit_event(
                "VotingWeightCalculated",
                metadata.cooperative_id.clone(),
                HashMap::from([
                    ("member".to_string(), context.caller_did.clone()),
                    ("weight".to_string(), weight.to_string()),
                ])
            );
            Ok(())
        } else {
            Err("No execution context".to_string())
        }
    }

    fn handle_federation_operation(&mut self, op: &OpCode, metadata: &CooperativeMetadata) -> Result<(), String> {
        match op {
            OpCode::InitiateFederation => {
                if let Some(context) = &self.execution_context {
                    self.emit_event(
                        "FederationInitiated",
                        metadata.cooperative_id.clone(),
                        HashMap::from([
                            ("initiator".to_string(), context.caller_did.clone()),
                        ])
                    );
                    Ok(())
                } else {
                    Err("No execution context".to_string())
                }
            },
            OpCode::JoinFederation => {
                if let Some(fed_id) = &metadata.federation_id {
                    if let Some(context) = &self.execution_context {
                        self.emit_event(
                            "FederationJoined",
                            metadata.cooperative_id.clone(),
                            HashMap::from([
                                ("federation_id".to_string(), fed_id.clone()),
                                ("member".to_string(), context.caller_did.clone()),
                            ])
                        );
                        Ok(())
                    } else {
                        Err("No execution context".to_string())
                    }
                } else {
                    Err("No federation ID specified".to_string())
                }
            },
            _ => Err("Federation operation not implemented".to_string())
        }
    }

    fn handle_resource_operation(&mut self, op: &OpCode, value: Option<i64>, metadata: &CooperativeMetadata) -> Result<(), String> {
        match op {
            OpCode::CreateResource => {
                if let Some(resource_id) = value {
                    if metadata.resource_impact.cpu_intensity > 80 || 
                       metadata.resource_impact.memory_usage > 80 {
                        return Err("Resource impact too high".to_string());
                    }
                    
                    self.emit_event(
                        "ResourceCreated",
                        metadata.cooperative_id.clone(),
                        HashMap::from([
                            ("resource_id".to_string(), resource_id.to_string()),
                        ])
                    );
                    Ok(())
                } else {
                    Err("Missing resource ID".to_string())
                }
            },
            OpCode::ShareResource => {
                if let Some(resource_id) = value {
                    if let Some(context) = &self.execution_context {
                        self.emit_event(
                            "ResourceShared",
                            metadata.cooperative_id.clone(),
                            HashMap::from([
                                ("resource_id".to_string(), resource_id.to_string()),
                                ("shared_by".to_string(), context.caller_did.clone()),
                            ])
                        );
                        Ok(())
                    } else {
                        Err("No execution context".to_string())
                    }
                } else {
                    Err("Missing resource ID".to_string())
                }
            },
            _ => Err("Resource operation not implemented".to_string())
        }
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

    fn create_test_metadata() -> CooperativeMetadata {
        CooperativeMetadata {
            creator_did: "did:icn:test".to_string(),
            cooperative_id: "test-coop".to_string(),
            purpose: "Test Cooperative".to_string(),
            resource_impact: ResourceImpact {
                cpu_intensity: 10,
                memory_usage: 10,
                network_usage: 10,
                storage_usage: 10,
                bandwidth_usage: 10,
            },
            federation_id: None,
            creation_timestamp: 1635724800,
            last_updated: 1635724800,
            member_count: 1,
            resource_allocation: HashMap::new(),
        }
    }

    fn create_test_context() -> ExecutionContext {
        ExecutionContext {
            caller_did: "did:icn:test".to_string(),
            cooperative_id: "test-coop".to_string(),
            block_number: 1,
            timestamp: 1635724800,
            federation_context: None,
        }
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let metadata = create_test_metadata();

        // Test Push and Pop
        vm.execute_instruction(&OpCode::Push, Some(42), &metadata).unwrap();
        assert_eq!(vm.get_stack().last(), Some(&42));

        // Test Dup
        vm.execute_instruction(&OpCode::Dup, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().len(), 2);
        assert_eq!(vm.get_stack().last(), Some(&42));

        // Test Swap
        vm.execute_instruction(&OpCode::Push, Some(24), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Swap, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().len(), 3);
        assert_eq!(vm.get_stack()[vm.get_stack().len() - 1], 42);
        assert_eq!(vm.get_stack()[vm.get_stack().len() - 2], 24);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let metadata = create_test_metadata();

        // Setup stack for arithmetic
        vm.execute_instruction(&OpCode::Push, Some(10), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Push, Some(5), &metadata).unwrap();

        // Test Add
        vm.execute_instruction(&OpCode::Add, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().last(), Some(&15));

        // Test Sub
        vm.execute_instruction(&OpCode::Push, Some(3), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Sub, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().last(), Some(&12));

        // Test Mul
        vm.execute_instruction(&OpCode::Push, Some(4), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Mul, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().last(), Some(&48));

        // Test Div
        vm.execute_instruction(&OpCode::Push, Some(6), &metadata).unwrap();
        vm.execute_instruction(&OpCode::Div, None, &metadata).unwrap();
        assert_eq!(vm.get_stack().last(), Some(&8));
    }

    #[test]
    fn test_cooperative_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let metadata = create_test_metadata();
        vm.set_execution_context(create_test_context());

        // Test CreateCooperative
        vm.execute_instruction(&OpCode::CreateCooperative, None, &metadata).unwrap();
        assert!(!vm.get_events().is_empty());
        assert_eq!(vm.get_events()[0].event_type, "CooperativeCreated");

        // Test JoinCooperative
        vm.execute_instruction(&OpCode::JoinCooperative, None, &metadata).unwrap();
        assert_eq!(vm.get_events()[1].event_type, "CooperativeJoined");
    }

    #[test]
    fn test_resource_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let metadata = create_test_metadata();
        vm.set_execution_context(create_test_context());

        // Test CreateResource
        vm.execute_instruction(&OpCode::CreateResource, Some(1), &metadata).unwrap();
        assert!(!vm.get_events().is_empty());
        assert_eq!(vm.get_events()[0].event_type, "ResourceCreated");

        // Test ShareResource
        vm.execute_instruction(&OpCode::ShareResource, Some(1), &metadata).unwrap();
        assert_eq!(vm.get_events()[1].event_type, "ResourceShared");
    }

    #[test]
    fn test_federation_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let mut metadata = create_test_metadata();
        metadata.federation_id = Some("fed1".to_string());
        vm.set_execution_context(create_test_context());

        // Test InitiateFederation
        vm.execute_instruction(&OpCode::InitiateFederation, None, &metadata).unwrap();
        assert!(!vm.get_events().is_empty());
        assert_eq!(vm.get_events()[0].event_type, "FederationInitiated");

        // Test JoinFederation
        vm.execute_instruction(&OpCode::JoinFederation, None, &metadata).unwrap();
        assert_eq!(vm.get_events()[1].event_type, "FederationJoined");
    }

    #[test]
    fn test_governance_operations() {
        let mut vm = VM::new(100, HashMap::new());
        let metadata = create_test_metadata();
        let mut reputation_context = HashMap::new();
        reputation_context.insert("did:icn:test".to_string(), 150);
        vm = VM::new(100, reputation_context);
        vm.set_execution_context(create_test_context());

        // Test CreateProposal
        vm.execute_instruction(&OpCode::CreateProposal, None, &metadata).unwrap();
        assert!(!vm.get_events().is_empty());
        assert_eq!(vm.get_events()[0].event_type, "ProposalCreated");

        // Test CastVote
        vm.execute_instruction(&OpCode::CastVote, Some(1), &metadata).unwrap();
        assert_eq!(vm.get_events()[1].event_type, "VoteCast");

        // Test CalculateVotingWeight
        vm.execute_instruction(&OpCode::CalculateVotingWeight, None, &metadata).unwrap();
        assert_eq!(vm.get_events()[2].event_type, "VotingWeightCalculated");
    }
}