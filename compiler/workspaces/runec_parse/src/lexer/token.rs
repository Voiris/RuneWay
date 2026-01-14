use runec_source::span::Spanned;

#[derive(PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum Radix {
    Binary,     // 0b
    Octal,      // 0o
    Decimal,
    Hex,        // 0x
}

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
    IntLiteral {                // >= 0
        digits: & 'src str,
        radix: Radix,
        suffix: Option<&'src str>,
    },
    FloatLiteral {              // >= 0.0
        literal: & 'src str,
        suffix: Option<&'src str>,
    },
    RawStringLiteral(&'src str),  // without escape sequence: r"string\n" or "string"
    StringLiteral(String),        // with escape sequence: "string\n"
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
