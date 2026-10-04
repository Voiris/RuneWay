use runec_abi::RuntimeFunctionId;
use runec_hir::ids::HirId;
use runec_source::span::Span;

use crate::block::MirBlock;
use crate::ids::{MirBlockId, MirLocalId};
use crate::ty::MirTy;

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction<'src> {
    pub hir_id: HirId,
    pub span: Span,
    pub name: &'src str,
    pub params: Box<[MirLocalId]>,
    pub locals: Vec<MirLocal<'src>>,
    pub blocks: Vec<MirBlock>,
    pub entry: MirBlockId,
    pub ret_ty: MirTy,
    pub ret_span: Span,
}

impl<'src> MirFunction<'src> {
    pub fn new(hir_id: HirId, name: &'src str, ret_ty: MirTy, span: Span, ret_span: Span) -> Self {
        Self {
            hir_id,
            span,
            name,
            params: Box::new([]),
            locals: Vec::new(),
            blocks: Vec::new(),
            entry: MirBlockId::from_usize(0),
            ret_ty,
            ret_span,
        }
    }

    pub fn push_local(&mut self, name: Option<&'src str>, ty: MirTy, span: Span) -> MirLocalId {
        let id = MirLocalId::from_usize(self.locals.len());
        self.locals.push(MirLocal { name, ty, span });
        id
    }

    pub fn push_block(&mut self, block: MirBlock) -> MirBlockId {
        let id = MirBlockId::from_usize(self.blocks.len());
        self.blocks.push(block);
        id
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirLocal<'src> {
    pub name: Option<&'src str>,
    pub ty: MirTy,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MirCallee {
    Function(HirId),
    Runtime(RuntimeFunctionId),
}

#[cfg(test)]
mod tests {
    use runec_hir::ids::HirId;
    use runec_source::byte_pos::BytePos;
    use runec_source::source_map::SourceId;
    use runec_source::span::Span;

    use super::{MirFunction, MirLocal};
    use crate::block::{MirBlock, MirTerminator};
    use crate::ids::{MirBlockId, MirLocalId};
    use crate::operand::{MirImmediate, MirOperand};
    use crate::ty::MirTy;

    #[test]
    fn initializes_function_metadata_and_storage() {
        let span =
            Span::new(BytePos::from_usize(1), BytePos::from_usize(8), SourceId::from_usize(0));
        let ret_span =
            Span::new(BytePos::from_usize(5), BytePos::from_usize(8), SourceId::from_usize(0));
        let hir_id = HirId::from_usize(2);

        let function = MirFunction::new(hir_id, "main", MirTy::Bool, span, ret_span);

        assert_eq!(function.hir_id, hir_id);
        assert_eq!(function.name, "main");
        assert_eq!(function.span, span);
        assert_eq!(function.ret_ty, MirTy::Bool);
        assert_eq!(function.ret_span, ret_span);
        assert_eq!(function.entry, MirBlockId::from_usize(0));
        assert!(function.params.is_empty());
        assert!(function.locals.is_empty());
        assert!(function.blocks.is_empty());
    }

    #[test]
    fn pushes_locals_with_consecutive_ids() {
        let span =
            Span::new(BytePos::from_usize(1), BytePos::from_usize(4), SourceId::from_usize(0));
        let mut function = MirFunction::new(HirId::from_usize(0), "main", MirTy::Unit, span, span);

        let first = function.push_local(Some("value"), MirTy::Bool, span);
        let second = function.push_local(None, MirTy::Char, span);

        assert_eq!(first, MirLocalId::from_usize(0));
        assert_eq!(second, MirLocalId::from_usize(1));
        assert_eq!(
            function.locals,
            [
                MirLocal { name: Some("value"), ty: MirTy::Bool, span },
                MirLocal { name: None, ty: MirTy::Char, span },
            ]
        );
    }

    #[test]
    fn pushes_blocks_with_consecutive_ids() {
        let span =
            Span::new(BytePos::from_usize(1), BytePos::from_usize(4), SourceId::from_usize(0));
        let mut function = MirFunction::new(HirId::from_usize(0), "main", MirTy::Unit, span, span);

        let first = function.push_block(MirBlock::new(MirTerminator::Return(None)));
        let second = function.push_block(MirBlock::new(MirTerminator::Return(Some(
            MirOperand::Immediate(MirImmediate::Unit),
        ))));

        assert_eq!(first, MirBlockId::from_usize(0));
        assert_eq!(second, MirBlockId::from_usize(1));
        assert_eq!(
            function.blocks,
            [
                MirBlock::new(MirTerminator::Return(None)),
                MirBlock::new(MirTerminator::Return(Some(MirOperand::Immediate(
                    MirImmediate::Unit,
                )))),
            ]
        );
    }
}
