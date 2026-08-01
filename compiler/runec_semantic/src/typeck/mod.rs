use std::collections::HashMap;

use runec_ast::expression::{FloatSuffix, IntSuffix};
use runec_builtins::{
    BuiltinId, BuiltinReturn, ContractId, PrimitiveType, TypeBits, TypeConstraint, builtin_decl,
    primitive_implements,
};
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::DiagLabel;
use runec_errors::message::DiagMessage;
use runec_source::span::Span;

use runec_hir::expression::{HirExpr, HirLiteral, SpannedHirExpr};
use runec_hir::ids::{HirId, HirLocalId};
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_hir::resolution::Res;
use runec_hir::statement::{HirBlock, HirStmt};
use runec_hir::ty::{HirPrimitiveTy, HirType, SpannedHirType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Bool,
    Int { signed: bool, bits: TypeBits },
    Float { bits: TypeBits },
    Char,
    Str,
    Tuple(Box<[Ty]>),
    Array { elem: Box<Ty>, len: Option<u64> },
    Struct(HirId),
    Enum(HirId),
    Function(HirId),
    Builtin(BuiltinId),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSig {
    pub params: Box<[Ty]>,
    pub ret: Ty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalInfo<'src> {
    pub name: &'src str,
    pub ty: Ty,
    pub is_mutable: bool,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct TypeInfo<'src> {
    function_sigs: HashMap<HirId, FunctionSig>,
    locals: HashMap<HirId, Vec<LocalInfo<'src>>>,
}

impl<'src> TypeInfo<'src> {
    pub fn function_sig(&self, id: HirId) -> Option<&FunctionSig> {
        self.function_sigs.get(&id)
    }

    pub fn locals(&self, function: HirId) -> &[LocalInfo<'src>] {
        self.locals
            .get(&function)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn local(&self, function: HirId, local: HirLocalId) -> Option<&LocalInfo<'src>> {
        self.locals
            .get(&function)
            .and_then(|locals| locals.get(local.to_usize()))
    }

    pub fn ty_of_expr(&self, function: HirId, expr: &SpannedHirExpr<'src>) -> Ty {
        match &expr.node {
            HirExpr::Error => Ty::Unknown,
            HirExpr::Literal(literal) => ty_of_literal(literal),
            HirExpr::Resolved(res) => self.ty_of_res(function, *res),
            HirExpr::Call { callee, .. } => self.call_return_ty(function, callee),
            HirExpr::Block(block) => self.ty_of_block(function, block),
            HirExpr::Path(_) => Ty::Unknown,
        }
    }

    fn ty_of_res(&self, function: HirId, res: Res) -> Ty {
        match res {
            Res::Local(local) => self
                .local(function, local)
                .map(|local| local.ty.clone())
                .unwrap_or(Ty::Unknown),
            Res::Def(id) => Ty::Function(id),
            Res::Builtin(id) => Ty::Builtin(id),
        }
    }

    fn call_return_ty(&self, function: HirId, callee: &SpannedHirExpr<'src>) -> Ty {
        match self.ty_of_expr(function, callee) {
            Ty::Function(id) => self
                .function_sig(id)
                .map(|sig| sig.ret.clone())
                .unwrap_or(Ty::Unknown),
            Ty::Builtin(id) => builtin_decl(id)
                .map(|decl| builtin_return_ty(decl.ret))
                .unwrap_or(Ty::Unknown),
            _ => Ty::Unknown,
        }
    }

    pub fn ty_of_block(&self, function: HirId, block: &HirBlock<'src>) -> Ty {
        block
            .tail
            .as_ref()
            .map(|tail| self.ty_of_expr(function, tail))
            .unwrap_or(Ty::Unit)
    }
}

pub struct TypeCheckResult<'src> {
    pub info: TypeInfo<'src>,
    pub diags: Vec<Diagnostic<'static>>,
}

pub struct TypeChecker<'src> {
    info: TypeInfo<'src>,
    diags: Vec<Diagnostic<'static>>,
}

impl<'src> TypeChecker<'src> {
    pub fn new() -> Self {
        Self {
            info: TypeInfo::default(),
            diags: Vec::new(),
        }
    }

    pub fn check(mut self, hir: &HirMap<'src>) -> TypeCheckResult<'src> {
        self.collect_function_sigs(hir);

        for (_, item) in hir.iter() {
            if let HirItem::Function(function) = item {
                self.check_function(function);
            }
        }

        TypeCheckResult {
            info: self.info,
            diags: self.diags,
        }
    }

    fn collect_function_sigs(&mut self, hir: &HirMap<'src>) {
        for (id, item) in hir.iter() {
            if let HirItem::Function(function) = item {
                let params = function
                    .params
                    .iter()
                    .map(|param| self.lower_ty(&param.ty))
                    .collect();
                let ret = self.lower_ty(&function.ret_ty);
                self.info
                    .function_sigs
                    .insert(id, FunctionSig { params, ret });
            }
        }
    }

    fn check_function(&mut self, function: &HirFunction<'src>) {
        let mut locals = Vec::new();
        for param in function.params.iter() {
            locals.push(LocalInfo {
                name: param.name.node,
                ty: self.lower_ty(&param.ty),
                is_mutable: false,
                span: param.span,
            });
        }
        self.info.locals.insert(function.id, locals);

        self.check_block(function.id, &function.body);
        let actual = self.info.ty_of_block(function.id, &function.body);
        let expected = self
            .info
            .function_sig(function.id)
            .map(|sig| sig.ret.clone())
            .unwrap_or(Ty::Unknown);
        self.expect_assignable(function.body.span, expected, actual);
    }

    fn check_block(&mut self, function: HirId, block: &HirBlock<'src>) {
        for stmt in block.stmts.iter() {
            self.check_stmt(function, stmt);
        }

        if let Some(tail) = &block.tail {
            self.check_expr(function, tail);
        }
    }

    fn check_stmt(&mut self, function: HirId, stmt: &HirStmt<'src>) {
        match stmt {
            HirStmt::Expr(expr) => {
                self.check_expr(function, expr);
            }
            HirStmt::Let {
                local,
                name,
                is_mutable,
                ty,
                init,
                span,
            } => {
                let Some(local) = local else {
                    self.push_diag(messages::MISSING_LOCAL_ID, &[], *span);
                    return;
                };

                let declared = ty.as_ref().map(|ty| self.lower_ty(ty));
                let init_ty = init.as_ref().map(|expr| self.check_expr(function, expr));
                let final_ty = declared.clone().or(init_ty.clone()).unwrap_or(Ty::Unknown);

                if let (Some(expected), Some(actual)) = (declared, init_ty) {
                    self.expect_assignable(*span, expected, actual);
                }

                let locals = self.info.locals.entry(function).or_default();
                let local_idx = local.to_usize();
                if locals.len() == local_idx {
                    locals.push(LocalInfo {
                        name: name.node,
                        ty: final_ty,
                        is_mutable: *is_mutable,
                        span: *span,
                    });
                } else if let Some(info) = locals.get_mut(local_idx) {
                    info.ty = final_ty;
                } else {
                    let local = format!("{local:?}");
                    self.push_diag(messages::UNKNOWN_LOCAL, &[("local", &local)], *span);
                }
            }
        }
    }

    fn check_expr(&mut self, function: HirId, expr: &SpannedHirExpr<'src>) -> Ty {
        match &expr.node {
            HirExpr::Error => Ty::Unknown,
            HirExpr::Literal(literal) => ty_of_literal(literal),
            HirExpr::Resolved(res) => self.check_res(function, *res, expr.span),
            HirExpr::Path(_) => {
                self.push_diag(messages::UNRESOLVED_EXPRESSION, &[], expr.span);
                Ty::Unknown
            }
            HirExpr::Block(block) => {
                self.check_block(function, block);
                self.info.ty_of_block(function, block)
            }
            HirExpr::Call { callee, args } => self.check_call(function, callee, args, expr.span),
        }
    }

    fn check_res(&mut self, function: HirId, res: Res, span: Span) -> Ty {
        match res {
            Res::Local(local) => self
                .info
                .local(function, local)
                .map(|local| local.ty.clone())
                .unwrap_or_else(|| {
                    let local = format!("{local:?}");
                    self.push_diag(messages::UNKNOWN_LOCAL, &[("local", &local)], span);
                    Ty::Unknown
                }),
            Res::Def(id) => Ty::Function(id),
            Res::Builtin(id) => Ty::Builtin(id),
        }
    }

    fn check_call(
        &mut self,
        function: HirId,
        callee: &SpannedHirExpr<'src>,
        args: &[SpannedHirExpr<'src>],
        span: Span,
    ) -> Ty {
        match self.check_expr(function, callee) {
            Ty::Function(id) => {
                let Some(sig) = self.info.function_sig(id).cloned() else {
                    return Ty::Unknown;
                };
                self.check_arg_count(span, sig.params.len(), args.len());
                for (arg, expected) in args.iter().zip(sig.params.iter()) {
                    let actual = self.check_expr(function, arg);
                    self.expect_assignable(arg.span, expected.clone(), actual);
                }
                sig.ret
            }
            Ty::Builtin(builtin) => {
                let Some(decl) = builtin_decl(builtin) else {
                    return Ty::Unknown;
                };

                self.check_arg_count(span, decl.params.len(), args.len());
                for (arg, constraint) in args.iter().zip(decl.params.iter()) {
                    let actual = self.check_expr(function, arg);
                    self.check_constraint(arg.span, *constraint, actual);
                }
                builtin_return_ty(decl.ret)
            }
            Ty::Unknown => {
                for arg in args {
                    self.check_expr(function, arg);
                }
                Ty::Unknown
            }
            actual => {
                for arg in args {
                    self.check_expr(function, arg);
                }
                let actual = format!("{actual:?}");
                self.push_diag(messages::NOT_CALLABLE, &[("actual", &actual)], span);
                Ty::Unknown
            }
        }
    }

    fn check_arg_count(&mut self, span: Span, expected: usize, actual: usize) {
        if expected != actual {
            let expected = expected.to_string();
            let actual = actual.to_string();
            self.push_diag(
                messages::ARGUMENT_COUNT_MISMATCH,
                &[("expected", &expected), ("actual", &actual)],
                span,
            );
        }
    }

    fn expect_assignable(&mut self, span: Span, expected: Ty, actual: Ty) {
        if expected == Ty::Unknown || actual == Ty::Unknown || expected == actual {
            return;
        }

        let expected = format!("{expected:?}");
        let actual = format!("{actual:?}");
        self.push_diag(
            messages::TYPE_MISMATCH,
            &[("expected", &expected), ("actual", &actual)],
            span,
        );
    }

    fn check_constraint(&mut self, span: Span, constraint: TypeConstraint, actual: Ty) {
        let TypeConstraint::Implements(contract_id) = constraint;
        if actual == Ty::Unknown || ty_implements(&actual, contract_id) {
            return;
        }

        let contract = contract_id.to_string();
        let actual = format!("{actual:?}");
        self.push_diag(
            messages::CONTRACT_NOT_IMPLEMENTED,
            &[("actual", &actual), ("contract", &contract)],
            span,
        );
    }

    fn lower_ty(&mut self, ty: &SpannedHirType<'src>) -> Ty {
        match &ty.node {
            HirType::Error => Ty::Unknown,
            HirType::Primitive(primitive) => primitive_ty(*primitive),
            HirType::Struct { def, .. } => Ty::Struct(*def),
            HirType::Enum { def, .. } => Ty::Enum(*def),
            HirType::Unit => Ty::Unit,
            HirType::Tuple(items) => {
                let items = items.iter().map(|item| self.lower_ty(item)).collect();
                Ty::Tuple(items)
            }
            HirType::Array { elem, len } => Ty::Array {
                elem: Box::new(self.lower_ty(elem)),
                len: const_array_len(len),
            },
            HirType::Unresolved(_) => {
                self.push_diag(messages::UNRESOLVED_TYPE, &[], ty.span);
                Ty::Unknown
            }
        }
    }

    fn push_diag(&mut self, message: &'static str, replacements: &[(&str, &str)], span: Span) {
        self.diags.push(
            *Diagnostic::error(DiagMessage::new(message, replacements))
                .add_label(DiagLabel::silent_primary(span)),
        );
    }
}

impl<'src> Default for TypeChecker<'src> {
    fn default() -> Self {
        Self::new()
    }
}

fn builtin_return_ty(ret: BuiltinReturn) -> Ty {
    match ret {
        BuiltinReturn::Unit => Ty::Unit,
    }
}

fn ty_implements(ty: &Ty, contract_id: ContractId) -> bool {
    let primitive = match ty {
        Ty::Str => PrimitiveType::Str,
        _ => return false,
    };

    primitive_implements(primitive, contract_id)
}

pub fn ty_of_literal(literal: &HirLiteral<'_>) -> Ty {
    match literal {
        HirLiteral::Int { suffix, .. } => int_suffix_ty(*suffix),
        HirLiteral::Float { suffix, .. } => float_suffix_ty(*suffix),
        HirLiteral::Bool(_) => Ty::Bool,
        HirLiteral::Char(_) => Ty::Char,
        HirLiteral::Str(_) => Ty::Str,
    }
}

fn primitive_ty(ty: HirPrimitiveTy) -> Ty {
    match ty {
        HirPrimitiveTy::I8 => Ty::Int {
            signed: true,
            bits: TypeBits::B8,
        },
        HirPrimitiveTy::I16 => Ty::Int {
            signed: true,
            bits: TypeBits::B16,
        },
        HirPrimitiveTy::I32 => Ty::Int {
            signed: true,
            bits: TypeBits::B32,
        },
        HirPrimitiveTy::I64 => Ty::Int {
            signed: true,
            bits: TypeBits::B64,
        },
        HirPrimitiveTy::I128 => Ty::Int {
            signed: true,
            bits: TypeBits::B128,
        },
        HirPrimitiveTy::U8 => Ty::Int {
            signed: false,
            bits: TypeBits::B8,
        },
        HirPrimitiveTy::U16 => Ty::Int {
            signed: false,
            bits: TypeBits::B16,
        },
        HirPrimitiveTy::U32 => Ty::Int {
            signed: false,
            bits: TypeBits::B32,
        },
        HirPrimitiveTy::U64 => Ty::Int {
            signed: false,
            bits: TypeBits::B64,
        },
        HirPrimitiveTy::U128 => Ty::Int {
            signed: false,
            bits: TypeBits::B128,
        },
        HirPrimitiveTy::F32 => Ty::Float {
            bits: TypeBits::B32,
        },
        HirPrimitiveTy::F64 => Ty::Float {
            bits: TypeBits::B64,
        },
        HirPrimitiveTy::Bool => Ty::Bool,
        HirPrimitiveTy::Char => Ty::Char,
        HirPrimitiveTy::Str => Ty::Str,
    }
}

fn int_suffix_ty(suffix: Option<IntSuffix>) -> Ty {
    match suffix {
        Some(IntSuffix::U8) => Ty::Int {
            signed: false,
            bits: TypeBits::B8,
        },
        Some(IntSuffix::U16) => Ty::Int {
            signed: false,
            bits: TypeBits::B16,
        },
        Some(IntSuffix::U32) => Ty::Int {
            signed: false,
            bits: TypeBits::B32,
        },
        Some(IntSuffix::U64) => Ty::Int {
            signed: false,
            bits: TypeBits::B64,
        },
        Some(IntSuffix::U128) => Ty::Int {
            signed: false,
            bits: TypeBits::B128,
        },
        Some(IntSuffix::I8) => Ty::Int {
            signed: true,
            bits: TypeBits::B8,
        },
        Some(IntSuffix::I16) => Ty::Int {
            signed: true,
            bits: TypeBits::B16,
        },
        Some(IntSuffix::I32) | None => Ty::Int {
            signed: true,
            bits: TypeBits::B32,
        },
        Some(IntSuffix::I64) => Ty::Int {
            signed: true,
            bits: TypeBits::B64,
        },
        Some(IntSuffix::I128) => Ty::Int {
            signed: true,
            bits: TypeBits::B128,
        },
        Some(IntSuffix::F32) => Ty::Float {
            bits: TypeBits::B32,
        },
        Some(IntSuffix::F64) => Ty::Float {
            bits: TypeBits::B64,
        },
    }
}

fn float_suffix_ty(suffix: Option<FloatSuffix>) -> Ty {
    match suffix {
        Some(FloatSuffix::F32) => Ty::Float {
            bits: TypeBits::B32,
        },
        Some(FloatSuffix::F64) | None => Ty::Float {
            bits: TypeBits::B64,
        },
    }
}

fn const_array_len(expr: &SpannedHirExpr<'_>) -> Option<u64> {
    let HirExpr::Literal(HirLiteral::Int { value, .. }) = &expr.node else {
        return None;
    };

    u64::try_from(*value).ok()
}

mod messages;

#[cfg(test)]
mod tests {
    use runec_ast::SpannedStr;
    use runec_builtins::PRINTLN;
    use runec_source::byte_pos::BytePos;
    use runec_source::source_map::SourceId;
    use runec_source::span::{Span, Spanned};

    use runec_hir::expression::{HirExpr, HirLiteral};
    use runec_hir::ids::HirId;
    use runec_hir::item::{HirFunction, HirItem};
    use runec_hir::map::HirMap;
    use runec_hir::resolution::Res;
    use runec_hir::statement::{HirBlock, HirStmt};
    use runec_hir::ty::{HirPrimitiveTy, HirType};

    use super::{Ty, TypeChecker};

    const SRC: SourceId = SourceId::from_usize(0);

    fn sp(lo: usize, hi: usize) -> Span {
        Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), SRC)
    }

    fn s<T>(node: T) -> Spanned<T> {
        Spanned::new(node, sp(0, 0))
    }

    #[test]
    fn reports_function_return_type_mismatch() {
        let mut hir = HirMap::new();
        hir.push(HirItem::Function(HirFunction {
            id: HirId::from_usize(0),
            name: SpannedStr::new("main", sp(0, 0)),
            params: Box::new([]),
            ret_ty: s(HirType::Primitive(HirPrimitiveTy::I32)),
            body: HirBlock {
                stmts: Box::new([]),
                tail: Some(Box::new(s(HirExpr::Literal(HirLiteral::Bool(true))))),
                span: sp(0, 0),
            },
            span: sp(0, 0),
        }));

        let result = TypeChecker::new().check(&hir);
        assert_eq!(result.diags.len(), 1);
        assert_eq!(result.diags[0].labels[0].span, sp(0, 0));
        assert!(result.diags[0].message.message.contains("expected type"));
        assert!(result.diags[0].message.message.contains("Bool"));
    }

    #[test]
    fn accepts_string_for_display_builtin() {
        let mut hir = HirMap::new();
        hir.push(function_with_builtin_arg(HirLiteral::Str("hello".into())));

        let result = TypeChecker::new().check(&hir);
        assert!(result.diags.is_empty());
    }

    #[test]
    fn distinguishes_builtin_reference_from_call_result() {
        let function_id = HirId::from_usize(0);
        let builtin = s(HirExpr::Resolved(Res::Builtin(PRINTLN)));
        let call = s(HirExpr::Call {
            callee: Box::new(s(HirExpr::Resolved(Res::Builtin(PRINTLN)))),
            args: Box::new([s(HirExpr::Literal(HirLiteral::Str("hello".into())))]),
        });

        let mut hir = HirMap::new();
        hir.push(HirItem::Function(HirFunction {
            id: function_id,
            name: SpannedStr::new("main", sp(0, 0)),
            params: Box::new([]),
            ret_ty: s(HirType::Unit),
            body: HirBlock {
                stmts: Box::new([HirStmt::Expr(call)]),
                tail: None,
                span: sp(0, 0),
            },
            span: sp(0, 0),
        }));

        let result = TypeChecker::new().check(&hir);
        assert!(result.diags.is_empty());
        assert_eq!(
            result.info.ty_of_expr(function_id, &builtin),
            Ty::Builtin(PRINTLN)
        );

        let HirItem::Function(function) = hir.get(function_id) else {
            panic!("expected function");
        };
        let HirStmt::Expr(call) = &function.body.stmts[0] else {
            panic!("expected call");
        };
        assert_eq!(result.info.ty_of_expr(function_id, call), Ty::Unit);
    }

    #[test]
    fn rejects_type_without_display_impl() {
        let mut hir = HirMap::new();
        hir.push(function_with_builtin_arg(HirLiteral::Int {
            value: 42,
            suffix: None,
        }));

        let result = TypeChecker::new().check(&hir);
        assert_eq!(result.diags.len(), 1);
        assert!(
            result.diags[0]
                .message
                .message
                .contains("core::fmt::Display")
        );
    }

    fn function_with_builtin_arg(literal: HirLiteral<'static>) -> HirItem<'static> {
        HirItem::Function(HirFunction {
            id: HirId::from_usize(0),
            name: SpannedStr::new("main", sp(0, 0)),
            params: Box::new([]),
            ret_ty: s(HirType::Unit),
            body: HirBlock {
                stmts: Box::new([HirStmt::Expr(s(HirExpr::Call {
                    callee: Box::new(s(HirExpr::Resolved(Res::Builtin(PRINTLN)))),
                    args: Box::new([s(HirExpr::Literal(literal))]),
                }))]),
                tail: None,
                span: sp(0, 0),
            },
            span: sp(0, 0),
        })
    }
}
