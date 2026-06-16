use runec_abi::RuntimeFunctionId;
use runec_hir::ids::HirId;

use crate::block::MirBlock;
use crate::ids::{MirBlockId, MirLocalId};
use crate::ty::MirTy;

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub hir_id: HirId,
    pub name: Box<str>,
    pub params: Box<[MirLocalId]>,
    pub locals: Vec<MirLocal>,
    pub blocks: Vec<MirBlock>,
    pub entry: MirBlockId,
    pub ret_ty: MirTy,
}

impl MirFunction {
    pub fn new(hir_id: HirId, name: impl Into<Box<str>>, ret_ty: MirTy) -> Self {
        Self {
            hir_id,
            name: name.into(),
            params: Box::new([]),
            locals: Vec::new(),
            blocks: Vec::new(),
            entry: MirBlockId::from_usize(0),
            ret_ty,
        }
    }

    pub fn push_local(&mut self, ty: MirTy) -> MirLocalId {
        let id = MirLocalId::from_usize(self.locals.len());
        self.locals.push(MirLocal { ty });
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MirCallee {
    Function(HirId),
    Runtime(RuntimeFunctionId),
}
