// src/vm/opcode.rs

use serde::{Serialize, Deserialize};

/// Enum representing the various operations (`OpCode`) that can be executed in the virtual machine.
/// Each variant is an operation that affects the VM stack, memory, or interacts with other subsystems like
/// cooperative, governance, reputation, identity, federation, and system operations.
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
    /// Update cooperative metadata (e.g., purpose, mission).
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
    /// Calculate the weight of a vote based on certain metrics.
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
    /// Update a Decentralized Identifier (DID) document.
    UpdateDIDDocument,
    /// Create a new credential associated with a DID.
    CreateCredential,
    /// Verify a given credential.
    VerifyCredential,
    /// Revoke a previously issued credential.
    RevokeCredential,

    // Federation Operations
    /// Initiate the process of federation.
    InitiateFederation,
    /// Join an existing federation.
    JoinFederation,
    /// Leave a federation.
    LeaveFederation,
    /// Synchronize federation state with peers.
    SyncFederationState,
    /// Validate a specific action within the federation.
    ValidateFederationAction,

    // Transaction Operations
    /// Create a new transaction.
    CreateTransaction,
    /// Validate an existing transaction.
    ValidateTransaction,
    /// Sign a transaction for verification purposes.
    SignTransaction,
    /// Broadcast a transaction to the network.
    BroadcastTransaction,

    // System Operations
    /// Log a message to the system logs.
    Log(String),
    /// Halt the execution of the current process.
    Halt,
    /// Emit an event with a message.
    EmitEvent(String),
    /// Retrieve the current block number.
    GetBlockNumber,
    /// Get the current timestamp.
    GetTimestamp,
    /// Retrieve the caller's DID or identity.
    GetCaller,

    // Comparison Operations
    /// Check if the top two stack values are equal.
    Equal,
    /// Check if the top two stack values are not equal.
    NotEqual,
    /// Check if the top value is greater than the second top value.
    GreaterThan,
    /// Check if the top value is less than the second top value.
    LessThan,

    // Logical Operations
    /// Perform logical AND on the top two stack values.
    And,
    /// Perform logical OR on the top two stack values.
    Or,
    /// Perform logical NOT on the top stack value.
    Not,

    // No Operation
    /// No operation (used for padding or delays).
    Nop,
}

