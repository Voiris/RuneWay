use runec_source::span::Span;

use crate::function::MirCallee;
use crate::operand::{MirOperand, MirPlace};

#[derive(Debug, Clone, PartialEq)]
pub struct MirBlock {
    pub stmts: Vec<MirStmt>,
    pub terminator: MirTerminator,
}

impl MirBlock {
    pub fn new(terminator: MirTerminator) -> Self {
        Self { stmts: Vec::new(), terminator }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirStmt {
    Assign { dst: MirPlace, rhs: MirRvalue, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirRvalue {
    Use(MirOperand),
    Call { callee: MirCallee, args: Box<[MirOperand]> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirTerminator {
    Return(Option<MirOperand>),
}

#[cfg(test)]
mod tests {
    use super::{MirBlock, MirTerminator};

    #[test]
    fn constructs_empty_block_with_terminator() {
        let block = MirBlock::new(MirTerminator::Return(None));

        assert!(block.stmts.is_empty());
        assert_eq!(block.terminator, MirTerminator::Return(None));
    }
}
