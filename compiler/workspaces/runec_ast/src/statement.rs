use runec_source::span::Spanned;
use crate::ast_type::TypeAnnotation;
use crate::expression::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt<'src> {
    SemiExpr(Expr<'src>),
    TailExpr(Expr<'src>),
    DefineLet {
        ident: &'src str,
        ty: TypeAnnotation<'src>,
        expr: Expr<'src>,
    },
    Assign {
        ident: &'src str,
        expr: Expr<'src>,
    },
    DefineConst {
        ident: &'src str,
        ty: TypeAnnotation<'src>,
        expr: Expr<'src>,
    },
    DefineFunction {
        ident: &'src str,
        args: Box<[FunctionArg<'src>]>,
        ret_ty: TypeAnnotation<'src>,
        body: SpannedStmtBlock<'src>,
    }
}

pub type StmtBlock<'src> = Box<[SpannedStmt<'src>]>;
pub type SpannedStmtBlock<'src> = Spanned<StmtBlock<'src>>;

#[derive(Debug, PartialEq)]
pub struct FunctionArg<'src> {
    pub ident: &'src str,
    // TODO: change to TypeAnnotation
    pub ty: &'src str,
}

pub type SpannedStmt<'src> = Spanned<Stmt<'src>>;
