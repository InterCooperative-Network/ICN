// src/vm/opcode.rs

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enum representing the various operations (`OpCode`) that can be executed in the virtual machine.
/// Each variant is an operation that affects the VM stack, memory, or interacts with other subsystems.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    // Stack Operations
    /// Push a value onto the stack.
    Push(i64),        
    /// Pop the top value from the stack.
    Pop,              
    /// Duplicate the top value on the stack.
    Dup,              
    /// Swap the top two values on the stack.
    Swap,             

    // Arithmetic Operations
    /// Add the top two values on the stack.
    Add,              
    /// Subtract the top two values on the stack.
    Sub,              
    /// Multiply the top two values on the stack.
    Mul,              
    /// Divide the top two values on the stack.
    Div,              
    /// Compute the modulo of the top two values on the stack.
    Mod,              

    // Memory Operations
    /// Store a value in memory with a specific key.
    Store(String),    
    /// Load a value from memory using a specific key.
    Load(String),     

    // Control Flow Operations
    /// Unconditional jump to a specified instruction index.
    Jump(usize),      
    /// Conditional jump to an instruction index if the top of the stack is non-zero.
    JumpIf(usize),    

    // Cooperative Operations
    /// Create a new cooperative entity.
    CreateCooperative {
        name: String,
        description: String,
        resources: HashMap<String, u64>,
        federation_id: Option<String>,
    },
    /// Join an existing cooperative.
    JoinCooperative {
        cooperative_id: String,
        role: String,
        commitment: String,
    },
    /// Leave a cooperative.
    LeaveCooperative {
        cooperative_id: String,
        reason: String,
    },
    /// Allocate resources within a cooperative.
    AllocateResource {
        resource_type: String,
        amount: u64,
        recipient: String,
        purpose: String,
    },
    /// Transfer resources within or between cooperatives.
    TransferResource {
        from_cooperative: String,
        to_cooperative: String,
        resource_type: String,
        amount: u64,
        purpose: String,
    },

    // Governance Operations
    /// Create a new governance proposal.
    CreateProposal {
        title: String,
        description: String,
        proposal_type: String,
        voting_period: u64,
        required_reputation: i64,
    },
    /// Cast a vote on a proposal.
    CastVote {
        proposal_id: String,
        approve: bool,
        comment: Option<String>,
    },
    /// Delegate voting rights to another member.
    DelegateVotes {
        delegate_to: String,
        scope: String,
        duration: Option<u64>,
    },
    /// Execute an approved proposal.
    ExecuteProposal {
        proposal_id: String,
    },

    // Reputation Operations
    /// Update a member's reputation by a specific amount.
    UpdateReputation {
        target: String,
        amount: i64,
        reason: String,
        context: String,
    },
    /// Get the current reputation of a member.
    GetReputation {
        target: String,
    },
    /// Burn (remove) reputation from a member.
    BurnReputation {
        amount: i64,
        reason: String,
    },

    // Relationship Operations
    /// Record a contribution with impact story.
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    /// Record mutual aid interaction between members.
    RecordMutualAid {
        receiver: String,
        description: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
        tags: Vec<String>,
    },
    /// Update relationship between members.
    UpdateRelationship {
        member_two: String,
        relationship_type: String,
        story: String,
        interaction: Option<String>,
    },
    /// Add endorsement to a relationship.
    AddEndorsement {
        to_did: String,
        content: String,
        context: String,
        skills: Vec<String>,
    },
    /// Record a relationship interaction.
    RecordInteraction {
        with_did: String,
        description: String,
        impact: Option<String>,
        interaction_type: String,
    },

    // Identity Operations
    /// Verify a Decentralized Identifier (DID).
    VerifyDID {
        did: String,
    },
    /// Update a DID document.
    UpdateDIDDocument {
        did: String,
        updates: HashMap<String, String>,
    },
    /// Create a new credential.
    CreateCredential {
        recipient: String,
        credential_type: String,
        claims: HashMap<String, String>,
    },
    /// Verify a credential.
    VerifyCredential {
        credential_id: String,
    },

    // Federation Operations
    /// Initiate a new federation.
    InitiateFederation {
        name: String,
        description: String,
        rules: Vec<String>,
        initial_members: Vec<String>,
    },
    /// Join an existing federation.
    JoinFederation {
        federation_id: String,
        commitment: Vec<String>,
    },
    /// Leave a federation.
    LeaveFederation {
        federation_id: String,
        reason: String,
    },
    /// Synchronize federation state.
    SyncFederationState {
        federation_id: String,
    },

    // System Operations
    /// Log a message.
    Log(String),
    /// Halt execution.
    Halt,
    /// Emit an event.
    EmitEvent {
        event_type: String,
        data: HashMap<String, String>,
    },
    /// Get current block number.
    GetBlockNumber,
    /// Get current timestamp.
    GetTimestamp,
    /// Get caller's DID.
    GetCaller,

    // Comparison Operations
    /// Check if values are equal.
    Equal,
    /// Check if values are not equal.
    NotEqual,
    /// Check if greater than.
    GreaterThan,
    /// Check if less than.
    LessThan,

    // Logical Operations
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
    /// Logical NOT.
    Not,

    // No Operation
    /// No operation (used for padding or delays).
    Nop,
}

impl OpCode {
    /// Returns the gas cost of executing this operation
    pub fn gas_cost(&self) -> u64 {
        match self {
            // Stack operations are cheap
            OpCode::Push(_) | OpCode::Pop | OpCode::Dup | OpCode::Swap => 1,
            
            // Arithmetic operations have fixed costs
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => 2,
            
            // Memory operations are more expensive
            OpCode::Store(_) | OpCode::Load(_) => 5,
            
            // Control flow has moderate cost
            OpCode::Jump(_) | OpCode::JumpIf(_) => 8,
            
            // Cooperative operations are expensive
            OpCode::CreateCooperative { .. } => 1000,
            OpCode::JoinCooperative { .. } => 500,
            OpCode::LeaveCooperative { .. } => 200,
            OpCode::AllocateResource { .. } => 300,
            OpCode::TransferResource { .. } => 400,
            
            // Governance operations have high costs
            OpCode::CreateProposal { .. } => 1000,
            OpCode::CastVote { .. } => 100,
            OpCode::DelegateVotes { .. } => 200,
            OpCode::ExecuteProposal { .. } => 500,
            
            // Reputation operations are moderate
            OpCode::UpdateReputation { .. } => 200,
            OpCode::GetReputation { .. } => 50,
            OpCode::BurnReputation { .. } => 300,
            
            // Relationship operations
            OpCode::RecordContribution { .. } => 200,
            OpCode::RecordMutualAid { .. } => 150,
            OpCode::UpdateRelationship { .. } => 100,
            OpCode::AddEndorsement { .. } => 100,
            OpCode::RecordInteraction { .. } => 50,
            
            // Identity operations are expensive
            OpCode::VerifyDID { .. } => 300,
            OpCode::UpdateDIDDocument { .. } => 500,
            OpCode::CreateCredential { .. } => 400,
            OpCode::VerifyCredential { .. } => 200,
            
            // Federation operations
            OpCode::InitiateFederation { .. } => 1000,
            OpCode::JoinFederation { .. } => 500,
            OpCode::LeaveFederation { .. } => 200,
            OpCode::SyncFederationState { .. } => 300,
            
            // System operations are cheap
            OpCode::Log(_) => 5,
            OpCode::Halt => 1,
            OpCode::EmitEvent { .. } => 10,
            OpCode::GetBlockNumber | OpCode::GetTimestamp | OpCode::GetCaller => 1,
            
            // Basic operations are very cheap
            OpCode::Equal | OpCode::NotEqual | OpCode::GreaterThan | OpCode::LessThan => 1,
            OpCode::And | OpCode::Or | OpCode::Not => 1,
            OpCode::Nop => 1,
        }
    }

    /// Returns whether this operation modifies state
    pub fn is_state_modifying(&self) -> bool {
        !matches!(self,
            OpCode::GetReputation { .. } |
            OpCode::GetBlockNumber |
            OpCode::GetTimestamp |
            OpCode::GetCaller |
            OpCode::Equal |
            OpCode::NotEqual |
            OpCode::GreaterThan |
            OpCode::LessThan |
            OpCode::Nop
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_costs() {
        // Test basic operations
        assert_eq!(OpCode::Push(1).gas_cost(), 1);
        assert_eq!(OpCode::Add.gas_cost(), 2);
        
        // Test expensive operations
        assert_eq!(OpCode::CreateCooperative {
            name: "test".to_string(),
            description: "test".to_string(),
            resources: HashMap::new(),
            federation_id: None,
        }.gas_cost(), 1000);
    }

    #[test]
    fn test_state_modification() {
        assert!(!OpCode::GetBlockNumber.is_state_modifying());
        assert!(OpCode::CreateProposal {
            title: "test".to_string(),
            description: "test".to_string(),
            proposal_type: "test".to_string(),
            voting_period: 100,
            required_reputation: 10,
        }.is_state_modifying());
    }
}