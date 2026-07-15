use std::collections::HashMap;

use runec_builtins::{BuiltinLowering, builtin_decl};
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::DiagLabel;
use runec_errors::message::DiagMessage;
use runec_hir::expression::{HirExpr, HirLiteral, SpannedHirExpr};
use runec_hir::ids::{HirId, HirLocalId};
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_hir::resolution::Res;
use runec_hir::statement::{HirBlock, HirStmt};
use runec_semantic::typeck::{Ty, TypeInfo};
use runec_source::span::Span;

use crate::block::{MirBlock, MirRvalue, MirStmt, MirTerminator};
use crate::constant::MirConstant;
use crate::function::{MirCallee, MirFunction};
use crate::ids::MirLocalId;
use crate::module::MirModule;
use crate::operand::{MirImmediate, MirOperand, MirPlace};
use crate::ty::{MirFloatTy, MirIntTy, MirTy};

#[derive(Debug, Default)]
pub struct MirLowerResult<'src, 'diag> {
    pub module: MirModule<'src>,
    pub diags: Vec<Diagnostic<'diag>>,
}

impl<'src, 'diag> MirLowerResult<'src, 'diag> {
    pub fn new() -> Self {
        Self {
            module: MirModule::new(),
            diags: Vec::new(),
        }
    }
}

pub struct MirLowerer<'src, 'info, 'diag> {
    type_info: &'info TypeInfo<'src>,
    res: MirLowerResult<'src, 'diag>,
}

struct FunctionLowerCtx<'src, 'mir> {
    function: HirId,
    lowered: &'mir mut MirFunction<'src>,
    block: &'mir mut MirBlock,
    locals: &'mir mut HashMap<HirLocalId, MirLocalId>,
}

impl<'src, 'info, 'diag> MirLowerer<'src, 'info, 'diag> {
    pub fn new(type_info: &'info TypeInfo<'src>) -> Self {
        Self {
            type_info,
            res: MirLowerResult::new(),
        }
    }

    pub fn lower(mut self, hir: &HirMap<'src>) -> MirLowerResult<'src, 'diag> {
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

    fn lower_function(&mut self, function: &HirFunction<'src>) -> Option<MirFunction<'src>> {
        let Some(sig) = self.type_info.function_sig(function.id) else {
            self.push_diag(function.span, messages::MISSING_FUNCTION_SIGNATURE, &[]);
            return None;
        };

        let Some(ret_ty) = lower_ty(&sig.ret) else {
            self.push_unsupported_type(function.ret_ty.span, &sig.ret);
            return None;
        };

        let mut lowered = MirFunction::new(
            function.id,
            function.name.node,
            ret_ty,
            function.span,
            function.ret_ty.span,
        );
        let mut locals = HashMap::new();
        let mut params = Vec::with_capacity(function.params.len());

        for idx in 0..function.params.len() {
            let hir_local = HirLocalId::from_usize(idx);
            let Some(local) = self.type_info.local(function.id, hir_local) else {
                self.push_missing_local_info(function.params[idx].span, hir_local);
                continue;
            };
            let Some(ty) = lower_ty(&local.ty) else {
                self.push_unsupported_type(function.params[idx].ty.span, &local.ty);
                continue;
            };

            let mir_local = lowered.push_local(
                Some(function.params[idx].name.node),
                ty,
                function.params[idx].span,
            );
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
        lowered: &mut MirFunction<'src>,
        locals: &mut HashMap<HirLocalId, MirLocalId>,
    ) -> MirBlock {
        let mut mir_block = MirBlock::new(MirTerminator::Return(None));
        let mut ctx = FunctionLowerCtx {
            function,
            lowered,
            block: &mut mir_block,
            locals,
        };

        for stmt in block.stmts.iter() {
            self.lower_stmt(stmt, &mut ctx);
        }

        if let Some(tail) = &block.tail
            && let Some(operand) = self.lower_expr(tail, &mut ctx)
        {
            ctx.block.terminator = MirTerminator::Return(Some(operand));
        }

        mir_block
    }

    fn lower_stmt(&mut self, stmt: &HirStmt<'src>, ctx: &mut FunctionLowerCtx<'src, '_>) {
        match stmt {
            HirStmt::Expr(expr) => {
                let _ = self.lower_expr(expr, ctx);
            }
            HirStmt::Let {
                local,
                name,
                init,
                span,
                ..
            } => {
                let Some(hir_local) = local else {
                    self.push_diag(*span, messages::MISSING_LOCAL_ID, &[]);
                    return;
                };

                let Some(info) = self.type_info.local(ctx.function, *hir_local) else {
                    self.push_missing_local_info(*span, *hir_local);
                    return;
                };
                let Some(ty) = lower_ty(&info.ty) else {
                    self.push_unsupported_type(*span, &info.ty);
                    return;
                };

                let mir_local = ctx.lowered.push_local(Some(name.node), ty, *span);
                ctx.locals.insert(*hir_local, mir_local);

                if let Some(init) = init {
                    let Some(operand) = self.lower_expr(init, ctx) else {
                        return;
                    };
                    ctx.block.stmts.push(MirStmt::Assign {
                        dst: MirPlace::new(mir_local),
                        rhs: MirRvalue::Use(operand),
                        span: *span,
                    });
                }
            }
        }
    }

    fn lower_expr(
        &mut self,
        expr: &SpannedHirExpr<'src>,
        ctx: &mut FunctionLowerCtx<'src, '_>,
    ) -> Option<MirOperand> {
        match &expr.node {
            HirExpr::Literal(literal) => self.lower_literal(ctx.function, expr, literal),
            HirExpr::Resolved(Res::Local(local)) => {
                let Some(local) = ctx.locals.get(local).copied() else {
                    let local = format!("{local:?}");
                    self.push_diag(expr.span, messages::UNKNOWN_LOCAL, &[("local", &local)]);
                    return None;
                };
                Some(MirOperand::Copy(MirPlace::new(local)))
            }
            HirExpr::Block(inner) => {
                for stmt in inner.stmts.iter() {
                    self.lower_stmt(stmt, ctx);
                }

                inner
                    .tail
                    .as_ref()
                    .and_then(|tail| self.lower_expr(tail, ctx))
                    .or(Some(MirOperand::Immediate(MirImmediate::Unit)))
            }
            HirExpr::Path(_) => {
                self.push_unsupported_expr(expr.span, "unresolved path");
                None
            }
            HirExpr::Resolved(_) => {
                self.push_unsupported_expr(expr.span, "resolved item");
                None
            }
            HirExpr::Call { callee, args } => self.lower_call(expr, callee, args, ctx),
            HirExpr::Error => None,
        }
    }

    fn lower_call(
        &mut self,
        expr: &SpannedHirExpr<'src>,
        callee: &SpannedHirExpr<'src>,
        args: &[SpannedHirExpr<'src>],
        ctx: &mut FunctionLowerCtx<'src, '_>,
    ) -> Option<MirOperand> {
        let callee = self.lower_callee(ctx.function, callee)?;

        let args = args
            .iter()
            .map(|arg| self.lower_expr(arg, ctx))
            .collect::<Option<Box<[_]>>>()?;

        let ty = self.type_info.ty_of_expr(ctx.function, expr);
        let Some(ret_ty) = lower_ty(&ty) else {
            self.push_unsupported_type(expr.span, &ty);
            return None;
        };

        let dst = ctx.lowered.push_local(None, ret_ty, expr.span);
        ctx.block.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(dst),
            rhs: MirRvalue::Call { callee, args },
            span: expr.span,
        });

        Some(MirOperand::Copy(MirPlace::new(dst)))
    }

    fn lower_callee(
        &mut self,
        _function: HirId,
        callee: &SpannedHirExpr<'src>,
    ) -> Option<MirCallee> {
        match &callee.node {
            HirExpr::Resolved(Res::Def(id)) => Some(MirCallee::Function(*id)),
            HirExpr::Resolved(Res::Builtin(id)) => {
                let Some(decl) = builtin_decl(*id) else {
                    let builtin = format!("{id:?}");
                    self.push_diag(
                        callee.span,
                        messages::UNKNOWN_BUILTIN,
                        &[("builtin", &builtin)],
                    );
                    return None;
                };
                match decl.lowering {
                    BuiltinLowering::Runtime(runtime) => Some(MirCallee::Runtime(runtime)),
                }
            }
            _ => {
                self.push_unsupported_expr(callee.span, "call callee");
                None
            }
        }
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
                    .push_constant(MirConstant::Str(value.clone()));
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
                        self.push_unsupported_type(expr.span, &ty);
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
                        self.push_unsupported_type(expr.span, &ty);
                        None
                    }
                }
            }
        }
    }

    fn push_diag(&mut self, span: Span, message: &'static str, replacements: &[(&str, &str)]) {
        self.res.diags.push(
            *Diagnostic::error(DiagMessage::new(message, replacements))
                .add_label(DiagLabel::silent_primary(span)),
        );
    }

    fn push_missing_local_info(&mut self, span: Span, local: HirLocalId) {
        let local = format!("{local:?}");
        self.push_diag(span, messages::MISSING_LOCAL_INFO, &[("local", &local)]);
    }

    fn push_unsupported_expr(&mut self, span: Span, expression: &str) {
        self.push_diag(
            span,
            messages::UNSUPPORTED_EXPRESSION,
            &[("expression", expression)],
        );
    }

    fn push_unsupported_type(&mut self, span: Span, ty: &Ty) {
        let ty = format!("{ty:?}");
        self.push_diag(span, messages::UNSUPPORTED_TYPE, &[("ty", &ty)]);
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

mod messages;

#[cfg(test)]
mod tests;
