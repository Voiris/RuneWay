mod process;

pub use self::process::ParserProcess;
use super::lexer::tokenize;
use crate::runeway::core::ast::statement::SpannedStatement;
use crate::runeway::core::errors::RWResult;

#[derive(Debug, Clone)]
pub struct ParsedCode {
    pub code: String,
    pub ast: Vec<SpannedStatement>,
}

pub fn parse_code(filename: String, code: String) -> RWResult<ParsedCode> {
    let tokens = tokenize(code.clone());

    let mut process = ParserProcess::new(tokens, filename);

    Ok(ParsedCode {
        code,
        ast: process.parse_full()?,
    })
}
