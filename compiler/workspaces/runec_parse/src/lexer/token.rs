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
    IntLiteral(&'a str),
    UIntLiteral(&'a str),
    FloatLiteral(&'a str),
    StringLiteral(&'a str),
    Ident(&'a str),

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
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
}

pub type SpannedToken<'a> = Spanned<Token<'a>>;
