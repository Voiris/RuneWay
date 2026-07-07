use std::borrow::Cow;

use runec_abi::RUNTIME_PRINT;
use runec_ast::SpannedStr;
use runec_builtins::{PRINT, TypeBits};
use runec_hir::expression::{HirExpr, HirLiteral};
use runec_hir::ids::{HirId, HirLocalId};
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_hir::resolution::Res;
use runec_hir::statement::{HirBlock, HirStmt};
use runec_hir::ty::{HirPrimitiveTy, HirType};
use runec_semantic::typeck::TypeChecker;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::SourceId;
use runec_source::span::{Span, Spanned};

use crate::block::{MirRvalue, MirStmt, MirTerminator};
use crate::constant::MirConstant;
use crate::function::MirCallee;
use crate::lowering::{MirLowerErrorKind, MirLowerer};
use crate::operand::{MirImmediate, MirOperand};
use crate::ty::{MirIntTy, MirTy};

const SRC: SourceId = SourceId::from_usize(0);

fn sp(lo: usize, hi: usize) -> Span {
    Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), SRC)
}

fn dummy() -> Span {
    sp(0, 0)
}

fn s<T>(node: T) -> Spanned<T> {
    Spanned::new(node, dummy())
}

fn empty_unit_function(name: &'static str) -> HirItem<'static> {
    function_with_body(HirId::from_usize(0), name, empty_body())
}

fn unit_function_with_body(body: HirBlock<'static>) -> HirItem<'static> {
    function_with_body(HirId::from_usize(0), "main", body)
}

fn function_with_body(id: HirId, name: &'static str, body: HirBlock<'static>) -> HirItem<'static> {
    function_with_ret_ty(id, name, s(HirType::Unit), body)
}

fn function_with_ret_ty(
    id: HirId,
    name: &'static str,
    ret_ty: Spanned<HirType<'static>>,
    body: HirBlock<'static>,
) -> HirItem<'static> {
    HirItem::Function(HirFunction {
        id,
        name: SpannedStr::new(name, dummy()),
        params: Box::new([]),
        ret_ty,
        body,
        span: dummy(),
    })
}

fn empty_body() -> HirBlock<'static> {
    HirBlock {
        stmts: Box::new([]),
        tail: None,
        span: dummy(),
    }
}

#[test]
fn unsupported_return_type_error_preserves_span() {
    let return_span = sp(10, 15);
    let mut hir = HirMap::new();
    hir.push(function_with_ret_ty(
        HirId::from_usize(0),
        "invalid",
        Spanned::new(HirType::Tuple(Box::new([])), return_span),
        empty_body(),
    ));

    let typeck = TypeChecker::new().check(&hir);
    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].span, return_span);
    assert!(matches!(
        result.errors[0].kind,
        MirLowerErrorKind::UnsupportedType(_)
    ));
}

#[test]
fn lower_empty_main_function_shell() {
    let mut hir = HirMap::new();
    hir.push(empty_unit_function("main"));

    let typeck = TypeChecker::new().check(&hir);
    assert!(typeck.errors.is_empty());

    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert!(result.errors.is_empty());
    assert_eq!(result.module.functions.len(), 1);
    assert_eq!(result.module.entry.map(|id| id.to_usize()), Some(0));

    let function = &result.module.functions[0];
    assert_eq!(function.hir_id, HirId::from_usize(0));
    assert_eq!(function.name.as_ref(), "main");
    assert_eq!(function.ret_ty, MirTy::Unit);
    assert_eq!(function.blocks.len(), 1);
    assert_eq!(function.blocks[0].terminator, MirTerminator::Return(None));
}

#[test]
fn lower_let_string_literal_to_local_assignment() {
    let local = HirLocalId::from_usize(0);
    let body = HirBlock {
        stmts: Box::new([HirStmt::Let {
            local: Some(local),
            name: SpannedStr::new("message", dummy()),
            is_mutable: false,
            ty: None,
            init: Some(s(HirExpr::Literal(HirLiteral::Str(Cow::Borrowed("hello"))))),
            span: dummy(),
        }]),
        tail: None,
        span: dummy(),
    };

    let mut hir = HirMap::new();
    hir.push(unit_function_with_body(body));

    let typeck = TypeChecker::new().check(&hir);
    assert!(typeck.errors.is_empty());

    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert!(result.errors.is_empty());
    assert_eq!(result.module.constants.len(), 1);
    assert_eq!(result.module.constants[0], MirConstant::Str("hello".into()));

    let function = &result.module.functions[0];
    assert_eq!(function.locals.len(), 1);
    assert_eq!(function.locals[0].ty, MirTy::Str);

    let MirStmt::Assign { dst, rhs, span } = &function.blocks[0].stmts[0];
    assert_eq!(*span, dummy());
    assert_eq!(dst.local.to_usize(), 0);
    assert_eq!(
        *rhs,
        MirRvalue::Use(MirOperand::Constant(crate::MirConstantId::from_usize(0)))
    );
}

#[test]
fn lower_print_builtin_call_to_runtime_call() {
    let body = HirBlock {
        stmts: Box::new([HirStmt::Expr(s(HirExpr::Call {
            callee: Box::new(s(HirExpr::Resolved(Res::Builtin(PRINT)))),
            args: Box::new([s(HirExpr::Literal(HirLiteral::Str(Cow::Borrowed("hello"))))]),
        }))]),
        tail: None,
        span: dummy(),
    };

    let mut hir = HirMap::new();
    hir.push(unit_function_with_body(body));

    let typeck = TypeChecker::new().check(&hir);
    assert!(typeck.errors.is_empty());

    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert!(result.errors.is_empty());
    assert_eq!(result.module.constants[0], MirConstant::Str("hello".into()));

    let function = &result.module.functions[0];
    assert_eq!(function.locals.len(), 1);
    assert_eq!(function.locals[0].ty, MirTy::Unit);

    let MirStmt::Assign { dst, rhs, span } = &function.blocks[0].stmts[0];
    assert_eq!(*span, dummy());
    assert_eq!(dst.local.to_usize(), 0);

    let MirRvalue::Call { callee, args } = rhs else {
        panic!("expected runtime call");
    };
    assert_eq!(*callee, MirCallee::Runtime(RUNTIME_PRINT));
    assert_eq!(
        args.as_ref(),
        [MirOperand::Constant(crate::MirConstantId::from_usize(0))]
    );
}

#[test]
fn lower_user_function_call_to_function_callee() {
    let foo_id = HirId::from_usize(0);
    let main_id = HirId::from_usize(1);
    let main_body = HirBlock {
        stmts: Box::new([HirStmt::Expr(s(HirExpr::Call {
            callee: Box::new(s(HirExpr::Resolved(Res::Def(foo_id)))),
            args: Box::new([]),
        }))]),
        tail: None,
        span: dummy(),
    };

    let mut hir = HirMap::new();
    hir.push(function_with_body(foo_id, "foo", empty_body()));
    hir.push(function_with_body(main_id, "main", main_body));

    let typeck = TypeChecker::new().check(&hir);
    assert!(typeck.errors.is_empty());

    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert!(result.errors.is_empty());
    assert_eq!(result.module.functions.len(), 2);
    assert_eq!(result.module.entry.map(|id| id.to_usize()), Some(1));

    let main = &result.module.functions[1];
    let MirStmt::Assign { dst, rhs, span } = &main.blocks[0].stmts[0];
    assert_eq!(*span, dummy());
    assert_eq!(dst.local.to_usize(), 0);

    let MirRvalue::Call { callee, args } = rhs else {
        panic!("expected function call");
    };
    assert_eq!(*callee, MirCallee::Function(foo_id));
    assert!(args.is_empty());
}

#[test]
fn lower_tail_expr_to_return_operand() {
    let body = HirBlock {
        stmts: Box::new([]),
        tail: Some(Box::new(s(HirExpr::Literal(HirLiteral::Int {
            value: 42,
            suffix: None,
        })))),
        span: dummy(),
    };

    let mut hir = HirMap::new();
    hir.push(function_with_ret_ty(
        HirId::from_usize(0),
        "answer",
        s(HirType::Primitive(HirPrimitiveTy::I32)),
        body,
    ));

    let typeck = TypeChecker::new().check(&hir);
    assert!(typeck.errors.is_empty());

    let result = MirLowerer::new(&typeck.info).lower(&hir);

    assert!(result.errors.is_empty());
    assert_eq!(
        result.module.functions[0].blocks[0].terminator,
        MirTerminator::Return(Some(MirOperand::Immediate(MirImmediate::Int {
            value: 42,
            ty: MirIntTy {
                signed: true,
                bits: TypeBits::B32,
            },
        })))
    );
}
