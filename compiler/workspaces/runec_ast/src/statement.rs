use runec_source::span::Spanned;
use crate::ast_type::SpannedTypeAnnotation;
use crate::expression::Expr;
use crate::SpannedStr;

#[derive(Debug, PartialEq)]
pub enum Stmt<'src> {
    SemiExpr(Expr<'src>),
    TailExpr(Expr<'src>),
    DefineLet {
        ident: SpannedStr<'src>,
        ty: SpannedTypeAnnotation<'src>,
        expr: Expr<'src>,
    },
    Assign {
        ident: SpannedStr<'src>,
        expr: Expr<'src>,
    },
    DefineConst {
        ident: SpannedStr<'src>,
        ty: SpannedTypeAnnotation<'src>,
        expr: Expr<'src>,
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
