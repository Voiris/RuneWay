use std::borrow::Cow;

use runec_ast::expression::{FloatSuffix, IntSuffix};
use runec_source::span::Spanned;

use crate::path::HirPath;
use crate::resolution::Res;
use crate::statement::HirBlock;

#[derive(Debug, PartialEq)]
pub enum HirExpr<'src> {
    Error,
    Literal(HirLiteral<'src>),

    /// A path that has not been processed by name resolution yet.
    Path(HirPath<'src>),

    /// A value namespace path filled in by name resolution.
    Resolved(Res),

    Call {
        callee: Box<SpannedHirExpr<'src>>,
        args: Box<[SpannedHirExpr<'src>]>,
    },

    Block(HirBlock<'src>),
}

pub type SpannedHirExpr<'src> = Spanned<HirExpr<'src>>;

#[derive(Debug, PartialEq)]
pub enum HirLiteral<'src> {
    Int {
        value: u128,
        suffix: Option<IntSuffix>,
    },
    Float {
        value: f64,
        suffix: Option<FloatSuffix>,
    },
    Bool(bool),
    Char(char),
    Str(Cow<'src, str>),
}
