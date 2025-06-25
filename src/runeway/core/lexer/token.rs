use super::super::ast::operators::{BinaryOperator, UnaryOperator};

#[derive(PartialEq, Debug)]
pub enum Token {
    // Code structure
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),

    // FStrings
    FStringStart,                       // f"
    FStringLiteral(String),
    FStringExprStart,                   // {
    FStringExprSubcommand(Vec<Token>),  // :TOKENS}
    FStringExprEnd,                     // }
    FStringEnd,                         // "

    // Keywords
    Let,
    In,
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
    Bang, // !

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
    // TildeSlash,     // ~/
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
    Colon,        // :
    DoubleColon,  // ::
    Semicolon,    // ;
    AtSymbol,     // @

    // Comments
    // DoubleSlash, // //
    // SlashStar,   // /*
    // StarSlash,   // */

    // Quotes
    // Quote,             // '
    // TripleQuote,       // '''
    // DoubleQuote,       // "
    // TripleDoubleQuote, // """

    // Special
    EOF
}

impl Token {
    pub fn to_binary_operator(&self) -> Option<BinaryOperator> {
        match self {
            Token::Plus => Some(BinaryOperator::Add),
            Token::Minus => Some(BinaryOperator::Sub),
            Token::Asterisk => Some(BinaryOperator::Mul),
            Token::DoubleAsterisk => Some(BinaryOperator::Pow),
            Token::Slash => Some(BinaryOperator::Div),
            Token::Percent => Some(BinaryOperator::Mod),

            Token::EqualEqual => Some(BinaryOperator::Eq),
            Token::NotEqual => Some(BinaryOperator::NotEq),
            Token::Greater => Some(BinaryOperator::Gt),
            Token::GreaterEqual => Some(BinaryOperator::GtEq),
            Token::Less => Some(BinaryOperator::Lt),
            Token::LessEqual => Some(BinaryOperator::LtEq),

            Token::And => Some(BinaryOperator::And),
            Token::Or => Some(BinaryOperator::Or),

            _ => None,
        }
    }

    pub fn to_unary_operator(&self) -> Option<UnaryOperator> {
        match self {
            Token::Not | Token::Bang => Some(UnaryOperator::Not),
            Token::Minus => Some(UnaryOperator::Neg),
            _ => None,
        }
    }
}
