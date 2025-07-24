mod process;

use crate::runeway::core::ast::statement::{SpannedStatement, Statement};
use crate::runeway::core::errors::{RWResult, RuneWayError};
use super::lexer::tokenize;
pub use self::process::ParserProcess;

#[derive(Debug, Clone)]
pub struct ParsedCode {
    pub code: String,
    pub ast: Vec<SpannedStatement>,
}

pub fn parse_code(code: String) -> RWResult<ParsedCode> {
    let tokens = tokenize(code.clone());

    let mut process = ParserProcess::new(tokens);

    Ok(ParsedCode {
        code,
        ast: process.parse_full()?
    })
}
