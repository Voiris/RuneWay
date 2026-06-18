use runec_ast::SpannedStr;
use runec_hir::ids::HirId;
use runec_hir::item::{HirFunction, HirItem};
use runec_hir::map::HirMap;
use runec_hir::statement::HirBlock;
use runec_hir::ty::HirType;
use runec_semantic::typeck::TypeChecker;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::SourceId;
use runec_source::span::{Span, Spanned};

use crate::lowering::MirLowerer;
use crate::ty::MirTy;

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
    HirItem::Function(HirFunction {
        id: HirId::from_usize(0),
        name: SpannedStr::new(name, dummy()),
        params: Box::new([]),
        ret_ty: s(HirType::Unit),
        body: HirBlock {
            stmts: Box::new([]),
            tail: None,
            span: dummy(),
        },
        span: dummy(),
    })
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
}
