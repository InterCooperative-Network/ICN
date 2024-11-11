// src/vm/opcode.rs

use serde::{Serialize, Deserialize};

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
    CreateCooperative,
    /// Join an existing cooperative.
    JoinCooperative,
    /// Leave a cooperative.
    LeaveCooperative,
    /// Allocate resources within a cooperative.
    AllocateResource,
    /// Transfer resources within or between cooperatives.
    TransferResource,
    /// Update cooperative metadata.
    UpdateCooperativeMetadata,
    /// Add a member to a cooperative.
    AddCooperativeMember,
    /// Remove a member from a cooperative.
    RemoveCooperativeMember,
    /// Set a specific role for a cooperative member.
    SetMemberRole,

    // Governance Operations
    /// Create a new proposal in governance.
    CreateProposal,
    /// Cast a vote on a proposal.
    CastVote,
    /// Delegate voting rights to another member.
    DelegateVotes,
    /// Execute an approved proposal.
    ExecuteProposal,
    /// Update the quorum requirements for voting.
    UpdateQuorum,
    /// Cancel an existing proposal.
    CancelProposal,
    /// Extend the voting period for a proposal.
    ExtendVotingPeriod,
    /// Calculate the weight of a vote based on metrics.
    CalculateVotingWeight,

    // Reputation Operations
    /// Update a member's reputation by a specified value.
    UpdateReputation(i64),
    /// Retrieve a member's reputation.
    GetReputation,
    /// Transfer reputation between members.
    TransferReputation,
    /// Burn (remove) a certain amount of reputation.
    BurnReputation,
    /// Mint (create) a certain amount of reputation.
    MintReputation,

    // Identity Operations
    /// Verify a Decentralized Identifier (DID).
    VerifyDID,
    /// Update a DID document.
    UpdateDIDDocument,
    /// Create a new credential.
    CreateCredential,
    /// Verify a credential.
    VerifyCredential,
    /// Revoke a credential.
    RevokeCredential,

    // Federation Operations
    /// Initiate federation process.
    InitiateFederation,
    /// Join an existing federation.
    JoinFederation,
    /// Leave a federation.
    LeaveFederation,
    /// Synchronize federation state.
    SyncFederationState,
    /// Validate federation action.
    ValidateFederationAction,

    // Transaction Operations
    /// Create a new transaction.
    CreateTransaction,
    /// Validate a transaction.
    ValidateTransaction,
    /// Sign a transaction.
    SignTransaction,
    /// Broadcast a transaction.
    BroadcastTransaction,

    // Relationship Operations
    /// Record a contribution with impact story
    RecordContribution {
        description: String,
        impact_story: String,
        context: String,
        tags: Vec<String>,
    },
    /// Record mutual aid interaction
    RecordMutualAid {
        description: String,
        receiver: String,
        impact_story: Option<String>,
        reciprocity_notes: Option<String>,
    },
    /// Update relationship between members
    UpdateRelationship {
        member_two: String,
        relationship_type: String,
        story: String,
    },
    /// Add endorsement to relationship
    AddEndorsement {
        to_did: String,
        content: String,
        context: String,
        skills: Vec<String>,
    },
    /// Record relationship interaction
    RecordInteraction {
        with_did: String,
        description: String,
        impact: Option<String>,
        interaction_type: String,
    },
    /// Add witness to contribution
    AddWitness {
        contribution_id: String,
        witness_did: String,
    },
    /// Add feedback to contribution
    AddFeedback {
        contribution_id: String,
        content: String,
        endorsement_type: String,
    },

    // System Operations
    /// Log a message.
    Log(String),
    /// Halt execution.
    Halt,
    /// Emit an event.
    EmitEvent(String),
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