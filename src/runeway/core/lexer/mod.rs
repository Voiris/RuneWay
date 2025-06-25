pub mod token;
mod process;

use self::token::Token;
use self::process::LexerProcess;

pub fn tokenize(input: String) -> Vec<Token> {
    LexerProcess::new(input).tokenize()
}
