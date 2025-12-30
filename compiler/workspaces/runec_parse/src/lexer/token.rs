use runec_source::span::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
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
    NonNegIntLiteral(&'a str),  // >= 0
    NegIntLiteral(&'a str),     //  < 0
    FloatLiteral(&'a str),
    RawStringLiteral(&'a str),  // without escape sequence: r"string\n" or "string"
    StringLiteral(String),      // with escape sequence: "string\n"
    Ident(&'a str),

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
