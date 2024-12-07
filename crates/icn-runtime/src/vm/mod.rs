use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMState {
    pub memory: HashMap<String, Vec<u8>>,
    pub stack: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct VM {
    pub state: VMState,
}

impl VM {
    pub fn new() -> Self {
        Self {
            state: VMState {
                memory: HashMap::new(),
                stack: Vec::new(),
            }
        }
    }
}
