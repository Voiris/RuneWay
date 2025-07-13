use super::operators::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Types
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Box<Expr>>),
    FString(Vec<FStringExpr>),

    Iterator {
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
    },
    Null,

    // Operations
    BinaryOperation {
        left_operand: Box<Expr>,
        operator: BinaryOperator,
        right_operand: Box<Expr>
    },
    UnaryOperation {
        operator: UnaryOperator,
        operand: Box<Expr>
    },

    Expr(Box<Expr>),
    Variable(String),
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    GetAttr {
        object: Box<Expr>,
        field: String,
    },
    Slice {
        object: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringExpr {
    String(String),
    Expr(Expr),
}
