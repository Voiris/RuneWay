use super::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::core::spanned::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Types
    Integer(i64),
    UInteger(u64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Box<SpannedExpr>>),
    Tuple(Vec<Box<SpannedExpr>>),
    // Set(Vec<Box<SpannedExpr>>),
    Dict(Vec<(Box<SpannedExpr>, Box<SpannedExpr>)>),
    FString(Vec<FStringExpr>),

    Iterator {
        start: Box<SpannedExpr>,
        end: Box<SpannedExpr>,
        step: Option<Box<SpannedExpr>>,
    },
    Null,

    // Operations
    BinaryOperation {
        left_operand: Box<SpannedExpr>,
        operator: BinaryOperator,
        right_operand: Box<SpannedExpr>,
    },
    UnaryOperation {
        operator: UnaryOperator,
        operand: Box<SpannedExpr>,
    },

    Expr(Box<SpannedExpr>),
    Variable(String),
    Call {
        callee: Box<SpannedExpr>,
        arguments: Vec<SpannedExpr>,
    },
    AttributeAccess {
        object: Box<SpannedExpr>,
        field: String,
    },
    SetAttr {
        object: Box<SpannedExpr>,
        value: Box<SpannedExpr>,
    },
    Slice {
        object: Box<SpannedExpr>,
        index: Box<SpannedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringExpr {
    String(String),
    Expr(SpannedExpr),
}

pub type SpannedExpr = Spanned<Expr>;
