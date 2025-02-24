use std::collections::HashMap;

#[derive(Debug)]
pub enum VMValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<VMValue>),
    Object(HashMap<String, VMValue>),
}

#[allow(dead_code)]
#[derive(Default)]
pub struct ExecutionContext {
    variables: HashMap<String, VMValue>,
    permissions: Vec<String>,
}

pub struct IcnVM;

impl IcnVM {
    pub fn new() -> Self {
        IcnVM
    }
    
    // Execute DSL code and return dummy state changes (empty vector)
    pub fn execute_with_state_diff(&self, _ast: crate::dsl::parser::CoopLangAST) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // In a real implementation, the AST would be compiled and executed.
        Ok(vec![])
    }
    
    // Execute validation DSL for a transaction string; return a dummy true result.
    pub fn execute_validation(&self, _tx: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}
