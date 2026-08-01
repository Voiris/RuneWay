use std::collections::HashMap;

use runec_builtins::builtin_from_name;
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::DiagLabel;
use runec_errors::message::DiagMessage;
use runec_source::span::Span;

use runec_hir::expression::{HirExpr, SpannedHirExpr};
use runec_hir::ids::{HirId, HirLocalId};
use runec_hir::item::{HirItem, HirVariantPayload};
use runec_hir::map::HirMap;
use runec_hir::resolution::Res;
use runec_hir::statement::{HirBlock, HirStmt};
use runec_hir::ty::{HirPrimitiveTy, HirType, SpannedHirType};

pub struct ResolveResult {
    pub diags: Vec<Diagnostic<'static>>,
}

pub struct Resolver<'src> {
    items: HashMap<&'src str, ResolvedItem>,
    diags: Vec<Diagnostic<'static>>,
}

#[derive(Debug, Copy, Clone)]
struct ResolvedItem {
    id: HirId,
    kind: ResolvedItemKind,
}

#[derive(Debug, Copy, Clone)]
enum ResolvedItemKind {
    Function,
    Struct,
    Enum,
}

impl<'src> Resolver<'src> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            diags: Vec::new(),
        }
    }

    pub fn resolve(mut self, hir: &mut HirMap<'src>) -> ResolveResult {
        self.collect_items(hir);

        for (_, item) in hir.iter_mut() {
            match item {
                HirItem::Function(function) => {
                    for param in function.params.iter_mut() {
                        self.resolve_ty(&mut param.ty);
                    }
                    self.resolve_ty(&mut function.ret_ty);

                    let mut locals = LocalScope::new();
                    for param in function.params.iter() {
                        locals.define(param.name.node, param.span, &mut self.diags);
                    }
                    self.resolve_block(&mut function.body, &mut locals);
                }
                HirItem::Struct(strukt) => {
                    for field in strukt.fields.iter_mut() {
                        self.resolve_ty(&mut field.ty);
                    }
                }
                HirItem::Enum(enm) => {
                    for variant in enm.variants.iter_mut() {
                        match &mut variant.payload {
                            HirVariantPayload::Unit => {}
                            HirVariantPayload::Tuple(items) => {
                                for item in items.iter_mut() {
                                    self.resolve_ty(item);
                                }
                            }
                            HirVariantPayload::Struct(fields) => {
                                for field in fields.iter_mut() {
                                    self.resolve_ty(&mut field.ty);
                                }
                            }
                        }
                    }
                }
            }
        }

        ResolveResult { diags: self.diags }
    }

    fn collect_items(&mut self, hir: &HirMap<'src>) {
        for (id, item) in hir.iter() {
            let kind = match item {
                HirItem::Function(_) => ResolvedItemKind::Function,
                HirItem::Struct(_) => ResolvedItemKind::Struct,
                HirItem::Enum(_) => ResolvedItemKind::Enum,
            };
            let resolved = ResolvedItem { id, kind };
            if self.items.insert(item.name().node, resolved).is_some() {
                self.diags.push(diagnostic(
                    messages::DUPLICATE_ITEM,
                    &[("name", item.name().node)],
                    item.name().span,
                ));
            }
        }
    }

    fn resolve_block(&mut self, block: &mut HirBlock<'src>, locals: &mut LocalScope<'src>) {
        for stmt in block.stmts.iter_mut() {
            self.resolve_stmt(stmt, locals);
        }

        if let Some(tail) = &mut block.tail {
            self.resolve_expr(tail, locals);
        }
    }

    fn resolve_stmt(&mut self, stmt: &mut HirStmt<'src>, locals: &mut LocalScope<'src>) {
        match stmt {
            HirStmt::Expr(expr) => self.resolve_expr(expr, locals),
            HirStmt::Let {
                local,
                name,
                ty,
                init,
                ..
            } => {
                if let Some(ty) = ty {
                    self.resolve_ty(ty);
                }
                if let Some(init) = init {
                    self.resolve_expr(init, locals);
                }
                *local = Some(locals.define(name.node, name.span, &mut self.diags));
            }
        }
    }

    fn resolve_expr(&mut self, expr: &mut SpannedHirExpr<'src>, locals: &mut LocalScope<'src>) {
        match &mut expr.node {
            HirExpr::Path(path) => {
                if let Some(local) = locals.get_path(path) {
                    expr.node = HirExpr::Resolved(Res::Local(local));
                } else if let Some(builtin) = builtin_from_path(path) {
                    expr.node = HirExpr::Resolved(Res::Builtin(builtin));
                } else if !path.from_root && path.segments.len() == 1 {
                    if let Some(item) = self.items.get(path.segments[0].name.node).copied() {
                        if matches!(item.kind, ResolvedItemKind::Function) {
                            expr.node = HirExpr::Resolved(Res::Def(item.id));
                        } else {
                            let name = format_path(path);
                            self.diags.push(diagnostic(
                                messages::UNRESOLVED_NAME,
                                &[("name", &name)],
                                expr.span,
                            ));
                        }
                    } else {
                        let name = format_path(path);
                        self.diags.push(diagnostic(
                            messages::UNRESOLVED_NAME,
                            &[("name", &name)],
                            expr.span,
                        ));
                    }
                } else {
                    let name = format_path(path);
                    self.diags.push(diagnostic(
                        messages::UNRESOLVED_NAME,
                        &[("name", &name)],
                        expr.span,
                    ));
                }
            }
            HirExpr::Call { callee, args } => {
                self.resolve_expr(callee, locals);
                for arg in args.iter_mut() {
                    self.resolve_expr(arg, locals);
                }
            }
            HirExpr::Block(block) => self.resolve_block(block, locals),
            HirExpr::Error | HirExpr::Literal(_) | HirExpr::Resolved(_) => {}
        }
    }

    fn resolve_ty(&mut self, ty: &mut SpannedHirType<'src>) {
        match &mut ty.node {
            HirType::Error => {}
            HirType::Unresolved(path) => {
                if let Some(primitive) = primitive_from_path(path) {
                    ty.node = HirType::Primitive(primitive);
                } else if !path.from_root && path.segments.len() == 1 {
                    if let Some(item) = self.items.get(path.segments[0].name.node).copied() {
                        match item.kind {
                            ResolvedItemKind::Struct => {
                                ty.node = HirType::Struct {
                                    def: item.id,
                                    generics: Box::new([]),
                                };
                            }
                            ResolvedItemKind::Enum => {
                                ty.node = HirType::Enum {
                                    def: item.id,
                                    generics: Box::new([]),
                                };
                            }
                            ResolvedItemKind::Function => {
                                let name = format_path(path);
                                self.diags.push(diagnostic(
                                    messages::UNRESOLVED_TYPE,
                                    &[("name", &name)],
                                    ty.span,
                                ));
                            }
                        }
                    } else {
                        let name = format_path(path);
                        self.diags.push(diagnostic(
                            messages::UNRESOLVED_TYPE,
                            &[("name", &name)],
                            ty.span,
                        ));
                    }
                } else {
                    let name = format_path(path);
                    self.diags.push(diagnostic(
                        messages::UNRESOLVED_TYPE,
                        &[("name", &name)],
                        ty.span,
                    ));
                }
            }
            HirType::Tuple(items) => {
                for item in items.iter_mut() {
                    self.resolve_ty(item);
                }
            }
            HirType::Array { elem, len } => {
                self.resolve_ty(elem);
                let mut empty = LocalScope::new();
                self.resolve_expr(len, &mut empty);
            }
            HirType::Primitive(_)
            | HirType::Struct { .. }
            | HirType::Enum { .. }
            | HirType::Unit => {}
        }
    }
}

impl<'src> Default for Resolver<'src> {
    fn default() -> Self {
        Self::new()
    }
}

struct LocalScope<'src> {
    names: HashMap<&'src str, HirLocalId>,
    next: usize,
}

impl<'src> LocalScope<'src> {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
            next: 0,
        }
    }

    fn define(
        &mut self,
        name: &'src str,
        span: Span,
        diags: &mut Vec<Diagnostic<'static>>,
    ) -> HirLocalId {
        let id = HirLocalId::from_usize(self.next);
        self.next += 1;
        if self.names.insert(name, id).is_some() {
            diags.push(diagnostic(
                messages::DUPLICATE_LOCAL,
                &[("name", name)],
                span,
            ));
        }
        id
    }

    fn get_path(&self, path: &runec_hir::path::HirPath<'src>) -> Option<HirLocalId> {
        if path.from_root || path.segments.len() != 1 {
            return None;
        }

        self.names.get(path.segments[0].name.node).copied()
    }
}

fn primitive_from_path(path: &runec_hir::path::HirPath<'_>) -> Option<HirPrimitiveTy> {
    if path.from_root || path.segments.len() != 1 {
        return None;
    }

    match path.segments[0].name.node {
        "i8" => Some(HirPrimitiveTy::I8),
        "i16" => Some(HirPrimitiveTy::I16),
        "i32" | "int" => Some(HirPrimitiveTy::I32),
        "i64" => Some(HirPrimitiveTy::I64),
        "i128" => Some(HirPrimitiveTy::I128),
        "u8" => Some(HirPrimitiveTy::U8),
        "u16" => Some(HirPrimitiveTy::U16),
        "u32" => Some(HirPrimitiveTy::U32),
        "u64" => Some(HirPrimitiveTy::U64),
        "u128" => Some(HirPrimitiveTy::U128),
        "f32" => Some(HirPrimitiveTy::F32),
        "f64" | "float" => Some(HirPrimitiveTy::F64),
        "bool" => Some(HirPrimitiveTy::Bool),
        "char" => Some(HirPrimitiveTy::Char),
        "str" | "string" => Some(HirPrimitiveTy::Str),
        _ => None,
    }
}

fn builtin_from_path(path: &runec_hir::path::HirPath<'_>) -> Option<runec_builtins::BuiltinId> {
    if path.from_root || path.segments.len() != 1 || path.segments[0].generics.is_some() {
        return None;
    }

    builtin_from_name(path.segments[0].name.node)
}

fn format_path(path: &runec_hir::path::HirPath<'_>) -> String {
    let joined = path
        .segments
        .iter()
        .map(|segment| segment.name.node)
        .collect::<Vec<_>>()
        .join("::");
    if joined.is_empty() {
        "<empty>".to_owned()
    } else if path.from_root {
        format!("::{joined}")
    } else {
        joined
    }
}

fn diagnostic(
    message: &'static str,
    replacements: &[(&str, &str)],
    span: Span,
) -> Diagnostic<'static> {
    *Diagnostic::error(DiagMessage::new(message, replacements))
        .add_label(DiagLabel::silent_primary(span))
}

mod messages;

#[cfg(test)]
mod tests {
    use runec_ast::SpannedStr;
    use runec_source::byte_pos::BytePos;
    use runec_source::source_map::SourceId;
    use runec_source::span::{Span, Spanned};

    use runec_builtins::builtin_from_name;
    use runec_hir::expression::{HirExpr, SpannedHirExpr};
    use runec_hir::ids::{HirId, HirLocalId};
    use runec_hir::item::{HirFunction, HirFunctionParam, HirItem};
    use runec_hir::map::HirMap;
    use runec_hir::path::{HirPath, HirPathSegment};
    use runec_hir::resolution::Res;
    use runec_hir::statement::{HirBlock, HirStmt};
    use runec_hir::ty::{HirPrimitiveTy, HirType};

    use super::Resolver;

    const SRC: SourceId = SourceId::from_usize(0);

    fn sp(lo: usize, hi: usize) -> Span {
        Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), SRC)
    }

    fn s<T>(node: T) -> Spanned<T> {
        Spanned::new(node, sp(0, 0))
    }

    fn path_expr(name: &'static str) -> SpannedHirExpr<'static> {
        s(HirExpr::Path(HirPath {
            from_root: false,
            segments: Box::new([HirPathSegment {
                name: SpannedStr::new(name, sp(0, 0)),
                generics: None,
                span: sp(0, 0),
            }]),
            span: sp(0, 0),
        }))
    }

    #[test]
    fn resolves_params_lets_and_builtins() {
        let mut hir = HirMap::new();
        hir.push(HirItem::Function(HirFunction {
            id: HirId::from_usize(0),
            name: SpannedStr::new("main", sp(0, 0)),
            params: Box::new([HirFunctionParam {
                name: SpannedStr::new("x", sp(0, 0)),
                ty: s(HirType::Primitive(HirPrimitiveTy::I32)),
                span: sp(0, 0),
            }]),
            ret_ty: s(HirType::Unit),
            body: HirBlock {
                stmts: Box::new([
                    HirStmt::Let {
                        local: None,
                        name: SpannedStr::new("y", sp(0, 0)),
                        is_mutable: false,
                        ty: None,
                        init: Some(path_expr("x")),
                        span: sp(0, 0),
                    },
                    HirStmt::Expr(s(HirExpr::Call {
                        callee: Box::new(path_expr("println")),
                        args: Box::new([path_expr("y")]),
                    })),
                ]),
                tail: None,
                span: sp(0, 0),
            },
            span: sp(0, 0),
        }));

        let result = Resolver::new().resolve(&mut hir);
        assert!(result.diags.is_empty());

        let HirItem::Function(function) = hir.get(HirId::from_usize(0)) else {
            panic!("expected function");
        };
        let HirStmt::Let {
            local,
            init: Some(init),
            ..
        } = &function.body.stmts[0]
        else {
            panic!("expected let");
        };
        assert_eq!(*local, Some(HirLocalId::from_usize(1)));
        assert_eq!(
            init.node,
            HirExpr::Resolved(Res::Local(HirLocalId::from_usize(0)))
        );

        let HirStmt::Expr(call) = &function.body.stmts[1] else {
            panic!("expected call statement");
        };
        let HirExpr::Call { callee, args } = &call.node else {
            panic!("expected call");
        };
        assert_eq!(
            callee.node,
            HirExpr::Resolved(Res::Builtin(
                builtin_from_name("println").expect("println builtin")
            ))
        );
        assert_eq!(
            args[0].node,
            HirExpr::Resolved(Res::Local(HirLocalId::from_usize(1)))
        );
    }

    #[test]
    fn reports_unresolved_name_as_diagnostic() {
        let unresolved_span = sp(10, 17);
        let mut hir = HirMap::new();
        hir.push(HirItem::Function(HirFunction {
            id: HirId::from_usize(0),
            name: SpannedStr::new("main", sp(0, 4)),
            params: Box::new([]),
            ret_ty: s(HirType::Unit),
            body: HirBlock {
                stmts: Box::new([HirStmt::Expr(Spanned::new(
                    HirExpr::Path(HirPath {
                        from_root: false,
                        segments: Box::new([HirPathSegment {
                            name: SpannedStr::new("missing", unresolved_span),
                            generics: None,
                            span: unresolved_span,
                        }]),
                        span: unresolved_span,
                    }),
                    unresolved_span,
                ))]),
                tail: None,
                span: sp(8, 18),
            },
            span: sp(0, 18),
        }));

        let result = Resolver::new().resolve(&mut hir);

        assert_eq!(result.diags.len(), 1);
        assert_eq!(result.diags[0].labels[0].span, unresolved_span);
        assert_eq!(
            result.diags[0].message.message,
            "cannot resolve value `missing`"
        );
    }
}
