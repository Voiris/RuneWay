use crate::SpannedStr;
use crate::ast_type::SpannedTypeAnnotation;
use crate::operators::{BinaryOp, UnaryOp};
use crate::statement::{SpannedStmtBlock, StmtBlock};
use runec_source::span::Spanned;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum Expr<'src> {
    Primitive(PrimitiveValue<'src>),
    Block(SpannedStmtBlock<'src>),
    If(IfExpr<'src>),
    Ident(&'src str),
    Path(Box<[SpannedStr<'src>]>),
    TypeCast {
        from: Box<SpannedExpr<'src>>,
        ty: Box<SpannedTypeAnnotation<'src>>,
    },
    Call {
        callee: Box<SpannedExpr<'src>>,
        args: Box<[SpannedExpr<'src>]>,
    },
    Binary {
        lhs: Box<SpannedExpr<'src>>,
        rhs: Box<SpannedExpr<'src>>,
        op: BinaryOp,
    },
    Unary {
        operand: Box<SpannedExpr<'src>>,
        op: UnaryOp,
    },
    Tuple(Box<[SpannedExpr<'src>]>),
    FullyDefinedArray(Box<[SpannedExpr<'src>]>),
    RepeatingArray {
        value: Box<SpannedExpr<'src>>,
        count: Box<SpannedExpr<'src>>,
    },
    Deref(Box<SpannedExpr<'src>>),
    AttributeAccess {
        value: Box<SpannedExpr<'src>>,
        name: SpannedStr<'src>,
    },
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntSuffix {
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
    // 0f64 - int with float suffix
    F32,
    F64,
}

impl IntSuffix {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "u8" => Some(IntSuffix::U8),
            "u16" => Some(IntSuffix::U16),
            "u32" => Some(IntSuffix::U32),
            "u64" => Some(IntSuffix::U64),
            "u128" => Some(IntSuffix::U128),
            "i8" => Some(IntSuffix::I8),
            "i16" => Some(IntSuffix::I16),
            "i32" => Some(IntSuffix::I32),
            "i64" => Some(IntSuffix::I64),
            "i128" => Some(IntSuffix::I128),
            "f32" => Some(IntSuffix::F32),
            "f64" => Some(IntSuffix::F64),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FloatSuffix {
    F32,
    F64,
}

impl FloatSuffix {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "f32" => Some(FloatSuffix::F32),
            "f64" => Some(FloatSuffix::F64),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveValue<'src> {
    True,
    False,
    Int {
        value: u128,
        suffix: Option<IntSuffix>,
    },
    Float {
        value: f64,
        suffix: Option<FloatSuffix>,
    },
    Char(char),
    String(Cow<'src, str>),
}
