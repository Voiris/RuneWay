mod process;

use crate::runeway::core::ast::statement::Statement;
use super::lexer::tokenize;
pub use self::process::ParserProcess;

pub fn parse_code(code: String) -> Result<Vec<Statement>, String> {
    let tokens = tokenize(code);

    println!("Tokens: {}", tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().join("\n"));

    let mut process = ParserProcess::new(tokens);

    process.parse_full()
}
