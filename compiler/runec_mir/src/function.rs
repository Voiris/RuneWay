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
    use crate::ids::MirLocalId;
    use crate::ty::MirTy;

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
}
