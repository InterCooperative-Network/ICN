use super::*;

#[test]
fn test_parse_validation_rules() {
    let input = r#"
        validation:
          pre_checks:
            - balance >= 100: require_minimum_balance
            - reputation > 50: check_reputation
          post_checks:
            - total_supply < 1000000: ensure_supply_cap
          state_validation:
            current: "PENDING"
            expected: "APPROVED"
            transition: "PENDING->APPROVED"
    "#;

    let (_, ast) = CoopLangAST::parse(input).unwrap();
    let validation = ast.validation.unwrap();
    
    assert_eq!(validation.pre_checks.len(), 2);
    assert_eq!(validation.post_checks.len(), 1);
    assert!(validation.state_validation.is_some());
}

#[test]
fn test_bytecode_generation() {
    let ast = CoopLangAST {
        validation: Some(ValidationNode {
            pre_checks: vec![
                Check {
                    condition: "balance >= 100".to_string(),
                    action: "require_minimum_balance".to_string(),
                }
            ],
            post_checks: vec![],
            state_validation: None,
            resource_checks: None,
            custom_merge: None,
        }),
        governance: None,
        reputation: None,
        marketplace: None,
        federation: None,
        logging: None,
    };

    let bytecode = compile_to_icvm(&ast);
    
    // Verify bytecode structure
    assert_eq!(&bytecode[0..4], b"ICVM"); // Magic bytes
    assert_eq!(bytecode[4], 0x01); // Version
    assert_eq!(bytecode[5], 0x01); // Validation section
    assert_eq!(bytecode[6], 0x01); // One pre-check
}

#[test]
fn test_validation_rule_execution() {
    let runtime = RuntimeManager::new();
    let context = ExecutionContext::default();
    
    let validation = ValidationNode {
        pre_checks: vec![
            Check {
                condition: "balance >= 100".to_string(),
                action: "require_minimum_balance".to_string(),
            }
        ],
        post_checks: vec![],
        state_validation: Some(StateValidation {
            current: Some("PENDING".to_string()),
            expected: Some("APPROVED".to_string()),
            transition: Some("PENDING->APPROVED".to_string()),
        }),
        resource_checks: None,
        custom_merge: None,
    };

    let result = runtime.execute_validation_rules(&validation, &context);
    assert!(result.is_ok());
}
