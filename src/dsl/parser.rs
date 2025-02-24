use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/coop_lang.pest"] // Path relative to this file
pub struct CoopLangParser;

#[derive(Debug)]
pub struct CoopLangAST {
    pub raw: String,
}

impl CoopLangAST {
    pub fn new(raw: String) -> Self {
        Self { raw }
    }
}

pub fn parse(input: &str) -> Result<CoopLangAST, Box<dyn std::error::Error>> {
    let _ = CoopLangParser::parse(Rule::program, input)?;
    Ok(CoopLangAST::new(input.to_owned()))
}
