use std::iter::Peekable;
use std::str::Chars;
use crate::runeway::errors::{RuneWayError, RuneWayErrorKind};

pub enum LexerErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
}

impl RuneWayErrorKind for LexerErrorKind {
    fn description(&self) -> &str {
        match self {
            LexerErrorKind::UnexpectedCharacter => "UnexpectedCharacter",
            LexerErrorKind::UnterminatedString => "UnterminatedString",
        }
    }
}

#[derive(PartialEq)]
pub enum Token {
    // Code structure
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),

    // Keywords
    Let,
    In,
    Is,
    Null,

    // Classes
    Class,
    Property,
    Set,
    Get,

    // Functions
    Act,
    Return,

    // Logic
    If,
    Else,

    And,
    Or,
    Not,

    True,
    False,

    // Loops
    For,
    While,
    Break,
    Continue,

    // Equalising and comparison
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=

    // Mathematics
    Plus,           // +
    Minus,          // -
    Asterisk,       // *
    DoubleAsterisk, // **
    Slash,          // /
    Percent,        // %

    // Compound assignment operators
    PlusEqual,           // +=
    MinusEqual,          // -=
    AsteriskEqual,       // *=
    DoubleAsteriskEqual, // **=
    SlashEqual,          // /=
    PercentEqual,        // %=

    // Arrows
    Arrow,        // ->
    DoubleArrow,  // =>

    // Brackets
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]

    // Other
    Comma,        // ,
    Dot,          // .
    Semicolon,    // ;

    // Comments
    DoubleSlash, // //
    SlashStar,   // /*
    StarSlash,   // */

    // Quotes
    // Quote,             // '
    // TripleQuote,       // '''
    // DoubleQuote,       // "
    // TripleDoubleQuote, // """

    // Special
    EOF
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            line: 1,
        }
    }

    fn lex_string_literal(&mut self) -> Token {
        let mut value = String::new();



        return Token::StringLiteral(value);
    }

    fn _tokenize(&mut self) -> Result<Vec<Token>, RuneWayError<LexerErrorKind>> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(&char) = self.chars.peek() {
            match char {
                // Space and escape sequence
                ' ' | '\t' | '\r' => {
                    self.chars.next();
                }
                '\n' => {
                    self.line += 1;
                    self.chars.next();
                }

                // Code structure
                '"' | '\'' => {

                }

                // UnexpectedCharacter
                _ => {
                    return Err(RuneWayError::new(
                        LexerErrorKind::UnexpectedCharacter,
                        format!("Lexer founded unexpected character: {}", char.to_string()),
                        self.line,
                    ));
                }
            }
        }
        Ok(tokens)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        self._tokenize().unwrap_or_else(|error| {
            panic!(
                "Caught error at line {line}\n{kind}: {message}",
                kind = error.kind.description(),
                message = error.message,
                line = error.line
            );
        })
    }
}
