// src/vm/mod.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    PUSH,
    POP,
    ADD,
    SUB,
    STORE,
    LOAD,
    CALL,
    RETURN,
    COOPERATIVE_ACTION,  // New opcode for cooperative-specific actions
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<i64>,
    memory: HashMap<String, i64>,
    program_counter: usize,
    contribution_credits: u64,  // Renamed from gas
    reputation_context: HashMap<String, i64>,  // Added reputation context
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub code: Vec<(OpCode, Option<i64>)>,
    pub state: HashMap<String, i64>,
    pub required_reputation: i64,  // Minimum reputation required to execute
    pub cooperative_metadata: CooperativeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeMetadata {
    pub creator_did: String,
    pub cooperative_id: String,
    pub purpose: String,
    pub resource_impact: ResourceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    pub cpu_intensity: u32,
    pub memory_usage: u32,
    pub network_usage: u32,
}

impl VM {
    pub fn new(contribution_limit: u64, reputation_context: HashMap<String, i64>) -> Self {
        VM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program_counter: 0,
            contribution_credits: contribution_limit,
            reputation_context,
        }
    }

    pub fn execute_contract(&mut self, contract: &Contract, executor_did: &str) -> Result<i64, String> {
        // Check reputation requirements
        let executor_reputation = self.reputation_context.get(executor_did)
            .ok_or("Executor not found in reputation context")?;
        
        if *executor_reputation < contract.required_reputation {
            return Err("Insufficient reputation to execute contract".to_string());
        }

        // Calculate credit cost based on resource impact
        let credit_cost = self.calculate_credit_cost(&contract.cooperative_metadata.resource_impact);
        if credit_cost > self.contribution_credits {
            return Err("Insufficient contribution credits".to_string());
        }

        let mut code = contract.code.clone();
        self.memory = contract.state.clone();

        while self.program_counter < code.len() && self.contribution_credits >= credit_cost {
            let (op, value) = &code[self.program_counter];
            self.execute_instruction(op, value.clone(), &contract.cooperative_metadata)?;
            self.contribution_credits -= credit_cost;
            self.program_counter += 1;
        }

        if self.contribution_credits < credit_cost {
            return Err("Out of contribution credits".to_string());
        }

        Ok(self.stack.pop().unwrap_or(0))
    }

    fn calculate_credit_cost(&self, impact: &ResourceImpact) -> u64 {
        // Calculate based on resource impact
        let base_cost = 1;
        let cpu_factor = (impact.cpu_intensity as f64 / 100.0) + 1.0;
        let memory_factor = (impact.memory_usage as f64 / 100.0) + 1.0;
        let network_factor = (impact.network_usage as f64 / 100.0) + 1.0;

        (base_cost as f64 * cpu_factor * memory_factor * network_factor) as u64
    }

    fn execute_instruction(
        &mut self, 
        op: &OpCode, 
        value: Option<i64>,
        metadata: &CooperativeMetadata
    ) -> Result<(), String> {
        match op {
            OpCode::PUSH => {
                if let Some(v) = value {
                    self.stack.push(v);
                }
            }
            OpCode::POP => {
                self.stack.pop().ok_or("Stack underflow")?;
            }
            OpCode::ADD => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a + b);
            }
            OpCode::SUB => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(a - b);
            }
            OpCode::STORE => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let key = format!("var_{}", value);
                self.memory.insert(key, value);
            }
            OpCode::LOAD => {
                if let Some(v) = value {
                    let key = format!("var_{}", v);
                    if let Some(&stored) = self.memory.get(&key) {
                        self.stack.push(stored);
                    }
                }
            }
            OpCode::COOPERATIVE_ACTION => {
                // Handle cooperative-specific actions
                // This could include resource allocation, voting weight calculation, etc.
                self.handle_cooperative_action(value, metadata)?;
            }
            OpCode::CALL => {
                // Implement contract calls with cooperative context
                self.handle_contract_call(value, metadata)?;
            }
            OpCode::RETURN => {
                return Ok(());
            }
        }
        Ok(())
    }

    fn handle_cooperative_action(
        &mut self,
        action_id: Option<i64>,
        metadata: &CooperativeMetadata
    ) -> Result<(), String> {
        match action_id {
            Some(1) => {
                // Resource allocation action
                // Implementation here
                Ok(())
            }
            Some(2) => {
                // Voting weight calculation
                // Implementation here
                Ok(())
            }
            // Add more cooperative actions as needed
            _ => Err("Unknown cooperative action".to_string())
        }
    }

    fn handle_contract_call(
        &mut self,
        contract_id: Option<i64>,
        metadata: &CooperativeMetadata
    ) -> Result<(), String> {
        // Implement cross-contract calls within the cooperative context
        Ok(())
    }
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooperative_contract() {
        let mut reputation_context = HashMap::new();
        reputation_context.insert("did:icn:test".to_string(), 100);

        let mut vm = VM::new(100, reputation_context);
        
        let metadata = CooperativeMetadata {
            creator_did: "did:icn:creator".to_string(),
            cooperative_id: "coop1".to_string(),
            purpose: "Test contract".to_string(),
            resource_impact: ResourceImpact {
                cpu_intensity: 10,
                memory_usage: 10,
                network_usage: 10,
            },
        };

        let contract = Contract {
            code: vec![
                (OpCode::PUSH, Some(5)),
                (OpCode::PUSH, Some(3)),
                (OpCode::ADD, None),
                (OpCode::COOPERATIVE_ACTION, Some(1)),
                (OpCode::RETURN, None),
            ],
            state: HashMap::new(),
            required_reputation: 50,
            cooperative_metadata: metadata,
        };

        let result = vm.execute_contract(&contract, "did:icn:test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);
    }
}