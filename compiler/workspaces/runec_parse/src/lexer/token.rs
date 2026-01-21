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
    CharLiteral(char),
    Ident(&'src str),

    /* Format strings control */
    FormatStringStart,
    FormatStringEnd,
    FormatCodeBlockStart,
    FormatCodeBlockEnd,

    /* Keywords */
    /// act
    Act,
    /// let
    Let,
    /// mut
    Mut,
    /// const
    Const,
    /// if
    If,
    /// else
    Else,
    /// for
    For,
    /// while
    While,
    /// loop
    Loop,
    /// break
    Break,
    /// continue
    Continue,
    /// return
    Return,
    /// true
    True,
    /// false
    False,
    /// null
    Null,
    /// as
    As,
    /// pub
    Pub,
    /// alias
    Alias,
    /// enum
    Enum,
    /// union
    Union,
    /// struct
    Struct,
    /// impl
    Impl,
    /// contract
    Contract,
    /// use
    Use,
    /// unsafe
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
