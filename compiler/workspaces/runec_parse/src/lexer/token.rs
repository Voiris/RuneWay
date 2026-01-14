use runec_source::span::Spanned;

#[derive(Debug, Clone, PartialEq)]
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

    /* Literals */
    IntLiteral(&'src str),      // >= 0
    OctalIntLiteral(&'src str), // >= 0o0
    HexIntLiteral(&'src str),   // >= 0x0
    BinIntLiteral(&'src str),   // >= 0b0
    FloatLiteral(&'src str),    // >= 0.0
    RawStringLiteral(&'src str),  // without escape sequence: r"string\n" or "string"
    StringLiteral(String),      // with escape sequence: "string\n"
    Ident(&'src str),

    /* Format strings control */
    FormatStringStart,
    FormatStringEnd,

    /* Keywords */
    /// act
    Act,
    /// let
    Let,
    /// if
    If,
    /// else
    Else,
    /// while
    While,
    /// loop
    Loop,
    /// return
    Return,
    /// true
    True,
    /// false
    False,

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

    /* Other */
    /// .
    Dot,
    /// ..
    Range,
    /// ..=
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

pub type SpannedToken<'a> = Spanned<Token<'a>>;
