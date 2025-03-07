use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    
    // Stack
    Push(i64),
    Pop,
    Dup,
    Swap,
    
    // Memory
    Load,
    Store,
    
    // Control Flow
    Jump,
    JumpIf,
    Call,
    Return,
    
    // System
    Log,
    Halt,
    EmitEvent,
    GetTimestamp,
    GetCaller,
}