use runec_ast::SpannedStr;
use runec_source::span::Span;
use crate::expression::SpannedHirExpr;
use crate::ty::SpannedHirType;

#[derive(Debug, PartialEq)]
pub struct HirPath<'src> {
    pub from_root: bool,
    pub segments: Box<[HirPathSegment<'src>]>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirPathSegment<'src> {
    pub name: SpannedStr<'src>,
    /// `None` — segment without `<...>`. `Some(..)` — segment with generics (possibly empty).
    pub generics: Option<Box<[HirGenericArg<'src>]>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum HirGenericArg<'src> {
    Type(SpannedHirType<'src>),
    /// For const generics: `Type<T, 3>`, `Type<T, N>`.
    Const(SpannedHirExpr<'src>),
}
