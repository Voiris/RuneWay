use runec_source::span::Spanned;

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Radix {
    Binary = 2, // 0b
    Octal = 8,  // 0o
    Decimal = 10,
    Hex = 16, // 0x
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Token<'src> {
    /* Expression-operator symbols. */
    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Bang,
    /// `~`
    Tilde,
    /// `?`
    Question,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^`
    Caret,
    /// `&`
    And,
    /// `|`
    Or,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,
    /// `%=`
    PercentEq,
    /// `^=`
    CaretEq,
    /// `&=`
    AndEq,
    /// `|=`
    OrEq,
    /// `<<=`
    ShlEq,
    /// `>>=`
    ShrEq,
    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,

    /* Literals */
    IntLiteral {
        // >= 0
        digits: &'src str,
        radix: Radix,
        suffix: Option<&'src str>,
    },
    FloatLiteral {
        // >= 0.0
        literal: &'src str,
        suffix: Option<&'src str>,
    },
    RawStringLiteral(&'src str), // without escape sequence: r"string\n" or "string"
    StringLiteral(String),       // with escape sequence: "string\n"
    Ident(&'src str),
    CharLiteral(char), // Simple

    /* Format strings control */
    FormatStringStart,
    FormatStringEnd,
    FormatCodeBlockStart,
    FormatCodeBlockEnd,

    /* Keywords */
    /// `act`
    Act,
    /// `let`
    Let,
    /// `mut`
    Mut,
    /// `const`
    Const,
    /// `if`
    If,
    /// `else`
    Else,
    /// `for`
    For,
    /// `while`
    While,
    /// `loop`
    Loop,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `return`
    Return,
    /// `true`
    True,
    /// `false`
    False,
    /// `null`
    Null,
    /// `as`
    As,
    /// `pub`
    Pub,
    /// `alias`
    Alias,
    /// `enum`
    Enum,
    /// `union`
    Union,
    /// `struct`
    Struct,
    /// `impl`
    Impl,
    /// `contract`
    Contract,
    /// `use`
    Use,
    /// `unsafe`
    Unsafe,

    /* Brackets */
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,

    /* Arrows */
    /// `->`
    Arrow,
    /// `=>`
    DArrow,

    /* Other */
    /// `.`
    Dot,
    /// `..`
    Range,
    /// `..=`
    RangeInclusive,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// ::
    DColon,
    /// `;`
    Semicolon,
}

impl Token<'_> {
    pub fn display(&self) -> &'static str {
        use token_display::*;

        match self {
            Token::Eq => EQ,
            Token::Lt => LT,
            Token::Le => LE,
            Token::EqEq => EQ_EQ,
            Token::Ne => NE,
            Token::Ge => GE,
            Token::Gt => GT,

            Token::AndAnd => AND_AND,
            Token::OrOr => OR_OR,

            Token::Bang => BANG,
            Token::Tilde => TILDE,
            Token::Question => QUESTION,

            Token::Plus => PLUS,
            Token::Minus => MINUS,
            Token::Star => STAR,
            Token::Slash => SLASH,
            Token::Percent => PERCENT,
            Token::Caret => CARET,
            Token::And => AND,
            Token::Or => OR,

            Token::Shl => SHL,
            Token::Shr => SHR,

            Token::PlusEq => PLUS_EQ,
            Token::MinusEq => MINUS_EQ,
            Token::StarEq => STAR_EQ,
            Token::SlashEq => SLASH_EQ,
            Token::PercentEq => PERCENT_EQ,
            Token::CaretEq => CARET_EQ,
            Token::AndEq => AND_EQ,
            Token::OrEq => OR_EQ,

            Token::ShlEq => SHL_EQ,
            Token::ShrEq => SHR_EQ,

            Token::PlusPlus => PLUS_PLUS,
            Token::MinusMinus => MINUS_MINUS,

            Token::CharLiteral(..) => CHAR_LITERAL,
            Token::StringLiteral(..) | Token::RawStringLiteral(..) => STRING_LITERAL,
            Token::IntLiteral { .. } => INT_LITERAL,
            Token::FloatLiteral { .. } => FLOAT_LITERAL,
            Token::Ident(..) => IDENTIFIER,

            Token::FormatStringStart | Token::FormatStringEnd => FORMAT_STRING,
            Token::FormatCodeBlockStart | Token::FormatCodeBlockEnd => FORMAT_CODE_BLOCK,

            Token::Act => ACT,
            Token::Let => LET,
            Token::Mut => MUT,
            Token::Const => CONST,

            Token::If => IF,
            Token::Else => ELSE,
            Token::For => FOR,
            Token::While => WHILE,
            Token::Loop => LOOP,

            Token::Break => BREAK,
            Token::Continue => CONTINUE,
            Token::Return => RETURN,

            Token::True => TRUE,
            Token::False => FALSE,
            Token::Null => NULL,

            Token::As => AS,
            Token::Pub => PUB,
            Token::Alias => ALIAS,

            Token::Enum => ENUM,
            Token::Union => UNION,
            Token::Struct => STRUCT,
            Token::Impl => IMPL,
            Token::Contract => CONTRACT,

            Token::Use => USE,
            Token::Unsafe => UNSAFE,

            Token::OpenParen => OPEN_PAREN,
            Token::CloseParen => CLOSE_PAREN,
            Token::OpenBrace => OPEN_BRACE,
            Token::CloseBrace => CLOSE_BRACE,
            Token::OpenBracket => OPEN_BRACKET,
            Token::CloseBracket => CLOSE_BRACKET,

            Token::Arrow => ARROW,
            Token::DArrow => DARROW,

            Token::Dot => DOT,
            Token::Range => RANGE,
            Token::RangeInclusive => RANGE_INCLUSIVE,

            Token::Comma => COMMA,
            Token::Colon => COLON,
            Token::DColon => DCOLON,
            Token::Semicolon => SEMICOLON,
        }
    }
}

#[allow(unused)]
pub mod token_display {
    pub const EQ: &str = "=";
    pub const LT: &str = "<";
    pub const LE: &str = "<=";
    pub const EQ_EQ: &str = "==";
    pub const NE: &str = "!=";
    pub const GE: &str = ">=";
    pub const GT: &str = ">";

    pub const AND_AND: &str = "&&";
    pub const OR_OR: &str = "||";

    pub const BANG: &str = "!";
    pub const TILDE: &str = "~";
    pub const QUESTION: &str = "?";

    pub const PLUS: &str = "+";
    pub const MINUS: &str = "-";
    pub const STAR: &str = "*";
    pub const SLASH: &str = "/";
    pub const PERCENT: &str = "%";
    pub const CARET: &str = "^";
    pub const AND: &str = "&";
    pub const OR: &str = "|";

    pub const SHL: &str = "<<";
    pub const SHR: &str = ">>";

    pub const PLUS_EQ: &str = "+=";
    pub const MINUS_EQ: &str = "-=";
    pub const STAR_EQ: &str = "*=";
    pub const SLASH_EQ: &str = "/=";
    pub const PERCENT_EQ: &str = "%=";
    pub const CARET_EQ: &str = "^=";
    pub const AND_EQ: &str = "&=";
    pub const OR_EQ: &str = "|=";

    pub const SHL_EQ: &str = "<<=";
    pub const SHR_EQ: &str = ">>=";

    pub const PLUS_PLUS: &str = "++";
    pub const MINUS_MINUS: &str = "--";

    pub const CHAR_LITERAL: &str = "char-literal";
    pub const STRING_LITERAL: &str = "string-literal";
    pub const INT_LITERAL: &str = "int-literal";
    pub const FLOAT_LITERAL: &str = "float-literal";
    pub const IDENTIFIER: &str = "identifier";

    pub const FORMAT_STRING: &str = "format-string";
    pub const FORMAT_CODE_BLOCK: &str = "format-code-block";

    pub const ACT: &str = "act";
    pub const LET: &str = "let";
    pub const MUT: &str = "mut";
    pub const CONST: &str = "const";

    pub const IF: &str = "if";
    pub const ELSE: &str = "else";
    pub const FOR: &str = "for";
    pub const WHILE: &str = "while";
    pub const LOOP: &str = "loop";

    pub const BREAK: &str = "break";
    pub const CONTINUE: &str = "continue";
    pub const RETURN: &str = "return";

    pub const TRUE: &str = "true";
    pub const FALSE: &str = "false";
    pub const NULL: &str = "null";

    pub const AS: &str = "as";
    pub const PUB: &str = "pub";
    pub const ALIAS: &str = "alias";

    pub const ENUM: &str = "enum";
    pub const UNION: &str = "union";
    pub const STRUCT: &str = "struct";
    pub const IMPL: &str = "impl";
    pub const CONTRACT: &str = "contract";

    pub const USE: &str = "use";
    pub const UNSAFE: &str = "unsafe";

    pub const ARROW: &str = "->";
    pub const DARROW: &str = "=>";

    pub const DOT: &str = ".";
    pub const RANGE: &str = "..";
    pub const RANGE_INCLUSIVE: &str = "..=";

    pub const COMMA: &str = ",";
    pub const COLON: &str = ":";
    pub const DCOLON: &str = "::";
    pub const SEMICOLON: &str = ";";

    pub const OPEN_PAREN: &str = "(";
    pub const CLOSE_PAREN: &str = ")";
    pub const OPEN_BRACE: &str = "{";
    pub const CLOSE_BRACE: &str = "}";
    pub const OPEN_BRACKET: &str = "[";
    pub const CLOSE_BRACKET: &str = "]";
}

pub type SpannedToken<'a> = Spanned<Token<'a>>;
