use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/coop_lang.pest"]
pub struct CoopLangParser;

// New: Minimal DSL AST that wraps the raw DSL input.
#[derive(Debug)]
pub struct CoopLangAST {
    pub raw: String,
}

impl CoopLangAST {
    pub fn new(raw: String) -> Self {
        Self { raw }
    }
}

// Remove the old AstNode definitions and helper functions.
// Instead, we simply delegate to CoopLangAST::new.

pub fn parse(input: &str) -> Result<CoopLangAST, Box<dyn std::error::Error>> {
    // Attempt to parse using Pest; errors are mapped if any.
    let _ = CoopLangParser::parse(Rule::program, input)?;
    // For now, simply return the raw DSL as our AST.
    Ok(CoopLangAST::new(input.to_owned()))
}
