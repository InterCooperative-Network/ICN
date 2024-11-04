use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    // Stack Operations
    Push(i64),        // Push a value onto the stack
    Pop,              // Pop the top value from the stack
    Dup,              // Duplicate the top value on the stack
    Swap,             // Swap the top two values on the stack

    // Arithmetic Operations
    Add,              // Add top two values
    Sub,              // Subtract top two values
    Mul,              // Multiply top two values
    Div,              // Divide top two values
    Mod,              // Modulo of top two values

    // Memory Operations
    Store(String),    // Store a value in memory with a key
    Load(String),     // Load a value from memory with a key

    // Control Flow Operations
    Jump(usize),      // Unconditional jump to instruction index
    JumpIf(usize),    // Conditional jump if top of stack is non-zero

    // Cooperative Operations
    CreateCooperative,
    JoinCooperative,
    LeaveCooperative,
    AllocateResource,
    TransferResource,
    UpdateCooperativeMetadata,
    AddCooperativeMember,
    RemoveCooperativeMember,
    SetMemberRole,

    // Governance Operations
    CreateProposal,
    CastVote,
    DelegateVotes,
    ExecuteProposal,
    UpdateQuorum,
    CancelProposal,
    ExtendVotingPeriod,
    CalculateVotingWeight,

    // Reputation Operations
    UpdateReputation(i64),
    GetReputation,
    TransferReputation,
    BurnReputation,
    MintReputation,

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
    Log(String),
    Halt,
    EmitEvent(String),
    GetBlockNumber,
    GetTimestamp,
    GetCaller,

    // Comparison Operations
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,

    // Logical Operations
    And,
    Or,
    Not,

    // No Operation
    Nop,
}