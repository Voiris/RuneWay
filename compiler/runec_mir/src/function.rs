use runec_abi::RuntimeFunctionId;
use runec_hir::ids::HirId;
use runec_source::span::Span;

use crate::block::MirBlock;
use crate::ids::{MirBlockId, MirLocalId};
use crate::ty::MirTy;

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub hir_id: HirId,
    pub span: Span,
    pub name: Box<str>,
    pub params: Box<[MirLocalId]>,
    pub locals: Vec<MirLocal>,
    pub blocks: Vec<MirBlock>,
    pub entry: MirBlockId,
    pub ret_ty: MirTy,
    pub ret_span: Span,
}

impl MirFunction {
    pub fn new(
        hir_id: HirId,
        name: impl Into<Box<str>>,
        ret_ty: MirTy,
        span: Span,
        ret_span: Span,
    ) -> Self {
        Self {
            hir_id,
            span,
            name: name.into(),
            params: Box::new([]),
            locals: Vec::new(),
            blocks: Vec::new(),
            entry: MirBlockId::from_usize(0),
            ret_ty,
            ret_span,
        }
    }

    pub fn push_local(&mut self, ty: MirTy, span: Span) -> MirLocalId {
        let id = MirLocalId::from_usize(self.locals.len());
        self.locals.push(MirLocal { ty, span });
        id
    }

    pub fn push_block(&mut self, block: MirBlock) -> MirBlockId {
        let id = MirBlockId::from_usize(self.blocks.len());
        self.blocks.push(block);
        id
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirLocal {
    pub ty: MirTy,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MirCallee {
    Function(HirId),
    Runtime(RuntimeFunctionId),
}
