use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug)]
pub enum VMValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<VMValue>),
    Object(HashMap<String, VMValue>),
}

#[derive(Default)]
pub struct ExecutionContext {
    variables: HashMap<String, VMValue>,
    permissions: Vec<String>,
}

pub struct IcnVM {
    state: HashMap<String, VMValue>,
    contexts: Vec<ExecutionContext>,
}

impl IcnVM {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            contexts: vec![ExecutionContext::default()],
        }
    }

    pub fn execute(&mut self, ast: crate::dsl::parser::AstNode) -> Result<(), Box<dyn std::error::Error>> {
        match ast {
            crate::dsl::parser::AstNode::Program(statements) => {
                self.create_execution_context()?;
                for stmt in statements {
                    self.execute_statement(stmt)?;
                }
                self.pop_execution_context()?;
            }
            _ => return Err("Expected program node".into()),
        }
        Ok(())
    }

    fn create_execution_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.contexts.push(ExecutionContext::default());
        Ok(())
    }

    fn pop_execution_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.contexts.pop().ok_or_else(|| "No context to pop".into())
    }

    fn current_context(&mut self) -> Result<&mut ExecutionContext, Box<dyn std::error::Error>> {
        self.contexts.last_mut().ok_or_else(|| "No execution context".into())
    }

    fn execute_statement(&mut self, node: crate::dsl::parser::AstNode) -> Result<(), Box<dyn std::error::Error>> {
        match node {
            crate::dsl::parser::AstNode::Governance(rules) => self.execute_rules("governance", rules),
            crate::dsl::parser::AstNode::Reputation(rules) => self.execute_rules("reputation", rules),
            crate::dsl::parser::AstNode::Marketplace(rules) => self.execute_rules("marketplace", rules),
            _ => Err("Invalid statement type".into()),
        }
    }

    fn execute_rules(&mut self, context: &str, rules: Vec<crate::dsl::parser::AstNode>) -> Result<(), Box<dyn std::error::Error>> {
        let ctx = self.current_context()?;
        for rule in rules {
            if let crate::dsl::parser::AstNode::Rule { key, value } = rule {
                ctx.variables.insert(format!("{}.{}", context, key), self.parse_value(&value)?);
            }
        }
        Ok(())
    }

    fn parse_value(&self, value: &str) -> Result<VMValue, Box<dyn std::error::Error>> {
        if let Ok(num) = value.parse::<f64>() {
            return Ok(VMValue::Number(num));
        }
        if value == "true" || value == "false" {
            return Ok(VMValue::Boolean(value == "true"));
        }
        Ok(VMValue::String(value.to_string()))
    }

    pub fn execute_with_state_diff(&self, ast: crate::dsl::parser::AstNode) 
        -> Result<HashMap<String, VMValue>, Box<dyn std::error::Error>> 
    {
        let mut state_before = self.state.clone();
        self.execute(ast)?;
        
        // Calculate state changes
        let mut changes = HashMap::new();
        for (key, value) in self.state.iter() {
            if !state_before.contains_key(key) || state_before[key] != *value {
                changes.insert(key.clone(), value.clone());
            }
        }
        
        Ok(changes)
    }

    pub fn execute_validation(&self, tx: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let ast = crate::dsl::parser::parse(tx)?;
        
        // Create validation context
        self.create_execution_context()?;
        
        // Execute validation rules
        let result = match ast {
            crate::dsl::parser::AstNode::Marketplace(rules) => {
                self.validate_marketplace_rules(&rules)
            },
            _ => Ok(false),
        };
        
        self.pop_execution_context()?;
        result
    }

    fn validate_marketplace_rules(&self, rules: &[crate::dsl::parser::AstNode]) 
        -> Result<bool, Box<dyn std::error::Error>> 
    {
        // Implement marketplace validation logic
        Ok(true)
    }
}
