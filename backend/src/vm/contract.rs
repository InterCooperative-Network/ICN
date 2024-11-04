use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::opcode::OpCode;
use super::cooperative_metadata::CooperativeMetadata;

#[derive(Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub code: Vec<OpCode>,
    pub state: HashMap<String, i64>,
    pub required_reputation: i64,
    pub cooperative_metadata: CooperativeMetadata,
    pub version: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}
