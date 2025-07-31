mod process;
pub mod token;

use self::process::LexerProcess;
use crate::runeway::core::lexer::token::SpannedToken;

pub fn tokenize(input: String) -> Vec<SpannedToken> {
    LexerProcess::new(input).tokenize()
}
