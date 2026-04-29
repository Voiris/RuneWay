use std::borrow::Cow;

use runec_ast::ast_type::{SpannedTypeAnnotation, TypeAnnotation};
use runec_ast::expression::{Expr, PrimitiveValue, SpannedExpr};
use runec_ast::statement::{DestructPattern, FunctionArg, Stmt, SpannedStmt, SpannedStmtBlock};
use runec_source::byte_pos::BytePos;
use runec_source::source_map::SourceId;
use runec_source::span::{Span, Spanned};

use crate::expression::{HirExpr, HirLiteral};
use crate::ids::HirId;
use crate::item::HirItem;
use crate::statement::HirStmt;
use crate::ty::HirType;
use super::HirLowerer;

const SRC: SourceId = SourceId::from_usize(0);

fn sp(lo: usize, hi: usize) -> Span {
    Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), SRC)
}

fn dummy() -> Span { sp(0, 0) }

fn s<T>(node: T) -> Spanned<T> { Spanned::new(node, dummy()) }

fn empty_block() -> SpannedStmtBlock<'static> {
    s(Box::new([]) as Box<[SpannedStmt<'static>]>)
}

fn unit_ty() -> SpannedTypeAnnotation<'static> { s(TypeAnnotation::Unit) }
fn ident_ty(name: &str) -> SpannedTypeAnnotation<'_> { s(TypeAnnotation::Ident(name)) }

fn ident_expr(name: &str) -> SpannedExpr<'_> { s(Expr::Ident(name)) }
fn int_expr(v: u128) -> SpannedExpr<'static> {
    s(Expr::Primitive(PrimitiveValue::Int { value: v, suffix: None }))
}

fn fn_stmt<'a>(
    name: &'a str,
    args: Box<[FunctionArg<'a>]>,
    ret_ty: SpannedTypeAnnotation<'a>,
    body: SpannedStmtBlock<'a>,
) -> SpannedStmt<'a> {
    s(Stmt::DefineFunction { ident: s(name), args, ret_ty, body })
}

#[test]
fn lower_empty_fn() {
    let stmts = [fn_stmt("main", Box::new([]), unit_ty(), empty_block())];
    let result = HirLowerer::new().lower(&stmts);

    assert!(result.diags.is_empty());
    assert_eq!(result.map.len(), 1);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else {
        panic!("expected HirItem::Function");
    };
    assert_eq!(f.id, HirId::from_usize(0));
    assert_eq!(f.name.node, "main");
    assert_eq!(f.params.len(), 0);
    assert!(matches!(f.ret_ty.node, HirType::Unit));
    assert_eq!(f.body.stmts.len(), 0);
    assert!(f.body.tail.is_none());
}

#[test]
fn lower_fn_with_params() {
    let args = Box::new([
        FunctionArg { ident: s("x"), ty: ident_ty("i32") },
        FunctionArg { ident: s("y"), ty: ident_ty("bool") },
    ]);
    let stmts = [fn_stmt("add", args, unit_ty(), empty_block())];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.node, "x");
    assert_eq!(f.params[1].name.node, "y");
    let HirType::Unresolved(ref p) = f.params[0].ty.node else { panic!() };
    assert_eq!(p.segments[0].name.node, "i32");
    let HirType::Unresolved(ref p) = f.params[1].ty.node else { panic!() };
    assert_eq!(p.segments[0].name.node, "bool");
}

#[test]
fn lower_fn_named_ret_type() {
    let stmts = [fn_stmt("f", Box::new([]), ident_ty("MyType"), empty_block())];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let HirType::Unresolved(ref path) = f.ret_ty.node else { panic!("expected Unresolved") };
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].name.node, "MyType");
    assert!(!path.from_root);
}

#[test]
fn lower_fn_int_literal_tail() {
    let body = s(Box::new([s(Stmt::TailExpr(int_expr(42)))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    assert_eq!(f.body.stmts.len(), 0);
    let tail = f.body.tail.as_ref().expect("block should have a tail expression");
    assert!(matches!(
        tail.node,
        HirExpr::Literal(HirLiteral::Int { value: 42, suffix: None })
    ));
}

#[test]
fn lower_fn_bool_literals() {
    for (prim, expected) in [
        (PrimitiveValue::True,  HirLiteral::Bool(true)),
        (PrimitiveValue::False, HirLiteral::Bool(false)),
    ] {
        let body = s(Box::new([s(Stmt::TailExpr(s(Expr::Primitive(prim))))]) as Box<[_]>);
        let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
        let result = HirLowerer::new().lower(&stmts);
        let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
        let tail = f.body.tail.as_ref().expect("should have tail");
        assert_eq!(tail.node, HirExpr::Literal(expected));
    }
}

#[test]
fn lower_fn_char_tail() {
    let body = s(Box::new([
        s(Stmt::TailExpr(s(Expr::Primitive(PrimitiveValue::Char('z')))))
    ]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("should have tail");
    assert!(matches!(tail.node, HirExpr::Literal(HirLiteral::Char('z'))));
}

#[test]
fn lower_fn_string_tail() {
    let body = s(Box::new([
        s(Stmt::TailExpr(s(Expr::Primitive(PrimitiveValue::String(Cow::Borrowed("hello"))))))
    ]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("should have tail");
    let HirExpr::Literal(HirLiteral::Str(ref s)) = tail.node else {
        panic!("expected Str literal");
    };
    assert_eq!(s.as_ref(), "hello");
}

#[test]
fn lower_semi_expr_becomes_stmt() {
    let body = s(Box::new([s(Stmt::SemiExpr(int_expr(7)))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    assert_eq!(f.body.stmts.len(), 1);
    assert!(f.body.tail.is_none());
    let HirStmt::Expr(ref e) = f.body.stmts[0] else { panic!("expected HirStmt::Expr") };
    assert!(matches!(e.node, HirExpr::Literal(HirLiteral::Int { value: 7, .. })));
}

#[test]
fn lower_tail_expr_not_last_becomes_stmt() {
    let body = s(Box::new([
        s(Stmt::TailExpr(int_expr(1))),
        s(Stmt::SemiExpr(int_expr(2))),
    ]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    assert_eq!(f.body.stmts.len(), 2);
    assert!(f.body.tail.is_none());
}

#[test]
fn lower_let_stmt() {
    let body = s(Box::new([s(Stmt::DefineLet {
        pattern: s(DestructPattern::Ident("x")),
        is_mutable: false,
        ty: Some(ident_ty("i32")),
        init_expr: Some(int_expr(10)),
    })]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    assert_eq!(f.body.stmts.len(), 1);
    let HirStmt::Let { name, is_mutable, ty, init, .. } = &f.body.stmts[0] else {
        panic!("expected HirStmt::Let");
    };
    assert_eq!(name.node, "x");
    assert!(!is_mutable);
    assert!(ty.is_some());
    let init_expr = init.as_ref().expect("should have init");
    assert!(matches!(init_expr.node, HirExpr::Literal(HirLiteral::Int { value: 10, .. })));
}

#[test]
fn lower_let_mut_no_ty_no_init() {
    let body = s(Box::new([s(Stmt::DefineLet {
        pattern: s(DestructPattern::Ident("y")),
        is_mutable: true,
        ty: None,
        init_expr: None,
    })]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let HirStmt::Let { name, is_mutable, ty, init, .. } = &f.body.stmts[0] else {
        panic!("expected HirStmt::Let");
    };
    assert_eq!(name.node, "y");
    assert!(*is_mutable);
    assert!(ty.is_none());
    assert!(init.is_none());
}

#[test]
fn lower_ident_expr_becomes_single_segment_path() {
    let body = s(Box::new([s(Stmt::TailExpr(ident_expr("foo")))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("should have tail");
    let HirExpr::Path(ref path) = tail.node else { panic!("expected Path") };
    assert!(!path.from_root);
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].name.node, "foo");
    assert!(path.segments[0].generics.is_none());
}

#[test]
fn lower_path_expr_preserves_segments() {
    let path_expr = s(Expr::Path(Box::new([s("a"), s("b"), s("c")])));
    let body = s(Box::new([s(Stmt::TailExpr(path_expr))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("should have tail");
    let HirExpr::Path(ref path) = tail.node else { panic!("expected Path") };
    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.segments[0].name.node, "a");
    assert_eq!(path.segments[1].name.node, "b");
    assert_eq!(path.segments[2].name.node, "c");
}

#[test]
fn lower_call_expr() {
    let call = s(Expr::Call {
        callee: Box::new(ident_expr("println")),
        args: Box::new([int_expr(1), int_expr(2)]),
    });
    let body = s(Box::new([s(Stmt::TailExpr(call))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("should have tail");
    let HirExpr::Call { ref callee, ref args } = tail.node else { panic!("expected Call") };
    assert!(matches!(callee.node, HirExpr::Path(_)));
    assert_eq!(args.len(), 2);
    assert!(matches!(args[0].node, HirExpr::Literal(HirLiteral::Int { value: 1, .. })));
    assert!(matches!(args[1].node, HirExpr::Literal(HirLiteral::Int { value: 2, .. })));
}

#[test]
fn lower_type_tuple() {
    let tuple_ty = s(TypeAnnotation::Tuple(Box::new([ident_ty("i32"), unit_ty()])));
    let stmts = [fn_stmt("f", Box::new([]), tuple_ty, empty_block())];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let HirType::Tuple(ref elems) = f.ret_ty.node else { panic!("expected Tuple") };
    assert_eq!(elems.len(), 2);
    assert!(matches!(elems[0].node, HirType::Unresolved(_)));
    assert!(matches!(elems[1].node, HirType::Unit));
}

#[test]
fn lower_type_array() {
    let arr_ty = s(TypeAnnotation::Array {
        item: Box::new(ident_ty("u8")),
        length: int_expr(16),
    });
    let stmts = [fn_stmt("f", Box::new([]), arr_ty, empty_block())];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let HirType::Array { ref elem, ref len } = f.ret_ty.node else { panic!("expected Array") };
    assert!(matches!(elem.node, HirType::Unresolved(_)));
    assert!(matches!(len.node, HirExpr::Literal(HirLiteral::Int { value: 16, .. })));
}

#[test]
fn lower_top_level_non_fn_stmts_ignored() {
    let stmts = [
        s(Stmt::DefineLet {
            pattern: s(DestructPattern::Ident("x")),
            is_mutable: false,
            ty: None,
            init_expr: None,
        }),
        s(Stmt::SemiExpr(int_expr(1))),
        s(Stmt::TailExpr(int_expr(2))),
    ];
    let result = HirLowerer::new().lower(&stmts);
    assert!(result.diags.is_empty());
    assert_eq!(result.map.len(), 0);
}

#[test]
fn lower_multiple_fns_get_distinct_ids() {
    let stmts = [
        fn_stmt("first",  Box::new([]), unit_ty(), empty_block()),
        fn_stmt("second", Box::new([]), unit_ty(), empty_block()),
    ];
    let result = HirLowerer::new().lower(&stmts);

    assert_eq!(result.map.len(), 2);
    let HirItem::Function(f0) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let HirItem::Function(f1) = result.map.get(HirId::from_usize(1)) else { panic!() };
    assert_eq!(f0.id, HirId::from_usize(0));
    assert_eq!(f1.id, HirId::from_usize(1));
    assert_eq!(f0.name.node, "first");
    assert_eq!(f1.name.node, "second");
}

#[test]
fn lower_nested_block_expr() {
    let inner = s(Box::new([s(Stmt::TailExpr(int_expr(99)))]) as Box<[_]>);
    let body = s(Box::new([s(Stmt::TailExpr(s(Expr::Block(inner))))]) as Box<[_]>);
    let stmts = [fn_stmt("f", Box::new([]), unit_ty(), body)];
    let result = HirLowerer::new().lower(&stmts);

    let HirItem::Function(f) = result.map.get(HirId::from_usize(0)) else { panic!() };
    let tail = f.body.tail.as_ref().expect("outer block should have tail");
    let HirExpr::Block(ref inner_block) = tail.node else { panic!("expected nested Block") };
    let inner_tail = inner_block.tail.as_ref().expect("inner block should have tail");
    assert!(matches!(inner_tail.node, HirExpr::Literal(HirLiteral::Int { value: 99, .. })));
}
