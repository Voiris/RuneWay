use std::borrow::Cow;
use runec_ast::expression::{FloatSuffix, IntSuffix};
use runec_source::span::Spanned;
use crate::ids::{HirId, HirLocalId};
use crate::path::HirPath;
use crate::statement::HirBlock;

#[derive(Debug, PartialEq)]
pub enum HirExpr<'src> {
    Literal(HirLiteral<'src>),

    /// Unresolved path — for bare identifiers (`println`) and qualified paths (`a::b::c`).
    /// The resolver replaces this with `Local(..)`, `FunctionRef(..)`, etc.
    Path(HirPath<'src>),

    /// Local variable or function parameter, filled in by the name resolver.
    Local(HirLocalId),

    /// Reference to a named function, filled in by the name resolver.
    FunctionRef(HirId),

    Call {
        callee: Box<SpannedHirExpr<'src>>,
        args: Box<[SpannedHirExpr<'src>]>,
    },

    Block(HirBlock<'src>),
}

pub type SpannedHirExpr<'src> = Spanned<HirExpr<'src>>;

#[derive(Debug, PartialEq)]
pub enum HirLiteral<'src> {
    Int   { value: u128, suffix: Option<IntSuffix> },
    Float { value: f64,  suffix: Option<FloatSuffix> },
    Bool(bool),
    Char(char),
    Str(Cow<'src, str>),
}
