use crate::expression::Expr;

pub enum Stmt<'src> {
    SemiExpr(Expr<'src>),
    TailExpr(Expr<'src>),
    DefineLet {
        ident: &'src str,
        ty: &'src str,
        expr: Expr<'src>,
    },
    Assign {
        ident: &'src str,
        expr: Expr<'src>,
    },
    DefineConst {
        ident: &'src str,
        ty: &'src str,
        expr: Expr<'src>,
    },
    DefineFunction {
        ident: &'src str,
        args: Box<[FunctionArg<'src>]>,
        ret_ty: &'src str,
        body: StmtBlock<'src>,
    }
}

pub type StmtBlock<'src> = Box<[Stmt<'src>]>;

pub struct FunctionArg<'src> {
    pub ident: &'src str,
    pub ty: &'src str,
}
