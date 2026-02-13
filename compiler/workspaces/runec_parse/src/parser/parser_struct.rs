use std::borrow::Cow;
use std::iter::Peekable;
use std::num::IntErrorKind;
use std::vec::IntoIter;
use fluent::FluentValue;
use runec_ast::ast_type::{SpannedTypeAnnotation, TypeAnnotation};
use runec_ast::expression::{Expr, PrimitiveValue, SpannedExpr, IntSuffix};
use runec_ast::operators::{BinaryOp, UnaryOp};
use runec_ast::SpannedStr;
use runec_ast::statement::{FunctionArg, SpannedStmt, SpannedStmtBlock, Stmt};
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::{DiagLabel, DiagNote};
use runec_errors::make_simple_diag;
use runec_errors::message::DiagMessage;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::{SourceFile, SourceId, SourceMap};
use runec_source::span::Span;
use crate::lexer::token::{Radix, SpannedToken, Token};
use crate::parser::result::ParseResult;
use crate::parser::pratt;

macro_rules! expect_token {
    ($self:expr, $expected:pat, $expected_str:expr) => {{
        if let Some(token) = $self.tokens.next() {
            match token.node {
                $expected => Ok(token),
                token => Err(unexpected_token!(token, $expected_str)),
            }
        } else {
            Err($self.unexpected_eof())
        }
    }};

    ($self:expr, $expected:pat, [$($expected_str:expr),*], *) => {{
        if let Some(token) = $self.tokens.next() {
            match token.node {
                $expected => Ok(token),
                token => Err(unexpected_token!(token, [$($expected_str),*], *))
            }
        } else {
            Err($self.unexpected_eof())
        }
    }};
}

macro_rules! unexpected_token {
    ($token:expr, $expected_str:expr) => {{
        InnerParseErr::with_skip(Diagnostic::error(
            DiagMessage::new_with_args(
                "unexpected-token",
                runec_utils::hashmap!(
                    "expected" => FluentValue::String(Cow::Borrowed($expected_str)),
                    "got" => FluentValue::String(Cow::Borrowed($token.display())),
                )
            )
        ))
    }};

    ($token:expr, [$($expected_str:expr),*], *) => {{
        InnerParseErr::with_skip(Diagnostic::error(
            DiagMessage::new_with_args(
                "unexpected-token",
                runec_utils::hashmap!(
                    "expected" => FluentValue::String(Cow::Owned([$($expected_str),*].join("` or `"))),
                    "got" => FluentValue::String(Cow::Borrowed($token.display())),
                )
            )
        ))
    }}
}

type InnerParserResult<'diag, T> = Result<T, InnerParseErr<'diag>>;

struct InnerParseErr<'diag> {
    diag: Box<Diagnostic<'diag>>,
    should_skip_until_new_stmt: bool
}

impl<'diag> InnerParseErr<'diag> {
    fn with_skip(diag: Box<Diagnostic<'diag>>) -> Self {
        Self { diag, should_skip_until_new_stmt: true }
    }

    fn without_skip(diag: Box<Diagnostic<'diag>>) -> Self {
        Self { diag, should_skip_until_new_stmt: false }
    }
}

pub struct Parser<'src, 'diag> {
    tokens: Peekable<IntoIter<SpannedToken<'src>>>,
    source_id: SourceId,
    source_file: &'src SourceFile,
    source_hi: BytePos,
    res: ParseResult<'src, 'diag>
}

impl<'src, 'diag> Parser<'src, 'diag> {
    pub fn new(tokens: Vec<SpannedToken<'src>>, source_id: SourceId, source_map: &'src SourceMap) -> Self {
        let source_file = source_map.get_file(&source_id).unwrap();
        Self {
            tokens: tokens.into_iter().peekable(),
            source_id,
            source_hi:BytePos::from_usize(source_file.src.len()),
            source_file,
            res: ParseResult::new()
        }
    }

    fn unexpected_token(got: &'static str) -> Box<Diagnostic<'diag>> {
        Diagnostic::error(
            DiagMessage::new_with_args("unexpected-token", runec_utils::hashmap!(
                "token" => FluentValue::String(Cow::Owned(got.to_string())),
            ))
        )
    }

    fn unexpected_eof(&self) -> InnerParseErr<'diag> {
        InnerParseErr::without_skip(Diagnostic::error(
            DiagMessage::new_with_args(
                "unexpected-eof",
                runec_utils::hashmap!(
                        "path" => FluentValue::String(
                            Cow::Owned(self.source_file.file_name.to_string())
                        )
                    )
            )
        ))
    }

    fn peek(&mut self) -> InnerParserResult<'diag, &SpannedToken<'src>> {
        // Keep this as a boolean to avoid overlapping borrows.
        if self.tokens.peek().is_none() {
            Err(self.unexpected_eof())
        } else {
            Ok(self.tokens.peek().unwrap())
        }
    }

    fn bump(&mut self) -> InnerParserResult<'diag, SpannedToken<'src>> {
        self.tokens.next().ok_or_else(|| self.unexpected_eof())
    }

    fn parse_statement(&mut self) -> InnerParserResult<'diag, SpannedStmt<'src>> {
        let token = self.peek()?;
        match token.node {
            Token::Act => self.parse_act(),
            Token::Ident( .. ) | Token::IntLiteral { .. } | Token::FloatLiteral { .. } |
            Token::RawStringLiteral( .. ) | Token::StringLiteral( .. ) | Token::CharLiteral( .. ) |
            Token::Tilde | Token::Bang | Token::Minus |
            Token::Plus | Token::PlusPlus | Token::MinusMinus |
            Token::OpenParen | Token::OpenBrace | Token::OpenBracket |
            Token::True | Token::False => {
                let expr = self.parse_expr(0)?;
                let stmt = match self.tokens.peek() {
                    Some(t) if t.node == Token::Semicolon => {
                        let lo = expr.span.lo;
                        let hi = self.bump()?.span.hi;
                        SpannedStmt::new(Stmt::SemiExpr(expr), Span::new(lo, hi, self.source_id))
                    }
                    _ => {
                        let span = expr.span;
                        SpannedStmt::new(Stmt::TailExpr(expr), span)
                    }
                };
                Ok(stmt)
            }
            _ => {
                Err(InnerParseErr::with_skip(Self::unexpected_token(token.node.display())))
            }
        }
    }

    fn parse_act(&mut self) -> InnerParserResult<'diag, SpannedStmt<'src>> {
        let lo = expect_token!(self, Token::Act, Token::Act.display())?.span.lo;

        let ident = if let Some(token) = self.tokens.next() {
            match token.node {
                Token::Ident(ident) => {
                    (ident, token.span)
                }
                _ => return Err(unexpected_token!(token, "identifier")),
            }
        } else {
            return Err(self.unexpected_eof());
        };

        expect_token!(self, Token::OpenParen, Token::OpenParen.display())?;
        let mut args = Vec::new();
        let mut args_terminating_hi = None;
        let mut args_lo_opt = None;

        while let Some(token) = self.tokens.next() {
            args_lo_opt.get_or_insert(token.span.lo);
            match token.node {
                Token::Ident(ident) => {
                    self.tokens.next();
                    let ty = self.parse_type_annotation()?;
                    args.push(FunctionArg { ident: SpannedStr::new(ident, token.span), ty });
                }
                _ => return Err(unexpected_token!(token, "identifier")),
            }
            let token = expect_token!(self, Token::CloseParen | Token::Comma, [Token::CloseParen.display(), Token::Comma.display()], *)?;
            if token.node == Token::CloseParen {
                args_terminating_hi = Some(token.span.hi);
                break;
            } else if self.tokens.peek().is_some_and(|t| t.node == Token::CloseParen) {
                args_terminating_hi = Some(self.tokens.next().unwrap().span.hi);
                break;
            }
        }

        match args_terminating_hi {
            // unterminated arguments block
            None => {
                if let Some(args_lo) = args_lo_opt {
                    Err(
                        InnerParseErr::without_skip(
                            make_simple_diag!(
                                error;
                                "unterminated-args-block",
                                (self.source_id => args_lo..self.source_hi)
                            )
                        )
                    )
                } else {
                    Err(self.unexpected_eof())
                }
            }
            // terminated arguments block
            Some(args_hi) => {
                let ret_ty = if self.tokens.peek().is_some_and(|t| t.node == Token::Arrow) {
                    self.tokens.next();

                    self.parse_type_annotation()?
                } else { SpannedTypeAnnotation::new(TypeAnnotation::Unit, Span::new(args_hi, args_hi, self.source_id)) };

                let stmt_block = self.parse_stmt_block()?;
                let hi = stmt_block.span.hi;

                Ok(SpannedStmt::new(Stmt::DefineFunction {
                    ident: SpannedStr::new(ident.0, ident.1),
                    args: args.into_boxed_slice(),
                    ret_ty,
                    body: stmt_block
                }, Span::new(lo, hi, self.source_id)))
            }
        }
    }

    fn parse_type_annotation(&mut self) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        if let Some(token) = self.tokens.next() {
            let lo = token.span.lo;
            match &token.node {
                Token::Ident(ident) => Ok(SpannedTypeAnnotation::new(TypeAnnotation::Ident(ident), Span::new(lo, token.span.hi, self.source_id))),
                _ => todo!()
            }
        } else {
            Err(self.unexpected_eof())
        }
    }

    fn parse_stmt_block(&mut self) -> InnerParserResult<'diag, SpannedStmtBlock<'src>> {
        let lo = expect_token!(self, Token::OpenBrace, Token::OpenBrace.display())?.span.lo;

        let mut stmts = Vec::new();
        let mut terminated = false;
        let mut hi_opt = None;

        while let Some(token) = self.tokens.peek() {
            match token.node {
                Token::CloseBrace => {
                    hi_opt = Some(self.tokens.next().unwrap().span.hi);
                    terminated = true;
                    break;
                }
                _ => stmts.push(self.parse_statement()?)
            }
        }

        if !terminated {
            return Err(
                InnerParseErr::without_skip(
                    make_simple_diag!(
                        error;
                        "unterminated-code-block",
                        (self.source_id => lo..self.source_hi)
                    )
                )
            )
        }

        Ok(SpannedStmtBlock::new(stmts.into_boxed_slice(), Span::new(lo, hi_opt.unwrap(), self.source_id)))
    }

    fn parse_expr(&mut self, min_bp: u8) -> InnerParserResult<'diag, SpannedExpr<'src>> {
        let mut lhs = {
            let token = self.peek()?;
            match token.node {
                Token::Ident(ident) => {
                    let token = self.bump()?;
                    SpannedExpr::new(Expr::Ident(ident), token.span)
                },
                Token::True | Token::False | Token::CharLiteral( .. ) |
                Token::IntLiteral { .. } | Token::FloatLiteral { .. } |
                Token::StringLiteral ( .. ) | Token::RawStringLiteral( .. ) => {
                    let token = self.bump()?;
                    let span = token.span;
                    // SAFETY: all Option variants are handled by match
                    let primitive_value = unsafe { Self::parse_primitive(token)?.unwrap_unchecked() };
                    SpannedExpr::new(Expr::Primitive(primitive_value), span)
                }
                Token::Bang | Token::Tilde | Token::Plus | Token::Minus | Token::PlusPlus | Token::MinusMinus => {
                    let token = self.bump()?;
                    let op = match token.node {
                        Token::Minus => UnaryOp::Neg,
                        Token::Plus => UnaryOp::Pos,
                        Token::Bang => UnaryOp::Not,
                        Token::Tilde => UnaryOp::BitNot,
                        Token::PlusPlus => UnaryOp::PrefInc,
                        Token::MinusMinus => UnaryOp::PrefDec,
                        _ => unreachable!()
                    };
                    let operand = self.parse_expr(pratt::rbp(&token.node))?;
                    let hi = operand.span.hi;

                    SpannedExpr::new(Expr::Unary { op, operand: Box::new(operand) }, Span::new(token.span.lo, hi, self.source_id))
                }
                Token::OpenParen => {
                    self.bump()?;
                    let expr = self.parse_expr(0)?;
                    expect_token!(self, Token::CloseParen, Token::CloseParen.display())?;
                    expr
                }
                Token::OpenBrace => {
                    let stmt_block = self.parse_stmt_block()?;
                    let span = stmt_block.span;
                    SpannedExpr::new(Expr::Block(stmt_block), span)
                }
                _ => todo!()
            }
        };

        while let Some(op_token) = self.tokens.peek() {
            let op_lbp = pratt::lbp(&op_token.node);
            if op_lbp < min_bp {
                break;
            }

            match op_token.node {
                Token::Plus | Token::Minus | Token::Star |
                Token::Slash | Token::Shl | Token::Shr |
                Token::EqEq | Token::Ne | Token::Lt |
                Token::Le | Token::Gt | Token::Ge |
                Token::AndAnd | Token::OrOr | Token::And |
                Token::Or | Token::Caret => {
                    let op = match op_token.node {
                        Token::Plus => BinaryOp::Add,
                        Token::Minus => BinaryOp::Sub,
                        Token::Star => BinaryOp::Mul,
                        Token::Slash => BinaryOp::Div,
                        Token::Shl => BinaryOp::Shl,
                        Token::Shr => BinaryOp::Shr,
                        Token::EqEq => BinaryOp::Eq,
                        Token::Ne => BinaryOp::Ne,
                        Token::Lt => BinaryOp::Lt,
                        Token::Le => BinaryOp::Le,
                        Token::Gt => BinaryOp::Gt,
                        Token::Ge => BinaryOp::Ge,
                        Token::AndAnd => BinaryOp::And,
                        Token::OrOr => BinaryOp::Or,
                        Token::And => BinaryOp::BitAnd,
                        Token::Or => BinaryOp::BitOr,
                        Token::Caret => BinaryOp::BitXor,
                        _ => unreachable!()
                    };
                    self.tokens.next();
                    let rhs = self.parse_expr(op_lbp + 1)?;
                    let lo = lhs.span.lo;
                    let hi = rhs.span.hi;
                    lhs = SpannedExpr::new(Expr::Binary {
                        lhs: Box::new(lhs),
                        op,
                        rhs: Box::new(rhs),
                    }, Span::new(lo, hi, self.source_id));
                }
                Token::PlusPlus | Token::MinusMinus => {
                    let op = match op_token.node {
                        Token::PlusPlus => UnaryOp::PostInc,
                        Token::MinusMinus => UnaryOp::PostDec,
                        _ => unreachable!()
                    };
                    let lo = lhs.span.lo;
                    let hi = op_token.span.hi;
                    self.tokens.next();
                    lhs = SpannedExpr::new(Expr::Unary { operand: Box::new(lhs), op }, Span::new(lo, hi, self.source_id));
                }
                Token::OpenParen => {
                    unimplemented!()
                }
                Token::OpenBracket => {
                    unimplemented!()
                }
                Token::OpenBrace => break,
                _ => break,
            }
        }

        Ok(lhs)
    }

    fn parse_int(digits: &'src str, radix: Radix, suffix_opt: Option<&'src str>, span: Span) -> InnerParserResult<'diag, PrimitiveValue<'src>> {
        match u128::from_str_radix(digits, radix as u32) {
            Ok(value) => {
                let suffix = match suffix_opt {
                    Some(suffix) => Some(match suffix {
                        "u8"   => IntSuffix::U8,
                        "u16"  => IntSuffix::U16,
                        "u32"  => IntSuffix::U32,
                        "u64"  => IntSuffix::U64,
                        "u128" => IntSuffix::U128,
                        "i8"   => IntSuffix::I8,
                        "i16"  => IntSuffix::I16,
                        "i32"  => IntSuffix::I32,
                        "i64"  => IntSuffix::I64,
                        "i128" => IntSuffix::I128,
                        "f32"  => IntSuffix::F32,
                        "f64"  => IntSuffix::F64,
                        _ => return Err(
                            InnerParseErr::with_skip(
                                make_simple_diag!(
                                error; "unsupported-suffix",
                                ( span.src_id => span.lo..span.hi ),
                                { note = "supported-suffixes-int" }
                            )
                            )
                        )
                    }),
                    None => None
                };
                Ok(PrimitiveValue::Int {
                    value,
                    suffix,
                })
            },

            Err(err) => {
                match err.kind() {
                    IntErrorKind::PosOverflow => {
                        Err(InnerParseErr::with_skip(
                            Diagnostic::error(
                                DiagMessage::new_simple("integer-literal-is-too-large"),
                            ).add_label(
                                DiagLabel::silent_primary(span)
                            ).set_note(
                                DiagNote::new_simple("integer-literal-value-exceeds-limit")
                            )
                        ))
                    },
                    IntErrorKind::Empty => unreachable!(),
                    IntErrorKind::NegOverflow => unreachable!(),
                    IntErrorKind::InvalidDigit => unreachable!(),
                    IntErrorKind::Zero => unreachable!(),
                    _ => unreachable!()
                }
            }
        }
    }

    fn parse_primitive(token: SpannedToken<'src>) -> InnerParserResult<'diag, Option<PrimitiveValue<'src>>> {
        Ok(Some(match token.node {
            Token::IntLiteral { digits, radix, suffix } => {
                Self::parse_int(digits, radix, suffix, token.span)?
            },
            Token::FloatLiteral { literal, suffix } => {
                todo!()
            }
            Token::True => PrimitiveValue::True,
            Token::False => PrimitiveValue::False,
            Token::CharLiteral(char) => PrimitiveValue::Char(char),
            Token::StringLiteral(string) => PrimitiveValue::String(Cow::Owned(string)),
            Token::RawStringLiteral(string_ref) => PrimitiveValue::String(Cow::Borrowed(string_ref)),
            _ => return Ok(None)
        }))
    }

    pub fn parse_full(mut self) -> ParseResult<'src, 'diag> {
        while self.tokens.peek().is_some() {
            let stmt_res = self.parse_statement();
            match stmt_res {
                Ok(stmt) => self.res.stmts.push(stmt),
                Err(err) => {
                    self.res.diags.push(*err.diag);
                    if err.should_skip_until_new_stmt {
                        for token in self.tokens.by_ref() {
                            match token.node {
                                Token::Semicolon | Token::CloseBrace => break,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        self.res
    }
}

#[cfg(test)]
mod tests {
    use runec_ast::ast_type::SpannedTypeAnnotation;
    use runec_ast::statement::StmtBlock;
    use runec_source::source_map::SourceMap;
    use crate::generate_source;
    use crate::lexer::lexer_struct::Lexer;
    use super::*;

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
}
