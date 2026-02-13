use std::borrow::Cow;
use runec_source::span::Spanned;
use crate::operators::{BinaryOp, UnaryOp};
use crate::statement::{SpannedStmtBlock, StmtBlock};

#[derive(Debug, PartialEq)]
pub enum Expr<'src> {
    Primitive(PrimitiveValue<'src>),
    Block(SpannedStmtBlock<'src>),
    If(IfExpr<'src>),
    Ident(&'src str),
    Binary { lhs: Box<SpannedExpr<'src>>, rhs: Box<SpannedExpr<'src>>, op: BinaryOp },
    Unary { operand: Box<SpannedExpr<'src>>, op: UnaryOp },
}

pub type SpannedExpr<'src> = Spanned<Expr<'src>>;

#[derive(Debug, PartialEq)]
pub struct IfExpr<'src> {
    pub cond: Box<SpannedExpr<'src>>,
    pub then: StmtBlock<'src>,
    pub else_: Option<ElseBranch<'src>>,
}

#[derive(Debug, PartialEq)]
pub enum ElseBranch<'src> {
    Block(StmtBlock<'src>),
    If(Box<IfExpr<'src>>),
}

#[derive(Debug, PartialEq)]
pub enum Suffix {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveValue<'src> {
    True,
    False,
    Int { value: u128, suffix: Suffix },
    Float { value: f64, suffix: Suffix },
    Char(char),
    String(Cow<'src, str>)
}
