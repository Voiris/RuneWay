use crate::ast_type::TypeAnnotation;
use crate::expression::Expr;

#[derive(Debug)]
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
        body: StmtBlock<'src>,
    }
}

pub type StmtBlock<'src> = Box<[Stmt<'src>]>;

#[derive(Debug)]
pub struct FunctionArg<'src> {
    pub ident: &'src str,
    // TODO: change to TypeAnnotation
    pub ty: &'src str,
}
