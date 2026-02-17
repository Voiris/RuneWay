use runec_source::span::Spanned;
use crate::ast_type::SpannedTypeAnnotation;
use crate::expression::{Expr, SpannedExpr};
use crate::SpannedStr;

#[derive(Debug, PartialEq)]
pub enum Stmt<'src> {
    SemiExpr(SpannedExpr<'src>),
    TailExpr(SpannedExpr<'src>),
    DefineLet {
        ident: SpannedDestructPattern<'src>,
        ty: SpannedTypeAnnotation<'src>,
        init_expr: Option<SpannedExpr<'src>>,
    },
    Assign {
        ident: SpannedStr<'src>,
        expr: SpannedExpr<'src>,
    },
    DefineConst {
        ident: SpannedStr<'src>,
        ty: SpannedTypeAnnotation<'src>,
        expr: SpannedExpr<'src>,
    },
    DefineFunction {
        ident: SpannedStr<'src>,
        args: Box<[FunctionArg<'src>]>,
        ret_ty: SpannedTypeAnnotation<'src>,
        body: SpannedStmtBlock<'src>,
    }
}

pub type StmtBlock<'src> = Box<[SpannedStmt<'src>]>;
pub type SpannedStmtBlock<'src> = Spanned<StmtBlock<'src>>;

#[derive(Debug, PartialEq)]
pub struct FunctionArg<'src> {
    pub ident: SpannedStr<'src>,
    pub ty: SpannedTypeAnnotation<'src>,
}

pub type SpannedStmt<'src> = Spanned<Stmt<'src>>;

#[derive(Debug, PartialEq)]
pub enum DestructPattern<'src> {
    Ident(&'src str),
    Tuple(Box<[SpannedDestructPattern<'src>]>),
}

pub type SpannedDestructPattern<'src> = Spanned<DestructPattern<'src>>;
