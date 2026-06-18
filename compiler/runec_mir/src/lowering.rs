use runec_hir::ids::HirId;
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_semantic::typeck::{Ty, TypeInfo};

use crate::block::{MirBlock, MirTerminator};
use crate::function::MirFunction;
use crate::module::MirModule;
use crate::ty::{MirFloatTy, MirIntTy, MirTy};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MirLowerResult {
    pub module: MirModule,
    pub errors: Vec<MirLowerError>,
}

impl MirLowerResult {
    pub fn new() -> Self {
        Self {
            module: MirModule::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirLowerError {
    pub function: Option<HirId>,
    pub kind: MirLowerErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirLowerErrorKind {
    MissingFunctionSignature,
    UnsupportedType(Ty),
}

pub struct MirLowerer<'src, 'info> {
    type_info: &'info TypeInfo<'src>,
    res: MirLowerResult,
}

impl<'src, 'info> MirLowerer<'src, 'info> {
    pub fn new(type_info: &'info TypeInfo<'src>) -> Self {
        Self {
            type_info,
            res: MirLowerResult::new(),
        }
    }

    pub fn lower(mut self, hir: &HirMap<'src>) -> MirLowerResult {
        for (_, item) in hir.iter() {
            let HirItem::Function(function) = item else {
                continue;
            };

            if let Some(function) = self.lower_function_shell(function) {
                let function_id = self.res.module.push_function(function);
                if item.name().node == "main" {
                    self.res.module.entry = Some(function_id);
                }
            }
        }

        self.res
    }

    fn lower_function_shell(&mut self, function: &HirFunction<'src>) -> Option<MirFunction> {
        let Some(sig) = self.type_info.function_sig(function.id) else {
            self.push_error(function.id, MirLowerErrorKind::MissingFunctionSignature);
            return None;
        };

        let Some(ret_ty) = lower_ty(&sig.ret) else {
            self.push_error(
                function.id,
                MirLowerErrorKind::UnsupportedType(sig.ret.clone()),
            );
            return None;
        };

        let mut lowered = MirFunction::new(function.id, function.name.node, ret_ty);
        lowered.entry = lowered.push_block(MirBlock::new(MirTerminator::Return));
        Some(lowered)
    }

    fn push_error(&mut self, function: HirId, kind: MirLowerErrorKind) {
        self.res.errors.push(MirLowerError {
            function: Some(function),
            kind,
        });
    }
}

pub fn lower_ty(ty: &Ty) -> Option<MirTy> {
    match ty {
        Ty::Unit => Some(MirTy::Unit),
        Ty::Bool => Some(MirTy::Bool),
        Ty::Int { signed, bits } => Some(MirTy::Int(MirIntTy {
            signed: *signed,
            bits: *bits,
        })),
        Ty::Float { bits } => Some(MirTy::Float(MirFloatTy { bits: *bits })),
        Ty::Char => Some(MirTy::Char),
        Ty::Str => Some(MirTy::Str),
        Ty::Tuple(_)
        | Ty::Array { .. }
        | Ty::Struct(_)
        | Ty::Enum(_)
        | Ty::Function(_)
        | Ty::Builtin(_)
        | Ty::Unknown => None,
    }
}

#[cfg(test)]
mod tests;
