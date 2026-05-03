use runec_ast::SpannedStr;
use runec_ast::ast_type::{SpannedTypeAnnotation, TypeAnnotation};
use runec_ast::expression::{Expr, PrimitiveValue, SpannedExpr};
use runec_ast::statement::{DestructPattern, SpannedStmt, SpannedStmtBlock, Stmt};
use runec_errors::diagnostics::Diagnostic;
use runec_source::span::Spanned;

use crate::expression::{HirExpr, HirLiteral, SpannedHirExpr};
use crate::item::{HirFunction, HirFunctionParam, HirItem};
use crate::map::HirMap;
use crate::path::{HirPath, HirPathSegment};
use crate::statement::{HirBlock, HirStmt};
use crate::ty::{HirType, SpannedHirType};

pub struct HirLowerResult<'src, 'diag> {
    pub map: HirMap<'src>,
    pub diags: Vec<Diagnostic<'diag>>,
}

impl<'src, 'diag> HirLowerResult<'src, 'diag> {
    pub fn new() -> Self {
        Self { map: HirMap::new(), diags: Vec::new() }
    }
}

pub struct HirLowerer<'src, 'diag> {
    res: HirLowerResult<'src, 'diag>,
}

impl<'src, 'diag> HirLowerer<'src, 'diag> {
    pub fn new() -> Self {
        Self { res: HirLowerResult::new() }
    }

    pub fn lower(mut self, stmts: &[SpannedStmt<'src>]) -> HirLowerResult<'src, 'diag> {
        for stmt in stmts {
            self.lower_top_stmt(stmt);
        }
        self.res
    }

    // ---- top-level items ----

    fn lower_top_stmt(&mut self, stmt: &SpannedStmt<'src>) {
        match &stmt.node {
            Stmt::DefineFunction { ident, args, ret_ty, body } => {
                let id = self.res.map.reserve_id();
                let params: Box<[_]> = args.iter().map(|a| {
                    let ty = self.lower_type(&a.ty);
                    HirFunctionParam {
                        name: SpannedStr::new(a.ident.node, a.ident.span),
                        ty,
                        span: a.ident.span,
                    }
                }).collect();
                let ret_ty = self.lower_type(ret_ty);
                let body = self.lower_block(body);
                self.res.map.push(HirItem::Function(HirFunction {
                    id,
                    name: SpannedStr::new(ident.node, ident.span),
                    params,
                    ret_ty,
                    body,
                    span: stmt.span,
                }));
            }
            Stmt::DefineLet { .. }
            | Stmt::DefineConst { .. }
            | Stmt::Assign { .. }
            | Stmt::SemiExpr(_)
            | Stmt::TailExpr(_) => {}
        }
    }

    // ---- blocks & statements ----

    fn lower_block(&mut self, block: &SpannedStmtBlock<'src>) -> HirBlock<'src> {
        let mut stmts = Vec::with_capacity(block.node.len());
        let mut tail: Option<Box<SpannedHirExpr<'src>>> = None;

        for (i, s) in block.node.iter().enumerate() {
            let is_last = i + 1 == block.node.len();
            match &s.node {
                Stmt::TailExpr(e) if is_last => {
                    tail = Some(Box::new(self.lower_expr(e)));
                }
                Stmt::SemiExpr(e) | Stmt::TailExpr(e) => {
                    stmts.push(HirStmt::Expr(self.lower_expr(e)));
                }
                Stmt::DefineLet { pattern, is_mutable, ty, init_expr } => {
                    let name = match &pattern.node {
                        DestructPattern::Ident(n) => SpannedStr::new(n, pattern.span),
                        DestructPattern::Tuple(_) | DestructPattern::AttributeAccess { .. } => {
                            todo!("let-destructuring lowering");
                        }
                    };
                    let ty = ty.as_ref().map(|t| self.lower_type(t));
                    let init = init_expr.as_ref().map(|e| self.lower_expr(e));
                    stmts.push(HirStmt::Let {
                        name,
                        is_mutable: *is_mutable,
                        ty,
                        init,
                        span: s.span,
                    });
                }
                Stmt::DefineFunction { .. }
                | Stmt::DefineConst { .. }
                | Stmt::Assign { .. } => {
                    todo!("lowering of nested function / const / assign");
                }
            }
        }

        HirBlock {
            stmts: stmts.into_boxed_slice(),
            tail,
            span: block.span,
        }
    }

    // ---- expressions ----

    fn lower_expr(&mut self, expr: &SpannedExpr<'src>) -> SpannedHirExpr<'src> {
        let hir = match &expr.node {
            Expr::Primitive(p) => HirExpr::Literal(Self::lower_literal(p)),

            Expr::Ident(name) => HirExpr::Path(HirPath {
                from_root: false,
                segments: Box::new([HirPathSegment {
                    name: SpannedStr::new(name, expr.span),
                    generics: None,
                    span: expr.span,
                }]),
                span: expr.span,
            }),

            Expr::Path(segments) => {
                let segments: Box<[_]> = segments.iter().map(|s| HirPathSegment {
                    name: SpannedStr::new(s.node, s.span),
                    generics: None,
                    span: s.span,
                }).collect();
                HirExpr::Path(HirPath {
                    from_root: false,
                    segments,
                    span: expr.span,
                })
            }

            Expr::Call { callee, args } => HirExpr::Call {
                callee: Box::new(self.lower_expr(callee)),
                args: args.iter().map(|a| self.lower_expr(a)).collect(),
            },

            Expr::Block(b) => HirExpr::Block(self.lower_block(b)),

            Expr::If(_)
            | Expr::TypeCast { .. }
            | Expr::Binary { .. }
            | Expr::Unary { .. }
            | Expr::Tuple(_)
            | Expr::FullyDefinedArray(_)
            | Expr::RepeatingArray { .. }
            | Expr::Deref(_)
            | Expr::AttributeAccess { .. } => {
                todo!("expression lowering for this variant");
            }
        };
        Spanned::new(hir, expr.span)
    }

    fn lower_literal(p: &PrimitiveValue<'src>) -> HirLiteral<'src> {
        match p {
            PrimitiveValue::Int { value, suffix } =>
                HirLiteral::Int { value: *value, suffix: *suffix },
            PrimitiveValue::Float { value, suffix } =>
                HirLiteral::Float { value: *value, suffix: *suffix },
            PrimitiveValue::True  => HirLiteral::Bool(true),
            PrimitiveValue::False => HirLiteral::Bool(false),
            PrimitiveValue::Char(c) => HirLiteral::Char(*c),
            PrimitiveValue::String(s) => HirLiteral::Str(s.clone()),
        }
    }

    // ---- types ----

    fn lower_type(&mut self, ty: &SpannedTypeAnnotation<'src>) -> SpannedHirType<'src> {
        let hir = match &ty.node {
            TypeAnnotation::Unit => HirType::Unit,

            TypeAnnotation::Ident(name) => HirType::Unresolved(HirPath {
                from_root: false,
                segments: Box::new([HirPathSegment {
                    name: SpannedStr::new(name, ty.span),
                    generics: None,
                    span: ty.span,
                }]),
                span: ty.span,
            }),

            TypeAnnotation::Tuple(items) =>
                HirType::Tuple(items.iter().map(|t| self.lower_type(t)).collect()),

            TypeAnnotation::Array { item, length } => HirType::Array {
                elem: Box::new(self.lower_type(item)),
                len: Box::new(self.lower_expr(length)),
            },

            TypeAnnotation::Path { .. } | TypeAnnotation::Generic { .. } => {
                todo!("lower_type: Path / Generic");
            }
        };
        Spanned::new(hir, ty.span)
    }
}

impl<'src, 'diag> Default for HirLowerer<'src, 'diag> {
    fn default() -> Self { Self::new() }
}

impl<'src, 'diag> Default for HirLowerResult<'src, 'diag> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests;
