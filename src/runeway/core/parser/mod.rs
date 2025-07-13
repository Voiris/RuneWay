mod process;

use crate::runeway::core::ast::statement::Statement;
use super::lexer::tokenize;
pub use self::process::ParserProcess;

pub fn parse_code(code: String) -> Result<Vec<Statement>, String> {
    let tokens = tokenize(code);

    let mut process = ParserProcess::new(tokens);

    process.parse_full()
}
