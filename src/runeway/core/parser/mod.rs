mod process;

use super::lexer::token::Token;
pub use self::process::ParserProcess;

/*
pub fn parse(tokens: Vec<Token>) -> Module {
    let mut process = ParserProcess::new(tokens);

    process.parse_module()
}
 */
