use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dsl/grammar.pest"]
pub struct CoopLangParser;

#[derive(Debug)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Governance(Vec<AstNode>),
    Reputation(Vec<AstNode>),
    Marketplace(Vec<AstNode>),
    Rule { key: String, value: String },
    Threshold(Vec<AstNode>),
    Federation(Vec<AstNode>),
    Value(Value),
}

pub fn parse(input: &str) -> Result<AstNode, Box<dyn std::error::Error>> {
    let pairs = CoopLangParser::parse(Rule::program, input)?;
    let mut ast = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for statement in pair.into_inner() {
                    ast.push(parse_statement(statement)?);
                }
            }
            _ => {}
        }
    }

    Ok(AstNode::Program(ast))
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, Box<dyn std::error::Error>> {
    match pair.as_rule() {
        Rule::governance_rule => Ok(AstNode::Governance(parse_block(pair)?)),
        Rule::reputation_rule => Ok(AstNode::Reputation(parse_block(pair)?)),
        Rule::marketplace_rule => Ok(AstNode::Marketplace(parse_block(pair)?)),
        Rule::threshold_rule => Ok(AstNode::Threshold(parse_block(pair)?)),
        Rule::federation_rule => Ok(AstNode::Federation(parse_block(pair)?)),
        _ => Err("Unknown statement type".into()),
    }
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Vec<AstNode>, Box<dyn std::error::Error>> {
    let mut rules = Vec::new();
    
    for pair in pair.into_inner() {
        if let Rule::rule_statement = pair.as_rule() {
            let mut inner = pair.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let value = inner.next().unwrap().as_str().to_string();
            rules.push(AstNode::Rule { key, value });
        }
    }

    Ok(rules)
}

fn parse_complex_value(pair: pest::iterators::Pair<Rule>) -> Result<Value, Box<dyn std::error::Error>> {
    match pair.as_rule() {
        Rule::array_value => parse_array(pair),
        Rule::object_value => parse_object(pair),
        Rule::simple_value => parse_simple_value(pair),
        _ => Err("Invalid value type".into()),
    }
}

// Add helper functions for parsing complex values...
