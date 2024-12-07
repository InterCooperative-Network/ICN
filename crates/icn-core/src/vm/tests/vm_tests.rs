#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    fn setup_vm() -> VM {
        let instruction_limit = 1000;
        let reputation_context = HashMap::new();
        VM::new(instruction_limit, reputation_context)
    }

    fn default_execution_context() -> ExecutionContext {
        ExecutionContext {
            caller_did: "did:icn:12345".to_string(),
            cooperative_id: "coop-1".to_string(),
            timestamp: 1620000000,
            block_number: 1,
            reputation_score: 100,
            permissions: vec!["cooperative.create".to_string(), "proposal.create".to_string()],
        }
    }

    fn default_cooperative_metadata() -> CooperativeMetadata {
        CooperativeMetadata {
            creator_did: "did:icn:12345".to_string(),
            cooperative_id: "coop-1".to_string(),
            purpose: "Test Cooperative".to_string(),
            resource_impact: ResourceImpact {
                cpu_intensity: 1,
                memory_usage: 1,
                network_usage: 1,
                storage_usage: 1,
                bandwidth_usage: 1,
            },
            federation_id: None,
            creation_timestamp: 1620000000,
            last_updated: 1620000000,
            member_count: 1,
            resource_allocation: HashMap::new(),
        }
    }

    #[test]
    fn test_push_pop() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "test_contract".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(20),
                OpCode::Pop,
                OpCode::Push(30),
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], 10);
        assert_eq!(stack[1], 30);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "arithmetic_contract".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(5),
                OpCode::Add,
                OpCode::Push(2),
                OpCode::Mul,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], 30); // (10 + 5) * 2 = 30
    }

    #[test]
    fn test_memory_operations() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "memory_contract".to_string(),
            code: vec![
                OpCode::Push(42),
                OpCode::Store("answer".to_string()),
                OpCode::Load("answer".to_string()),
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], 42);
    }

    #[test]
    fn test_control_flow_jump() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "control_flow_contract".to_string(),
            code: vec![
                OpCode::Push(1),             // 0
                OpCode::Jump(4),             // 1
                OpCode::Push(999),           // 2 (should be skipped)
                OpCode::Halt,                // 3 (should be skipped)
                OpCode::Push(2),             // 4
                OpCode::Halt,                // 5
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], 1);
        assert_eq!(stack[1], 2);
    }

    #[test]
    fn test_control_flow_jumpif() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "control_flow_jumpif_contract".to_string(),
            code: vec![
                OpCode::Push(0),             // 0
                OpCode::JumpIf(4),           // 1 (should not jump)
                OpCode::Push(1),             // 2
                OpCode::Halt,                // 3
                OpCode::Push(2),             // 4
                OpCode::Halt,                // 5
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], 1);
    }

    #[test]
    fn test_comparison_operations() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "comparison_contract".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(10),
                OpCode::Equal,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());
        let stack = vm.get_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], 1); // 1 for true
    }

    #[test]
    fn test_cooperative_operations() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "cooperative_contract".to_string(),
            code: vec![
                OpCode::CreateCooperative,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 100,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec!["cooperative.create".to_string()],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());

        let events = vm.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "CooperativeCreated");
    }

    #[test]
    fn test_reputation_operations() {
        let mut reputation_context = HashMap::new();
        reputation_context.insert("did:icn:12345".to_string(), 50);

        let mut vm = VM::new(1000, reputation_context);
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "reputation_contract".to_string(),
            code: vec![
                OpCode::UpdateReputation(25),
                OpCode::GetReputation,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());

        let stack = vm.get_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], 75);

        let updated_reputation = vm.reputation_context.get("did:icn:12345").copied().unwrap_or(0);
        assert_eq!(updated_reputation, 75);
    }

    #[test]
    fn test_error_handling_division_by_zero() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "error_contract".to_string(),
            code: vec![
                OpCode::Push(10),
                OpCode::Push(0),
                OpCode::Div,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Division by zero".to_string());
    }

    #[test]
    fn test_event_emission() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "event_contract".to_string(),
            code: vec![
                OpCode::EmitEvent("CustomEvent".to_string()),
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_ok());

        let events = vm.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "CustomEvent");
    }

    #[test]
    fn test_insufficient_reputation() {
        let mut vm = setup_vm();
        vm.set_execution_context(default_execution_context());

        let contract = Contract {
            id: "high_reputation_contract".to_string(),
            code: vec![
                OpCode::Push(1),
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 200,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient reputation to execute contract".to_string());
    }

    #[test]
    fn test_missing_permission() {
        let mut vm = setup_vm();
        let mut context = default_execution_context();
        context.permissions = vec![]; // No permissions
        vm.set_execution_context(context);

        let contract = Contract {
            id: "permission_contract".to_string(),
            code: vec![
                OpCode::CreateCooperative,
                OpCode::Halt,
            ],
            state: HashMap::new(),
            required_reputation: 0,
            cooperative_metadata: default_cooperative_metadata(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec!["cooperative.create".to_string()],
        };

        let result = vm.execute_contract(&contract);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing permission: cooperative.create".to_string());
    }
}
