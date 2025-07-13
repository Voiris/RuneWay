pub mod token;
mod process;

use crate::runeway::core::lexer::token::SpannedToken;
use self::process::LexerProcess;

pub fn tokenize(input: String) -> Vec<SpannedToken> {
    LexerProcess::new(input).tokenize()
}
