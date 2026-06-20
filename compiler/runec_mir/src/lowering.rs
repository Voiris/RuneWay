use std::collections::HashMap;

use runec_builtins::{BuiltinLowering, builtin_decl};
use runec_hir::expression::{HirExpr, HirLiteral, SpannedHirExpr};
use runec_hir::ids::{HirId, HirLocalId};
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_hir::resolution::Res;
use runec_hir::statement::{HirBlock, HirStmt};
use runec_semantic::typeck::{Ty, TypeInfo};

use crate::block::{MirBlock, MirRvalue, MirStmt, MirTerminator};
use crate::constant::MirConstant;
use crate::function::{MirCallee, MirFunction};
use crate::ids::MirLocalId;
use crate::module::MirModule;
use crate::operand::{MirImmediate, MirOperand, MirPlace};
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
    MissingLocalId,
    MissingLocalInfo(HirLocalId),
    UnknownBuiltin(runec_builtins::BuiltinId),
    UnknownLocal(HirLocalId),
    UnsupportedExpr(&'static str),
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

            if let Some(function) = self.lower_function(function) {
                let function_id = self.res.module.push_function(function);
                if item.name().node == "main" {
                    self.res.module.entry = Some(function_id);
                }
            }
        }

        self.res
    }

    fn lower_function(&mut self, function: &HirFunction<'src>) -> Option<MirFunction> {
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
        let mut locals = HashMap::new();
        let mut params = Vec::with_capacity(function.params.len());

        for idx in 0..function.params.len() {
            let hir_local = HirLocalId::from_usize(idx);
            let Some(local) = self.type_info.local(function.id, hir_local) else {
                self.push_error(function.id, MirLowerErrorKind::MissingLocalInfo(hir_local));
                continue;
            };
            let Some(ty) = lower_ty(&local.ty) else {
                self.push_error(
                    function.id,
                    MirLowerErrorKind::UnsupportedType(local.ty.clone()),
                );
                continue;
            };

            let mir_local = lowered.push_local(ty);
            locals.insert(hir_local, mir_local);
            params.push(mir_local);
        }

        lowered.params = params.into_boxed_slice();

        let block = self.lower_block(function.id, &function.body, &mut lowered, &mut locals);
        lowered.entry = lowered.push_block(block);
        Some(lowered)
    }

    fn lower_block(
        &mut self,
        function: HirId,
        block: &HirBlock<'src>,
        lowered: &mut MirFunction,
        locals: &mut HashMap<HirLocalId, MirLocalId>,
    ) -> MirBlock {
        let mut mir_block = MirBlock::new(MirTerminator::Return);

        for stmt in block.stmts.iter() {
            self.lower_stmt(function, stmt, lowered, &mut mir_block, locals);
        }

        if let Some(tail) = &block.tail {
            let _ = self.lower_expr(function, tail, lowered, &mut mir_block, locals);
        }

        mir_block
    }

    fn lower_stmt(
        &mut self,
        function: HirId,
        stmt: &HirStmt<'src>,
        lowered: &mut MirFunction,
        block: &mut MirBlock,
        locals: &mut HashMap<HirLocalId, MirLocalId>,
    ) {
        match stmt {
            HirStmt::Expr(expr) => {
                let _ = self.lower_expr(function, expr, lowered, block, locals);
            }
            HirStmt::Let { local, init, .. } => {
                let Some(hir_local) = local else {
                    self.push_error(function, MirLowerErrorKind::MissingLocalId);
                    return;
                };

                let Some(info) = self.type_info.local(function, *hir_local) else {
                    self.push_error(function, MirLowerErrorKind::MissingLocalInfo(*hir_local));
                    return;
                };
                let Some(ty) = lower_ty(&info.ty) else {
                    self.push_error(
                        function,
                        MirLowerErrorKind::UnsupportedType(info.ty.clone()),
                    );
                    return;
                };

                let mir_local = lowered.push_local(ty);
                locals.insert(*hir_local, mir_local);

                if let Some(init) = init {
                    let Some(operand) = self.lower_expr(function, init, lowered, block, locals)
                    else {
                        return;
                    };
                    block.stmts.push(MirStmt::Assign {
                        dst: MirPlace::new(mir_local),
                        rhs: MirRvalue::Use(operand),
                    });
                }
            }
        }
    }

    fn lower_expr(
        &mut self,
        function: HirId,
        expr: &SpannedHirExpr<'src>,
        lowered: &mut MirFunction,
        block: &mut MirBlock,
        locals: &mut HashMap<HirLocalId, MirLocalId>,
    ) -> Option<MirOperand> {
        match &expr.node {
            HirExpr::Literal(literal) => self.lower_literal(function, expr, literal),
            HirExpr::Resolved(Res::Local(local)) => {
                let Some(local) = locals.get(local).copied() else {
                    self.push_error(function, MirLowerErrorKind::UnknownLocal(*local));
                    return None;
                };
                Some(MirOperand::Copy(MirPlace::new(local)))
            }
            HirExpr::Block(inner) => {
                for stmt in inner.stmts.iter() {
                    self.lower_stmt(function, stmt, lowered, block, locals);
                }

                inner
                    .tail
                    .as_ref()
                    .and_then(|tail| self.lower_expr(function, tail, lowered, block, locals))
                    .or(Some(MirOperand::Immediate(MirImmediate::Unit)))
            }
            HirExpr::Path(_) => {
                self.push_error(
                    function,
                    MirLowerErrorKind::UnsupportedExpr("unresolved path"),
                );
                None
            }
            HirExpr::Resolved(_) => {
                self.push_error(
                    function,
                    MirLowerErrorKind::UnsupportedExpr("resolved item"),
                );
                None
            }
            HirExpr::Call { callee, args } => {
                self.lower_call(function, expr, callee, args, lowered, block, locals)
            }
        }
    }

    fn lower_call(
        &mut self,
        function: HirId,
        expr: &SpannedHirExpr<'src>,
        callee: &SpannedHirExpr<'src>,
        args: &[SpannedHirExpr<'src>],
        lowered: &mut MirFunction,
        block: &mut MirBlock,
        locals: &mut HashMap<HirLocalId, MirLocalId>,
    ) -> Option<MirOperand> {
        let callee = match &callee.node {
            HirExpr::Resolved(Res::Def(id)) => MirCallee::Function(*id),
            HirExpr::Resolved(Res::Builtin(id)) => {
                let Some(decl) = builtin_decl(*id) else {
                    self.push_error(function, MirLowerErrorKind::UnknownBuiltin(*id));
                    return None;
                };
                match decl.lowering {
                    BuiltinLowering::Runtime(runtime) => MirCallee::Runtime(runtime),
                }
            }
            _ => {
                self.push_error(function, MirLowerErrorKind::UnsupportedExpr("call callee"));
                return None;
            }
        };

        let args = args
            .iter()
            .map(|arg| self.lower_expr(function, arg, lowered, block, locals))
            .collect::<Option<Box<[_]>>>()?;

        let ty = self.type_info.ty_of_expr(function, expr);
        let Some(ret_ty) = lower_ty(&ty) else {
            self.push_error(function, MirLowerErrorKind::UnsupportedType(ty));
            return None;
        };

        let dst = lowered.push_local(ret_ty);
        block.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(dst),
            rhs: MirRvalue::Call { callee, args },
        });

        Some(MirOperand::Copy(MirPlace::new(dst)))
    }

    fn lower_literal(
        &mut self,
        function: HirId,
        expr: &SpannedHirExpr<'src>,
        literal: &HirLiteral<'src>,
    ) -> Option<MirOperand> {
        match literal {
            HirLiteral::Bool(value) => Some(MirOperand::Immediate(MirImmediate::Bool(*value))),
            HirLiteral::Char(value) => Some(MirOperand::Immediate(MirImmediate::Char(*value))),
            HirLiteral::Str(value) => {
                let id = self
                    .res
                    .module
                    .push_constant(MirConstant::Str(value.as_ref().into()));
                Some(MirOperand::Constant(id))
            }
            HirLiteral::Int { value, .. } => {
                let ty = self.type_info.ty_of_expr(function, expr);
                match lower_ty(&ty) {
                    Some(MirTy::Int(ty)) => Some(MirOperand::Immediate(MirImmediate::Int {
                        value: *value,
                        ty,
                    })),
                    _ => {
                        self.push_error(function, MirLowerErrorKind::UnsupportedType(ty));
                        None
                    }
                }
            }
            HirLiteral::Float { value, .. } => {
                let ty = self.type_info.ty_of_expr(function, expr);
                match lower_ty(&ty) {
                    Some(MirTy::Float(ty)) => Some(MirOperand::Immediate(MirImmediate::Float {
                        value: *value,
                        ty,
                    })),
                    _ => {
                        self.push_error(function, MirLowerErrorKind::UnsupportedType(ty));
                        None
                    }
                }
            }
        }
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
