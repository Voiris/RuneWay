use crate::function::MirCallee;
use crate::operand::{MirOperand, MirPlace};

#[derive(Debug, Clone, PartialEq)]
pub struct MirBlock {
    pub stmts: Vec<MirStmt>,
    pub terminator: MirTerminator,
}

impl MirBlock {
    pub fn new(terminator: MirTerminator) -> Self {
        Self {
            stmts: Vec::new(),
            terminator,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirStmt {
    Assign { dst: MirPlace, rhs: MirRvalue },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirRvalue {
    Use(MirOperand),
    Call {
        callee: MirCallee,
        args: Box<[MirOperand]>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirTerminator {
    Return,
}
