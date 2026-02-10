#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    /* Arithmetic */
    Add,
    Sub,
    Mul,
    Div,

    /* Logical */
    Or,
    And,

    /* Compare */
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    /* Bitwise */
    BitOr,
    BitAnd,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    /* Arithmetic */
    Neg,
    Pos,

    /* Logical */
    Not,

    /* Bitwise */
    BitNot,

    /* Prefix */
    PrefInc,
    PrefDec,

    /* Postfix */
    PostInc,
    PostDec,
}
