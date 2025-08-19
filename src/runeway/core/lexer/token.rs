use super::super::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::core::spanned::Spanned;

pub type SpannedToken = Spanned<Token>;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    // Code structure
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    UIntegerLiteral(u64),
    FloatLiteral(f64),

    // FStrings
    FString(Vec<FStringPart>),

    // Keywords
    Let,
    In,
    Null,
    Assert,

    // Classes
    Class,
    Static,

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

    // Imports
    Import,
    As,  // Alias
    Get, // Selective

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
    Percent, // %

    // Compound assignment operators
    PlusEqual,           // +=
    MinusEqual,          // -=
    AsteriskEqual,       // *=
    DoubleAsteriskEqual, // **=
    SlashEqual,          // /=
    PercentEqual,        // %=

    // Arrows
    Arrow,       // ->
    DoubleArrow, // =>

    // Brackets
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Other
    Comma,       // ,
    Dot,         // .
    Colon,       // :
    DoubleColon, // ::
    Semicolon,   // ;
    AtSymbol,    // @
    Is,

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
    EOF,
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
            Token::Is => Some(BinaryOperator::Is),

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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = match self {
            Token::StringLiteral(s) => format!("\"{}\"", s),
            Token::Identifier(s) => format!("`{}`", s),
            Token::IntegerLiteral(i) => format!("`{}i`", i),
            Token::UIntegerLiteral(u) => format!("`{}u`", u),
            Token::FloatLiteral(f) => format!("`{}f`", f),
            Token::FString(s) => format!("`{:?}`", s),

            Token::Let => "`let`".to_string(),
            Token::In => "`in`".to_string(),
            Token::Null => "`null`".to_string(),
            Token::Class => "`class`".to_string(),
            Token::Static => "`static`".to_string(),
            Token::Act => "`act`".to_string(),
            Token::Return => "`return`".to_string(),
            Token::If => "`if`".to_string(),
            Token::Else => "`else`".to_string(),
            Token::And => "`and`".to_string(),
            Token::Or => "`or`".to_string(),
            Token::Not => "`not`".to_string(),
            Token::Bang => "`!`".to_string(),
            Token::True => "`true`".to_string(),
            Token::False => "`false`".to_string(),
            Token::Import => "`import`".to_string(),
            Token::As => "`as`".to_string(),
            Token::Get => "`get`".to_string(),
            Token::For => "`for`".to_string(),
            Token::While => "`while`".to_string(),
            Token::Break => "`break`".to_string(),
            Token::Continue => "`continue`".to_string(),
            Token::Assert => "`assert`".to_string(),
            Token::EOF => "EOF".to_string(),
            Token::Equal => "`=`".to_string(),
            Token::EqualEqual => "`==`".to_string(),
            Token::NotEqual => "`!=`".to_string(),
            Token::Greater => "`>`".to_string(),
            Token::GreaterEqual => "`>=`".to_string(),
            Token::Less => "`<`".to_string(),
            Token::LessEqual => "`<=`".to_string(),
            Token::Plus => "`+`".to_string(),
            Token::Minus => "`-`".to_string(),
            Token::Asterisk => "`*`".to_string(),
            Token::DoubleAsterisk => "**".to_string(),
            Token::Slash => "`/`".to_string(),
            Token::Percent => "`%`".to_string(),
            Token::PlusEqual => "`+=`".to_string(),
            Token::MinusEqual => "`-=`".to_string(),
            Token::AsteriskEqual => "`*=`".to_string(),
            Token::DoubleAsteriskEqual => "`**=`".to_string(),
            Token::SlashEqual => "`/=`".to_string(),
            Token::PercentEqual => "`%=`".to_string(),
            Token::Arrow => "`->`".to_string(),
            Token::DoubleArrow => "`=>`".to_string(),
            Token::LParen => "`(`".to_string(),
            Token::RParen => "`)`".to_string(),
            Token::LBrace => "`{`".to_string(),
            Token::RBrace => "`}`".to_string(),
            Token::LBracket => "`[`".to_string(),
            Token::RBracket => "`]`".to_string(),
            Token::Comma => "`,`".to_string(),
            Token::Dot => "`.`".to_string(),
            Token::Colon => "`:`".to_string(),
            Token::DoubleColon => "`::`".to_string(),
            Token::Semicolon => "`;`".to_string(),
            Token::AtSymbol => "`@`".to_string(),
            Token::Is => "`is`".to_string(),
        };
        write!(f, "{}", string)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum FStringPart {
    StringLiteral(String),
    Expr(Vec<SpannedToken>, Vec<SpannedToken>), // expr:subcommand
}
