use runec_ast::SpannedStr;
use runec_source::span::Span;
use crate::expression::SpannedHirExpr;
use crate::ty::SpannedHirType;

#[derive(Debug, PartialEq)]
pub struct HirBlock<'src> {
    pub stmts: Box<[HirStmt<'src>]>,
    /// Tail expression without `;` — the value of the block.
    pub tail: Option<Box<SpannedHirExpr<'src>>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum HirStmt<'src> {
    /// `expr;`
    Expr(SpannedHirExpr<'src>),

    /// `let [mut] name (: T)? (= init)?;`
    Let {
        name: SpannedStr<'src>,
        is_mutable: bool,
        ty: Option<SpannedHirType<'src>>,
        init: Option<SpannedHirExpr<'src>>,
        span: Span,
    },
}
