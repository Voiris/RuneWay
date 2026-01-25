use std::borrow::Cow;
use crate::statement::StmtBlock;

pub enum Expr<'src> {
    Primitive(PrimitiveValue<'src>),
    Block(StmtBlock<'src>),
    If(IfExpr<'src>),
}

pub struct IfExpr<'src> {
    pub cond: Box<Expr<'src>>,
    pub then: StmtBlock<'src>,
    pub else_: Option<ElseBranch<'src>>,
}

pub enum ElseBranch<'src> {
    Block(StmtBlock<'src>),
    If(Box<IfExpr<'src>>),
}

pub enum PrimitiveValue<'src> {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Char(char),
    String(Cow<'src, str>)
}
