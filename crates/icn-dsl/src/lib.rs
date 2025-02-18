use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone)]
pub struct CoopLangAST {
    pub governance: Option<GovernanceNode>,
    pub reputation: Option<ReputationNode>,
    pub marketplace: Option<MarketplaceNode>,
    pub federation: Option<FederationNode>,
    pub validation: Option<ValidationNode>,
    pub logging: Option<LoggingNode>,
}

#[derive(Debug, Clone)]
pub struct ValidationNode {
    pub pre_checks: Vec<Check>,
    pub post_checks: Vec<Check>,
    pub state_validation: Option<StateValidation>,
    pub resource_checks: Option<ResourceChecks>,
    pub custom_merge: Option<CustomMerge>,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub condition: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct StateValidation {
    pub current: Option<String>,
    pub expected: Option<String>,
    pub transition: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CustomMerge {
    pub strategy: String,
    pub handlers: Vec<ConflictHandler>,
}

#[derive(Debug, Clone)]
pub struct ConflictHandler {
    pub field_path: String,
    pub resolution_type: String,
}

// Parser implementation
impl CoopLangAST {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = multispace0(input)?;
        let (input, governance) = opt(Self::parse_governance_section)(input)?;
        let (input, reputation) = opt(Self::parse_reputation_section)(input)?;
        let (input, marketplace) = opt(Self::parse_marketplace_section)(input)?;
        let (input, federation) = opt(Self::parse_federation_section)(input)?;
        let (input, validation) = opt(Self::parse_validation_section)(input)?;
        let (input, logging) = opt(Self::parse_logging_section)(input)?;
        
        Ok((input, CoopLangAST {
            governance,
            reputation,
            marketplace,
            federation,
            validation,
            logging,
        }))
    }

    fn parse_validation_section(input: &str) -> IResult<&str, ValidationNode> {
        let (input, _) = tag("validation:")(input)?;
        let (input, _) = multispace1(input)?;
        
        let (input, pre_checks) = Self::parse_checks("pre_checks:")(input)?;
        let (input, post_checks) = Self::parse_checks("post_checks:")(input)?;
        let (input, state_validation) = opt(Self::parse_state_validation)(input)?;
        let (input, resource_checks) = opt(Self::parse_resource_checks)(input)?;
        let (input, custom_merge) = opt(Self::parse_custom_merge)(input)?;

        Ok((input, ValidationNode {
            pre_checks,
            post_checks,
            state_validation,
            resource_checks,
            custom_merge,
        }))
    }

    fn parse_checks(label: &'static str) -> impl Fn(&str) -> IResult<&str, Vec<Check>> {
        move |input: &str| {
            let (input, _) = tag(label)(input)?;
            let (input, _) = multispace1(input)?;
            many0(Self::parse_check)(input)
        }
    }

    fn parse_check(input: &str) -> IResult<&str, Check> {
        let (input, _) = char('-')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, condition) = take_while1(|c| c != ':')(input)?;
        let (input, _) = char(':')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, action) = take_while1(|c| c != '\n')(input)?;
        let (input, _) = multispace0(input)?;

        Ok((input, Check {
            condition: condition.trim().to_string(),
            action: action.trim().to_string(),
        }))
    }

    // Add other section parsers similarly...
}

// Bytecode generation
pub fn compile_to_icvm(ast: &CoopLangAST) -> Vec<u8> {
    let mut bytecode = Vec::new();

    // Header
    bytecode.extend_from_slice(&[0x49, 0x43, 0x56, 0x4D]); // "ICVM" magic bytes
    bytecode.push(0x01); // Version

    // Compile validation rules
    if let Some(validation) = &ast.validation {
        bytecode.push(0x01); // Validation section marker
        
        // Pre-checks
        bytecode.push(validation.pre_checks.len() as u8);
        for check in &validation.pre_checks {
            compile_check(&mut bytecode, check);
        }

        // Post-checks
        bytecode.push(validation.post_checks.len() as u8);
        for check in &validation.post_checks {
            compile_check(&mut bytecode, check);
        }

        // State validation
        if let Some(state_validation) = &validation.state_validation {
            bytecode.push(0x01);
            compile_state_validation(&mut bytecode, state_validation);
        } else {
            bytecode.push(0x00);
        }
    }

    // Compile other sections similarly...

    bytecode
}

fn compile_check(bytecode: &mut Vec<u8>, check: &Check) {
    // Convert check condition to bytecode operations
    bytecode.extend_from_slice(check.condition.as_bytes());
    bytecode.push(0x00); // Null terminator
    bytecode.extend_from_slice(check.action.as_bytes());
    bytecode.push(0x00); // Null terminator
}

fn compile_state_validation(bytecode: &mut Vec<u8>, validation: &StateValidation) {
    if let Some(current) = &validation.current {
        bytecode.push(0x01);
        bytecode.extend_from_slice(current.as_bytes());
        bytecode.push(0x00);
    } else {
        bytecode.push(0x00);
    }
    // Similarly for expected and transition...
}
