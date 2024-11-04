use std::collections::HashMap;
use super::opcode::OpCode;
use super::contract::Contract;
use super::execution_context::ExecutionContext;
use super::cooperative_metadata::CooperativeMetadata;
use super::event::Event;

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
        match op {
            OpCode::Push(val) => {
                self.stack.push(*val);
                Ok(())
            }
            OpCode::Pop => {
                self.stack.pop().ok_or("Stack underflow".to_string())?;
                Ok(())
            }
            OpCode::Dup => {
                let val = *self.stack.last().ok_or("Stack underflow")?;
                self.stack.push(val);
                Ok(())
            }
            OpCode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err("Not enough values on the stack to swap".to_string());
                }
                self.stack.swap(len - 1, len - 2);
                Ok(())
            }
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
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a / b);
                Ok(())
            }
            OpCode::Mod => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                if b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a % b);
                Ok(())
            }
            OpCode::Store(key) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.memory.insert(key.clone(), value);
                Ok(())
            }
            OpCode::Load(key) => {
                let value = self
                    .memory
                    .get(key)
                    .copied()
                    .ok_or("Key not found in memory".to_string())?;
                self.stack.push(value);
                Ok(())
            }
            OpCode::Jump(target) => {
                if *target >= self.instruction_limit {
                    return Err("Jump target out of bounds".to_string());
                }
                self.instruction_pointer = *target;
                Ok(())
            }
            OpCode::JumpIf(target) => {
                let condition = self.stack.pop().ok_or("Stack underflow")?;
                if condition != 0 {
                    if *target >= self.instruction_limit {
                        return Err("Jump target out of bounds".to_string());
                    }
                    self.instruction_pointer = *target;
                }
                Ok(())
            }
            OpCode::CreateCooperative => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                if !context.permissions.contains(&"cooperative.create".to_string()) {
                    return Err("Permission denied: cooperative.create".to_string());
                }
                let reputation = self.reputation_context
                    .get(&context.caller_did)
                    .copied()
                    .unwrap_or(0);
                if reputation < 100 {
                    return Err("Insufficient reputation to create cooperative".to_string());
                }
                self.emit_event(
                    "CooperativeCreated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::JoinCooperative => {
                let _context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                self.emit_event(
                    "CooperativeJoined",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::LeaveCooperative => {
                let _context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                self.emit_event(
                    "CooperativeLeft",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::AllocateResource => {
                let resource_amount = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "ResourceAllocated",
                    metadata.cooperative_id.clone(),
                    [("amount".to_string(), resource_amount.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::TransferResource => {
                let resource_amount = self.stack.pop().ok_or("Stack underflow")?;
                let to_member_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "ResourceTransferred",
                    metadata.cooperative_id.clone(),
                    [
                        ("amount".to_string(), resource_amount.to_string()),
                        ("to_member".to_string(), to_member_id.to_string()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                );
                Ok(())
            }
            OpCode::UpdateCooperativeMetadata => {
                Ok(())
            }
            OpCode::AddCooperativeMember => {
                let member_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "CooperativeMemberAdded",
                    metadata.cooperative_id.clone(),
                    [("member_id".to_string(), member_id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::RemoveCooperativeMember => {
                let member_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "CooperativeMemberRemoved",
                    metadata.cooperative_id.clone(),
                    [("member_id".to_string(), member_id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::SetMemberRole => {
                let role = self.stack.pop().ok_or("Stack underflow")?;
                let member_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "MemberRoleSet",
                    metadata.cooperative_id.clone(),
                    [
                        ("member_id".to_string(), member_id.to_string()),
                        ("role".to_string(), role.to_string()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                );
                Ok(())
            }
            OpCode::CreateProposal => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                if !context.permissions.contains(&"proposal.create".to_string()) {
                    return Err("Permission denied: proposal.create".to_string());
                }
                self.emit_event(
                    "ProposalCreated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::CastVote => {
                let vote_value = self.stack.pop().ok_or("Stack underflow")?;
                let proposal_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "VoteCast",
                    metadata.cooperative_id.clone(),
                    [
                        ("proposal_id".to_string(), proposal_id.to_string()),
                        ("vote_value".to_string(), vote_value.to_string()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                );
                Ok(())
            }
            OpCode::DelegateVotes => {
                let delegate_to_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "VotesDelegated",
                    metadata.cooperative_id.clone(),
                    [("delegate_to".to_string(), delegate_to_id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::ExecuteProposal => {
                let proposal_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "ProposalExecuted",
                    metadata.cooperative_id.clone(),
                    [("proposal_id".to_string(), proposal_id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::CancelProposal => {
                let proposal_id = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "ProposalCancelled",
                    metadata.cooperative_id.clone(),
                    [("proposal_id".to_string(), proposal_id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::ExtendVotingPeriod => {
                let proposal_id = self.stack.pop().ok_or("Stack underflow")?;
                let additional_time = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "VotingPeriodExtended",
                    metadata.cooperative_id.clone(),
                    [
                        ("proposal_id".to_string(), proposal_id.to_string()),
                        ("additional_time".to_string(), additional_time.to_string()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                );
                Ok(())
            }
            OpCode::UpdateQuorum => {
                let new_quorum = self.stack.pop().ok_or("Stack underflow")?;
                self.emit_event(
                    "QuorumUpdated",
                    metadata.cooperative_id.clone(),
                    [("new_quorum".to_string(), new_quorum.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::CalculateVotingWeight => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                let reputation = self.reputation_context
                    .get(&context.caller_did)
                    .copied()
                    .unwrap_or(0);
                self.stack.push(reputation);
                Ok(())
            }
            OpCode::UpdateReputation(amount) => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                let reputation = self.reputation_context
                    .entry(context.caller_did.clone())
                    .or_insert(0);
                *reputation += *amount;
                self.emit_event(
                    "ReputationUpdated",
                    metadata.cooperative_id.clone(),
                    [("amount".to_string(), amount.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::GetReputation => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                let reputation = self.reputation_context
                    .get(&context.caller_did)
                    .copied()
                    .unwrap_or(0);
                self.stack.push(reputation);
                Ok(())
            }
            OpCode::TransferReputation => {
                let amount = self.stack.pop().ok_or("Stack underflow")?;
                let to_did_hash = self.stack.pop().ok_or("Stack underflow")?;
                let to_did = self.reverse_hash_did(to_did_hash);
                let from_context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;

                    let from_reputation = self
                    .reputation_context
                    .entry(from_context.caller_did.clone())
                    .or_insert(0);
                if *from_reputation < amount {
                    return Err("Insufficient reputation to transfer".to_string());
                }

                *from_reputation -= amount;
                let to_reputation = self.reputation_context.entry(to_did).or_insert(0);
                *to_reputation += amount;

                self.emit_event(
                    "ReputationTransferred",
                    metadata.cooperative_id.clone(),
                    [("amount".to_string(), amount.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::BurnReputation => {
                let amount = self.stack.pop().ok_or("Stack underflow")?;
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                let reputation = self.reputation_context
                    .entry(context.caller_did.clone())
                    .or_insert(0);
                if *reputation < amount {
                    return Err("Insufficient reputation to burn".to_string());
                }
                *reputation -= amount;
                self.emit_event(
                    "ReputationBurned",
                    metadata.cooperative_id.clone(),
                    [("amount".to_string(), amount.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::MintReputation => {
                let amount = self.stack.pop().ok_or("Stack underflow")?;
                let to_did_hash = self.stack.pop().ok_or("Stack underflow")?;
                let to_did = self.reverse_hash_did(to_did_hash);
                let to_reputation = self.reputation_context.entry(to_did).or_insert(0);
                *to_reputation += amount;
                self.emit_event(
                    "ReputationMinted",
                    metadata.cooperative_id.clone(),
                    [("amount".to_string(), amount.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                );
                Ok(())
            }
            OpCode::VerifyDID => {
                self.stack.push(1); // 1 for true
                Ok(())
            }
            OpCode::UpdateDIDDocument => {
                self.emit_event(
                    "DIDDocumentUpdated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::CreateCredential => {
                self.emit_event(
                    "CredentialCreated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::VerifyCredential => {
                self.stack.push(1); // Assume credential is valid
                Ok(())
            }
            OpCode::RevokeCredential => {
                self.emit_event(
                    "CredentialRevoked",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::InitiateFederation => {
                self.emit_event(
                    "FederationInitiated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::JoinFederation => {
                self.emit_event(
                    "FederationJoined",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::LeaveFederation => {
                self.emit_event(
                    "FederationLeft",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::SyncFederationState => {
                self.emit_event(
                    "FederationStateSynced",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::ValidateFederationAction => {
                self.stack.push(1); // Assume action is valid
                Ok(())
            }
            OpCode::CreateTransaction => {
                self.emit_event(
                    "TransactionCreated",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::ValidateTransaction => {
                self.stack.push(1); // Assume transaction is valid
                Ok(())
            }
            OpCode::SignTransaction => {
                self.emit_event(
                    "TransactionSigned",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::BroadcastTransaction => {
                self.emit_event(
                    "TransactionBroadcasted",
                    metadata.cooperative_id.clone(),
                    HashMap::new(),
                );
                Ok(())
            }
            OpCode::Log(message) => {
                self.logs.push(format!("Log: {}", message));
                Ok(())
            }
            OpCode::Halt => {
                self.instruction_pointer = self.instruction_limit;
                Ok(())
            }
            OpCode::EmitEvent(event_type) => {
                self.emit_event(event_type, metadata.cooperative_id.clone(), HashMap::new());
                Ok(())
            }
            OpCode::GetBlockNumber => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                self.stack.push(context.block_number as i64);
                Ok(())
            }
            OpCode::GetTimestamp => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                self.stack.push(context.timestamp as i64);
                Ok(())
            }
                // Add the GetCaller implementation here:
            OpCode::GetCaller => {
                let context = self.execution_context.as_ref()
                    .ok_or("No execution context".to_string())?;
                let caller_id_hash = self.hash_did(&context.caller_did);
                self.stack.push(caller_id_hash);
                Ok(())
            }
            
            OpCode::Equal => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(if a == b { 1 } else { 0 });
                Ok(())
            }
            OpCode::NotEqual => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(if a != b { 1 } else { 0 });
                Ok(())
            }
            OpCode::GreaterThan => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(if a > b { 1 } else { 0 });
                Ok(())
            }
            OpCode::LessThan => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(if a < b { 1 } else { 0 });
                Ok(())
            }
            OpCode::And => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a & b);
                Ok(())
            }
            OpCode::Or => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a | b);
                Ok(())
            }
            OpCode::Not => {
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(!a);
                Ok(())
            }
            OpCode::Nop => Ok(()),
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

    fn hash_did(&self, did: &str) -> i64 {
        // Simple hash function for example purposes
        did.bytes().fold(0, |acc, b| acc + b as i64)
    }

    fn reverse_hash_did(&self, hash: i64) -> String {
        format!("did:placeholder:{}", hash)
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