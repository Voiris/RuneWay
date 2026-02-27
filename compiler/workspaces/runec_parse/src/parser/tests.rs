use runec_ast::ast_type::*;
use runec_ast::statement::*;
use runec_ast::expression::*;
use runec_ast::operators::*;
use runec_ast::*;
use runec_source::source_map::{SourceMap, SourceId};
use runec_source::span::Span;
use runec_source::byte_pos::BytePos;
use crate::generate_source;
use crate::lexer::lexer_struct::Lexer;
use crate::lexer::token::SpannedToken;
use super::parser_struct::Parser;

fn lex_source(source_map: &SourceMap, source_id: SourceId) -> Vec<SpannedToken<'_>> {
    let lexer = Lexer::new(source_id, source_map);
    lexer.lex_full().unwrap()
}

#[test]
fn act_parse_test() {
    let (source_map, source_id) = generate_source("act main(a: b, c: d) -> e {}");
    let tokens = lex_source(&source_map, source_id);
    let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

    assert_eq!(parse_result.diags.len(), 0);

    let expected_stmts = [
        SpannedStmt::new(Stmt::DefineFunction {
            ident: SpannedStr::new("main", Span::new(BytePos::from_usize(4), BytePos::from_usize(8), source_id)),
            args: Box::new([
                FunctionArg {
                    ident: SpannedStr::new("a", Span::new(BytePos::from_usize(9), BytePos::from_usize(10), source_id)),
                    ty: SpannedTypeAnnotation::new(TypeAnnotation::Ident("b"), Span::new(BytePos::from_usize(12), BytePos::from_usize(13), source_id)),
                },
                FunctionArg {
                    ident: SpannedStr::new("c", Span::new(BytePos::from_usize(15), BytePos::from_usize(16), source_id)),
                    ty: SpannedTypeAnnotation::new(TypeAnnotation::Ident("d"), Span::new(BytePos::from_usize(18), BytePos::from_usize(19), source_id)),
                }
            ]),
            ret_ty: SpannedTypeAnnotation::new(TypeAnnotation::Ident("e"), Span::new(BytePos::from_usize(24), BytePos::from_usize(25), source_id)),
            body: SpannedStmtBlock::new(
                Box::new([]),
                Span::new(BytePos::from_usize(26), BytePos::from_usize(28), source_id)
            ),
        }, Span::new(BytePos::from_usize(0), BytePos::from_usize(28), source_id))
    ];

    assert_eq!(parse_result.stmts, expected_stmts);
}

#[test]
fn expr_parsing_test() {
    let (source_map, source_id) = generate_source("a * b + c / (d - e)");
    let tokens = lex_source(&source_map, source_id);
    let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

    let expected_stmts = [
        SpannedStmt::new(Stmt::TailExpr(
            SpannedExpr::new(Expr::Binary {
                lhs: Box::new(SpannedExpr::new(Expr::Binary {
                    lhs: Box::new(SpannedExpr::new(Expr::Ident("a"), Span::new(BytePos::from_usize(0), BytePos::from_usize(1), source_id))),
                    rhs: Box::new(SpannedExpr::new(Expr::Ident("b"), Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id))),
                    op: BinaryOp::Mul
                }, Span::new(BytePos::from_usize(0), BytePos::from_usize(5), source_id))),
                rhs: Box::new(SpannedExpr::new(Expr::Binary {
                    lhs: Box::new(SpannedExpr::new(Expr::Ident("c"), Span::new(BytePos::from_usize(8), BytePos::from_usize(9), source_id))),
                    rhs: Box::new(SpannedExpr::new(Expr::Binary {
                        lhs: Box::new(SpannedExpr::new(Expr::Ident("d"), Span::new(BytePos::from_usize(13), BytePos::from_usize(14), source_id))),
                        rhs: Box::new(SpannedExpr::new(Expr::Ident("e"), Span::new(BytePos::from_usize(17), BytePos::from_usize(18), source_id))),
                        op: BinaryOp::Sub
                    }, Span::new(BytePos::from_usize(13), BytePos::from_usize(18), source_id))),
                    op: BinaryOp::Div
                }, Span::new(BytePos::from_usize(8), BytePos::from_usize(18), source_id))),
                op: BinaryOp::Add,
            }, Span::new(BytePos::from_usize(0), BytePos::from_usize(18), source_id))
        ), Span::new(BytePos::from_usize(0), BytePos::from_usize(18), source_id))
    ];

    assert_eq!(parse_result.diags.len(), 0);
    assert_eq!(parse_result.stmts, expected_stmts);
}

#[test]
fn stmt_block_expr_test() {
    let (source_map, source_id) = generate_source("{ a; b; c; d }");
    let tokens = lex_source(&source_map, source_id);
    let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

    let expected_stmts = [
        SpannedStmt::new(Stmt::TailExpr(
            SpannedExpr::new(Expr::Block(
                SpannedStmtBlock::new(Box::new([
                    SpannedStmt::new(
                        Stmt::SemiExpr(SpannedExpr::new(
                            Expr::Ident("a"),
                            Span::new(BytePos::from_usize(2), BytePos::from_usize(3), source_id))
                        ),
                        Span::new(BytePos::from_usize(2), BytePos::from_usize(4), source_id)
                    ),
                    SpannedStmt::new(
                        Stmt::SemiExpr(SpannedExpr::new(
                            Expr::Ident("b"),
                            Span::new(BytePos::from_usize(5), BytePos::from_usize(6), source_id))
                        ),
                        Span::new(BytePos::from_usize(5), BytePos::from_usize(7), source_id)
                    ),
                    SpannedStmt::new(
                        Stmt::SemiExpr(SpannedExpr::new(
                            Expr::Ident("c"),
                            Span::new(BytePos::from_usize(8), BytePos::from_usize(9), source_id))
                        ),
                        Span::new(BytePos::from_usize(8), BytePos::from_usize(10), source_id)
                    ),
                    SpannedStmt::new(
                        Stmt::TailExpr(SpannedExpr::new(
                            Expr::Ident("d"),
                            Span::new(BytePos::from_usize(11), BytePos::from_usize(12), source_id))
                        ),
                        Span::new(BytePos::from_usize(11), BytePos::from_usize(12), source_id)
                    )
                ]) as StmtBlock, Span::new(BytePos::from_usize(0), BytePos::from_usize(14), source_id))
            ), Span::new(BytePos::from_usize(0), BytePos::from_usize(14), source_id))
        ), Span::new(BytePos::from_usize(0), BytePos::from_usize(14), source_id))
    ];

    assert_eq!(parse_result.diags.len(), 0);
    assert_eq!(parse_result.stmts, expected_stmts);
}

#[test]
fn function_call_parse_test() {
    let (source_map, source_id) = generate_source("print(a, b)");
    let tokens = lex_source(&source_map, source_id);
    let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

    let expected_stmts = [
        SpannedStmt::new(Stmt::TailExpr(
            SpannedExpr::new(Expr::Call {
                callee: Box::new(SpannedExpr::new(
                    Expr::Ident("print"),
                    Span::new(BytePos::from_usize(0), BytePos::from_usize(5), source_id)
                )),
                args: Box::new([
                    SpannedExpr::new(
                        Expr::Ident("a"),
                        Span::new(BytePos::from_usize(6), BytePos::from_usize(7), source_id)
                    ),
                    SpannedExpr::new(
                        Expr::Ident("b"),
                        Span::new(BytePos::from_usize(9), BytePos::from_usize(10), source_id)
                    )
                ]),
            }, Span::new(BytePos::from_usize(0), BytePos::from_usize(11), source_id))
        ), Span::new(BytePos::from_usize(0), BytePos::from_usize(11), source_id))
    ];

    assert_eq!(parse_result.diags.len(), 0);
    assert_eq!(parse_result.stmts, expected_stmts);
}

#[test]
fn let_parse_test() {
    let (source_map, source_id) = generate_source("let a = a;let b: u8 = b;let c;let e: i8;let mut f;let mut g: f32;");
    let tokens = lex_source(&source_map, source_id);
    let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

    let expected_stmts = [
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("a"),
                Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id),
            ),
            is_mutable: false,
            ty: None,
            init_expr: Some(SpannedExpr::new(
                Expr::Ident("a"),
                Span::new(BytePos::from_usize(8), BytePos::from_usize(9), source_id),
            )),
        }, Span::new(BytePos::from_usize(0), BytePos::from_usize(10), source_id)),
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("b"),
                Span::new(BytePos::from_usize(14), BytePos::from_usize(15), source_id),
            ),
            is_mutable: false,
            ty: Some(SpannedTypeAnnotation::new(
                TypeAnnotation::Ident("u8"),
                Span::new(BytePos::from_usize(17), BytePos::from_usize(19), source_id),
            )),
            init_expr: Some(SpannedExpr::new(
                Expr::Ident("b"),
                Span::new(BytePos::from_usize(22), BytePos::from_usize(23), source_id),
            )),
        }, Span::new(BytePos::from_usize(10), BytePos::from_usize(24), source_id)),
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("c"),
                Span::new(BytePos::from_usize(28), BytePos::from_usize(29), source_id),
            ),
            is_mutable: false,
            ty: None,
            init_expr: None,
        }, Span::new(BytePos::from_usize(24), BytePos::from_usize(30), source_id)),
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("e"),
                Span::new(BytePos::from_usize(34), BytePos::from_usize(35), source_id),
            ),
            is_mutable: false,
            ty: Some(SpannedTypeAnnotation::new(
                TypeAnnotation::Ident("i8"),
                Span::new(BytePos::from_usize(37), BytePos::from_usize(39), source_id),
            )),
            init_expr: None,
        }, Span::new(BytePos::from_usize(30), BytePos::from_usize(40), source_id)),
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("f"),
                Span::new(BytePos::from_usize(48), BytePos::from_usize(49), source_id),
            ),
            is_mutable: true,
            ty: None,
            init_expr: None,
        }, Span::new(BytePos::from_usize(40), BytePos::from_usize(50), source_id)),
        SpannedStmt::new(Stmt::DefineLet {
            pattern: SpannedDestructPattern::new(
                DestructPattern::Ident("g"),
                Span::new(BytePos::from_usize(58), BytePos::from_usize(59), source_id),
            ),
            is_mutable: true,
            ty: Some(SpannedTypeAnnotation::new(
                TypeAnnotation::Ident("f32"),
                Span::new(BytePos::from_usize(61), BytePos::from_usize(64), source_id),
            )),
            init_expr: None,
        }, Span::new(BytePos::from_usize(50), BytePos::from_usize(65), source_id)),
    ];

    assert_eq!(parse_result.diags.len(), 0);
    assert_eq!(parse_result.stmts, expected_stmts);
}

#[test]
fn ident_destruct_pattern_parse_test() {
    let (source_map, source_id) = generate_source("ident (a, b) ((a, b), c)");
    let tokens = lex_source(&source_map, source_id);
    let mut parser = Parser::new(tokens, source_id, &source_map);

    assert_eq!(
        parser.parse_destruct_pattern().unwrap(),
        SpannedDestructPattern::new(
            DestructPattern::Ident("ident"),
            Span::new(BytePos::from_usize(0), BytePos::from_usize(5), source_id),
        )
    );
}

#[test]
fn tuple_destruct_pattern_parse_test() {
    let (source_map, source_id) = generate_source("(a, b)");
    let tokens = lex_source(&source_map, source_id);
    let mut parser = Parser::new(tokens, source_id, &source_map);

    assert_eq!(
        parser.parse_destruct_pattern().unwrap(),
        SpannedDestructPattern::new(
            DestructPattern::Tuple(
                Box::new([
                    SpannedDestructPattern::new(
                        DestructPattern::Ident("a"),
                        Span::new(BytePos::from_usize(1), BytePos::from_usize(2), source_id),
                    ),
                    SpannedDestructPattern::new(
                        DestructPattern::Ident("b"),
                        Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id),
                    )
                ])
            ),
            Span::new(BytePos::from_usize(0), BytePos::from_usize(6), source_id),
        )
    );
}

#[test]
fn multilevel_tuple_destruct_pattern_parse_test() {
    let (source_map, source_id) = generate_source("((a, b), c)");
    let tokens = lex_source(&source_map, source_id);
    let mut parser = Parser::new(tokens, source_id, &source_map);

    assert_eq!(
        parser.parse_destruct_pattern().unwrap(),
        SpannedDestructPattern::new(
            DestructPattern::Tuple(
                Box::new([
                    SpannedDestructPattern::new(
                        DestructPattern::Tuple(Box::new([
                            SpannedDestructPattern::new(
                                DestructPattern::Ident("a"),
                                Span::new(BytePos::from_usize(2), BytePos::from_usize(3), source_id),
                            ),
                            SpannedDestructPattern::new(
                                DestructPattern::Ident("b"),
                                Span::new(BytePos::from_usize(5), BytePos::from_usize(6), source_id),
                            ),
                        ])),
                        Span::new(BytePos::from_usize(1), BytePos::from_usize(7), source_id),
                    ),
                    SpannedDestructPattern::new(
                        DestructPattern::Ident("c"),
                        Span::new(BytePos::from_usize(9), BytePos::from_usize(10), source_id),
                    )
                ])
            ),
            Span::new(BytePos::from_usize(0), BytePos::from_usize(11), source_id),
        )
    );
}

#[test]
fn unit_type_annotation_parse_test() {
    let (source_map, source_id) = generate_source("()");
    let tokens = lex_source(&source_map, source_id);
    let mut parser = Parser::new(tokens, source_id, &source_map);

    assert_eq!(
        parser.parse_type_annotation().unwrap(),
        SpannedTypeAnnotation::new(
            TypeAnnotation::Unit,
            Span::new(BytePos::from_usize(0), BytePos::from_usize(2), source_id),
        )
    );
}

#[test]
fn tuple_type_annotation_parse_test() {
    let (source_map, source_id) = generate_source("(a, b)");
    let tokens = lex_source(&source_map, source_id);
    let mut parser = Parser::new(tokens, source_id, &source_map);

    assert_eq!(
        parser.parse_type_annotation().unwrap(),
        SpannedTypeAnnotation::new(
            TypeAnnotation::Tuple(
                Box::new([
                    SpannedTypeAnnotation::new(
                        TypeAnnotation::Ident("a"),
                        Span::new(BytePos::from_usize(1), BytePos::from_usize(2), source_id),
                    ),
                    SpannedTypeAnnotation::new(
                        TypeAnnotation::Ident("b"),
                        Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id),
                    )
                ])
            ),
            Span::new(BytePos::from_usize(0), BytePos::from_usize(6), source_id),
        )
    );
}
