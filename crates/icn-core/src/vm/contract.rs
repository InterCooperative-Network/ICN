use crate::vm::opcode::OpCode;
use crate::vm::cooperative_metadata::CooperativeMetadata;
use std::collections::HashMap;

#[derive(Debug, Clone)]
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

impl Contract {
    pub fn new(code: Vec<OpCode>, metadata: CooperativeMetadata) -> Self {
        Self {
            id: generate_contract_id(),
            code,
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: metadata,
            version: "1.0.0".to_string(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
        }
    }
}

fn generate_contract_id() -> String {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    format!("contract-{}", rng.gen::<u64>())
}