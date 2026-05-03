use runec_source::span::Spanned;

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Radix {
    Binary  = 2,    // 0b
    Octal   = 8,    // 0o
    Decimal = 10,
    Hex     = 16,   // 0x
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
    IntLiteral {                // >= 0
        digits: &'src str,
        radix: Radix,
        suffix: Option<&'src str>,
    },
    FloatLiteral {              // >= 0.0
        literal: &'src str,
        suffix: Option<&'src str>,
    },
    RawStringLiteral(&'src str),  // without escape sequence: r"string\n" or "string"
    StringLiteral(String),        // with escape sequence: "string\n"
    Ident(&'src str),
    CharLiteral(char),  // Simple

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
        match self {
            Token::Eq => "=",
            Token::Lt => "<",
            Token::Le => "<=",
            Token::EqEq => "==",
            Token::Ne => "!=",
            Token::Ge => ">=",
            Token::Gt => ">",
            Token::AndAnd => "&&",
            Token::OrOr => "||",
            Token::Bang => "!",
            Token::Tilde => "~",
            Token::Question => "?",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Percent => "%",
            Token::Caret => "^",
            Token::And => "&",
            Token::Or => "|",
            Token::Shl => "<<",
            Token::Shr => ">>",
            Token::PlusEq => "+=",
            Token::MinusEq => "-=",
            Token::StarEq => "*=",
            Token::SlashEq => "/=",
            Token::PercentEq => "%=",
            Token::CaretEq => "^=",
            Token::AndEq => "&=",
            Token::OrEq => "|=",
            Token::ShlEq => "<<=",
            Token::ShrEq => ">>=",
            Token::PlusPlus => "++",
            Token::MinusMinus => "--",
            Token::CharLiteral( .. ) => "char-literal",
            Token::FormatStringStart => "format-string",
            Token::FormatStringEnd => "format-string",
            Token::FormatCodeBlockStart => "format-code-block",
            Token::FormatCodeBlockEnd => "format-code-block",
            Token::Act => "act",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Const => "const",
            Token::If => "if",
            Token::Else => "else",
            Token::For => "for",
            Token::While => "while",
            Token::Loop => "loop",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Return => "return",
            Token::True => "true",
            Token::False => "false",
            Token::Null => "null",
            Token::As => "as",
            Token::Pub => "pub",
            Token::Alias => "alias",
            Token::Enum => "enum",
            Token::Union => "union",
            Token::Struct => "struct",
            Token::Impl => "impl",
            Token::Use => "use",
            Token::Unsafe => "unsafe",
            Token::Contract => "contract",
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::OpenBrace => "{",
            Token::CloseBrace => "}",
            Token::OpenBracket => "[",
            Token::CloseBracket => "]",
            Token::Arrow => "->",
            Token::DArrow => "=>",
            Token::Dot => ".",
            Token::Range => "..",
            Token::RangeInclusive => "..=",
            Token::Comma => ",",
            Token::Colon => ":",
            Token::DColon => "::",
            Token::Semicolon => ";",
            Token::IntLiteral { .. } => "int-literal",
            Token::FloatLiteral { .. } => "float-literal",
            Token::RawStringLiteral( .. ) => "string-literal",
            Token::StringLiteral( .. ) => "string-literal",
            Token::Ident( .. ) => "identifier",
        }
    }
}

pub type SpannedToken<'a> = Spanned<Token<'a>>;
