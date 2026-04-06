use std::borrow::Cow;
use std::iter::Peekable;
use std::num::IntErrorKind;
use std::vec::IntoIter;
use fluent::FluentValue;
use runec_ast::ast_type::{SpannedTypeAnnotation, TypeAnnotation};
use runec_ast::expression::{Expr, PrimitiveValue, SpannedExpr, IntSuffix, FloatSuffix};
use runec_ast::operators::{BinaryOp, UnaryOp};
use runec_ast::SpannedStr;
use runec_ast::statement::{DestructPattern, FunctionArg, SpannedDestructPattern, SpannedStmt, SpannedStmtBlock, Stmt};
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::{DiagLabel, DiagNote};
use runec_errors::make_simple_diag;
use runec_errors::message::DiagMessage;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::{Source, SourceId, SourceMap};
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

pub(super) type InnerParserResult<'diag, T> = Result<T, InnerParseErr<'diag>>;

#[derive(Debug)]
pub(super) struct InnerParseErr<'diag> {
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
    source_file: &'src Source,
    source_hi: BytePos,
    res: ParseResult<'src, 'diag>
}

impl<'src, 'diag> Parser<'src, 'diag> {
    pub fn new(tokens: Vec<SpannedToken<'src>>, source_id: SourceId, source_map: &'src SourceMap) -> Self {
        let source_file = source_map.get_file(&source_id).unwrap();
        Self {
            tokens: tokens.into_iter().peekable(),
            source_id,
            source_hi: BytePos::from_usize(source_file.src().len()),
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
                            Cow::Owned(self.source_file.path().display().to_string())
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

    fn parse_stmt(&mut self) -> InnerParserResult<'diag, SpannedStmt<'src>> {
        let token = self.peek()?;
        match token.node {
            Token::Act => self.parse_act(),
            Token::Let => self.parse_let(),
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

        if let Some(args_hi) = args_terminating_hi {
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
        else if let Some(args_lo) = args_lo_opt {
            Err(InnerParseErr::without_skip(
                make_simple_diag!(
                    error;
                    "unterminated-args-block",
                    (self.source_id => args_lo..self.source_hi)
                )
            ))
        }
        else {
            Err(self.unexpected_eof())
        }
    }

    fn parse_let(&mut self) -> InnerParserResult<'diag, SpannedStmt<'src>> {
        let lo = expect_token!(self, Token::Let, Token::Let.display())?.span.lo;

        let is_mutable = if self.tokens.peek().is_some_and(|t| t.node == Token::Mut) {
            self.tokens.next();
            true
        } else {
            false
        };

        let pattern = self.parse_destruct_pattern()?;

        let ty = if self.tokens.peek().is_some_and(|t| t.node == Token::Colon) {
            self.tokens.next();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let init_expr = if self.tokens.peek().is_some_and(|t| t.node == Token::Eq) {
            self.tokens.next();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        let hi = expect_token!(self, Token::Semicolon, Token::Semicolon.display())?.span.hi;

        Ok(SpannedStmt::new(Stmt::DefineLet {
            pattern,
            is_mutable,
            ty,
            init_expr,
        }, Span::new(lo, hi, self.source_id)))
    }

    fn parse_destruct_primary(&mut self) -> InnerParserResult<'diag, SpannedDestructPattern<'src>> {
        let token = expect_token!(self, Token::Ident ( .. ) | Token::OpenParen, ["identifier", Token::OpenParen.display()], *)?;
        match token.node {
            Token::Ident(ident) => Ok(
                SpannedDestructPattern::new(
                    DestructPattern::Ident(ident),
                    token.span
                )
            ),
            Token::OpenParen => {
                let lo = token.span.lo;
                let mut patterns = Vec::new();
                let mut terminating_hi = None;
                while self.tokens.peek().is_some() {
                    let pattern = self.parse_destruct_pattern()?;
                    patterns.push(pattern);
                    let token = expect_token!(self, Token::Comma | Token::CloseParen, [Token::Comma.display(), Token::CloseParen.display()], *)?;
                    if token.node == Token::CloseParen {
                        terminating_hi = Some(token.span.hi);
                        break;
                    }
                }

                if let Some(hi) = terminating_hi {
                    Ok(SpannedDestructPattern::new(DestructPattern::Tuple(patterns.into_boxed_slice()), Span::new(lo, hi, self.source_id)))
                } else {
                    Err(self.unexpected_eof())
                }
            }
            _ => unreachable!()
        }
    }

    pub(super) fn parse_destruct_pattern(&mut self) -> InnerParserResult<'diag, SpannedDestructPattern<'src>> {
        let mut pat = self.parse_destruct_primary()?;
        while let Some(token) = self.tokens.peek() {
            match token.node {
                Token::Dot => {
                    self.tokens.next();
                    let token = expect_token!(self, Token::Ident ( .. ), "identifier")?;
                    let Token::Ident(attr) = token.node else { unreachable!() };
                    let span = Span::new(pat.span.lo, token.span.hi, self.source_id);
                    pat = SpannedDestructPattern::new(
                        DestructPattern::AttributeAccess {
                            pattern: Box::new(pat),
                            attribute: SpannedStr::new(attr, token.span),
                        }, span
                    );
                }
                _ => break,
            }
        }
        Ok(pat)
    }

    fn parse_type_primary(&mut self) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        if let Some(token) = self.tokens.next() {
            let lo = token.span.lo;
            match &token.node {
                Token::Ident(ident) => Ok(SpannedTypeAnnotation::new(TypeAnnotation::Ident(ident), Span::new(lo, token.span.hi, self.source_id))),
                Token::OpenParen => {
                    let mut items = Vec::new();
                    let mut terminating_hi = None;
                    while let Some(token) = self.tokens.peek() {
                        if token.node == Token::CloseParen {
                            terminating_hi = Some(token.span.hi);
                            self.tokens.next();
                            break;
                        }
                        items.push(self.parse_type_annotation()?);
                        if self.tokens.peek().is_some_and(|t| t.node == Token::Comma) {
                            self.tokens.next();
                        }
                    }
                    if let Some(hi) = terminating_hi {
                        Ok(SpannedTypeAnnotation::new(
                            {
                                if items.is_empty() {
                                    TypeAnnotation::Unit
                                } else {
                                    TypeAnnotation::Tuple(items.into_boxed_slice())
                                }
                            },
                            Span::new(lo, hi, self.source_id)
                        ))
                    } else {
                        Err(InnerParseErr::without_skip(
                            make_simple_diag!(
                                error;
                                "unterminated-tuple-type-annotation",
                                (self.source_id => lo..self.source_hi)
                            )
                        ))
                    }
                }
                _ => todo!()
            }
        } else {
            Err(self.unexpected_eof())
        }
    }

    fn parse_array_type_postfix(&mut self, ty: SpannedTypeAnnotation<'src>) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        expect_token!(self, Token::OpenBracket, Token::OpenBracket.display())?;

        let length = self.parse_expr(0)?;

        let hi = expect_token!(self, Token::CloseBracket, Token::CloseBracket.display())?.span.hi;
        let lo = ty.span.lo;

        Ok(SpannedTypeAnnotation::new(TypeAnnotation::Array {
            item: Box::new(ty),
            length,
        }, Span::new(lo, hi, self.source_id)))
    }

    fn parse_generic_type_annotation(&mut self, ty: SpannedTypeAnnotation<'src>) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        todo!()
    }

    pub(super) fn parse_type_annotation(&mut self) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        let ty = self.parse_type_primary()?;
        self.parse_type_secondary(ty, true)
    }

    fn parse_path_type_annotation(&mut self, ty: SpannedTypeAnnotation<'src>) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        let mut path = vec![ty];
        while let Some(token) = self.tokens.peek() {
            if token.node == Token::DColon {
                self.tokens.next();
                let ty = self.parse_type_primary()?;
                let ty = self.parse_type_secondary(ty, false)?;
                path.push(ty)
            }
            else {
                break;
            }
        }
        // SAFETY: `path` always has more than 2 elements
        let lo = path.first().unwrap().span.lo;
        let hi = path.last().unwrap().span.hi;
        Ok(SpannedTypeAnnotation::new(TypeAnnotation::Path { from_root: false, path: path.into_boxed_slice() }, Span::new(lo, hi, self.source_id)))
    }

    fn parse_type_secondary(&mut self, mut ty: SpannedTypeAnnotation<'src>, parse_path: bool) -> InnerParserResult<'diag, SpannedTypeAnnotation<'src>> {
        loop {
            match self.tokens.peek().map(|t| &t.node) {
                Some(Token::OpenBracket) => ty = self.parse_array_type_postfix(ty)?,
                Some(Token::Lt) => ty = self.parse_generic_type_annotation(ty)?,
                Some(Token::DColon) if !parse_path => ty = self.parse_path_type_annotation(ty)?,
                _ => break,
            }
        }

        Ok(ty)
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
                _ => stmts.push(self.parse_stmt()?)
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
                    let lo = self.bump()?.span.lo;
                    let expr = self.parse_expr(0)?;
                    if self.tokens.peek().is_some_and(|t| t.node == Token::Comma) {
                        self.tokens.next();
                        let mut exprs = vec![expr];
                        let mut terminating_hi = None;
                        while self.tokens.peek().is_some() {
                            exprs.push(self.parse_expr(0)?);
                            if self.tokens.peek().is_some_and(|t| t.node == Token::Comma) {
                                self.tokens.next();
                            }
                            if self.tokens.peek().is_some_and(|t| t.node == Token::CloseParen) {
                                terminating_hi = Some(self.tokens.next().unwrap().span.hi);
                                break;
                            }
                        }
                        if let Some(hi) = terminating_hi {
                            SpannedExpr::new(
                                Expr::Tuple(exprs.into_boxed_slice()),
                                Span::new(lo, hi, self.source_id),
                            )
                        }
                        else {
                            return Err(InnerParseErr::without_skip(
                                make_simple_diag!(
                                    error;
                                    "unterminated-tuple",
                                    (self.source_id => lo..self.source_hi)
                                )
                            ))
                        }
                    }
                    else {
                        expect_token!(self, Token::CloseParen, Token::CloseParen.display())?;
                        expr
                    }
                }
                Token::OpenBrace => {
                    let stmt_block = self.parse_stmt_block()?;
                    let span = stmt_block.span;
                    SpannedExpr::new(Expr::Block(stmt_block), span)
                }
                Token::OpenBracket => {
                    let lo = self.bump()?.span.lo;
                    let expr = self.parse_expr(0)?;
                    match self.tokens.peek() {
                        Some(spanned_token) => {
                            match spanned_token.node {
                                Token::Comma => {
                                    self.tokens.next();
                                    let mut exprs = vec![expr];
                                    let mut terminating_hi = None;
                                    while self.tokens.peek().is_some() {
                                        exprs.push(self.parse_expr(0)?);
                                        if self.tokens.peek().is_some_and(|t| t.node == Token::Comma) {
                                            self.tokens.next();
                                        }
                                        if self.tokens.peek().is_some_and(|t| t.node == Token::CloseBracket) {
                                            terminating_hi = Some(self.bump()?.span.hi);
                                            break;
                                        }
                                    }
                                    if let Some(hi) = terminating_hi {
                                        SpannedExpr::new(Expr::FullyDefinedArray(exprs.into_boxed_slice()), Span::new(lo, hi, self.source_id))
                                    }
                                    else {
                                        return Err(InnerParseErr::without_skip(
                                            make_simple_diag!(
                                                error;
                                                "unterminated-array",
                                                (self.source_id => lo..self.source_hi)
                                            )
                                        ))
                                    }
                                }
                                Token::Semicolon => {
                                    self.tokens.next();
                                    let count = self.parse_expr(0)?;
                                    let hi = expect_token!(self, Token::CloseBracket, Token::CloseBracket.display())?.span.hi;
                                    SpannedExpr::new(Expr::RepeatingArray {
                                        value: Box::new(expr),
                                        count: Box::new(count),
                                    }, Span::new(lo, hi, self.source_id))
                                }
                                _ => return Err(unexpected_token!(spanned_token.node, [Token::Comma.display(), Token::Semicolon.display()], *))
                            }
                        }
                        None => {
                            return Err(InnerParseErr::without_skip(
                                make_simple_diag!(
                                    error;
                                    "unterminated-array",
                                    (self.source_id => lo..self.source_hi)
                                )
                            ))
                        }
                    }
                }
                _ => return Err(InnerParseErr::with_skip(Self::unexpected_token(token.node.display())))
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
                    self.tokens.next();
                    let mut args = Vec::new();
                    let mut terminating_hi = None;
                    while let Some(token) = self.tokens.peek() {
                        match token.node {
                            Token::CloseParen => {
                                terminating_hi = Some(self.bump()?.span.hi);
                                break;
                            }
                            _ => {
                                args.push(self.parse_expr(0)?);
                                if self.tokens.peek().is_some_and(|t| t.node == Token::Comma) {
                                    self.tokens.next();
                                }
                            }
                        }
                    }
                    let lo = lhs.span.lo;
                    if let Some(hi) = terminating_hi {
                        lhs = SpannedExpr::new(Expr::Call {
                            callee: Box::new(lhs),
                            args: args.into_boxed_slice()
                        }, Span::new(lo, hi, self.source_id));
                    } else {
                        return Err(
                            InnerParseErr::without_skip(
                                make_simple_diag!(
                                    error;
                                    "unterminated-args-block",
                                    (self.source_id => lo..self.source_hi)
                                )
                            )
                        )
                    }
                }
                Token::Dot => {
                    self.tokens.next();
                    let ident_token = expect_token!(self, Token::Ident(..), "identifier")?;
                    let span = ident_token.span;
                    let lo = lhs.span.lo;
                    let hi = ident_token.span.hi;
                    let Token::Ident(ident) = ident_token.node else { unreachable!() };
                    lhs = SpannedExpr::new(Expr::AttributeAccess {
                        value: Box::new(lhs),
                        name: SpannedStr::new(ident, span),
                    }, Span::new(lo, hi, self.source_id))
                }
                Token::DColon => {
                    todo!()
                }
                Token::OpenBracket => {
                    unimplemented!()
                }
                Token::OpenBrace => {
                    unimplemented!()
                },
                Token::As => {
                    self.tokens.next();
                    let ty = self.parse_type_annotation()?;
                    let span = Span::new(lhs.span.lo, ty.span.hi, self.source_id);
                    lhs = SpannedExpr::new(Expr::TypeCast {
                        from: Box::new(lhs),
                        ty: Box::new(ty)
                    }, span)
                }
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

    fn parse_float(literal: &'src str, suffix_opt: Option<&'src str>, span: Span) -> InnerParserResult<'diag, PrimitiveValue<'src>> {
        match literal.parse::<f64>() {
            Ok(value) => {
                let suffix = match suffix_opt {
                    Some(suffix) => Some(match suffix {
                        "f32"  => FloatSuffix::F32,
                        "f64"  => FloatSuffix::F64,
                        _ => return Err(
                            InnerParseErr::with_skip(
                                make_simple_diag!(
                                    error; "unsupported-float-suffix",
                                    ( span.src_id => span.lo..span.hi ),
                                    { note = "supported-suffixes-float" }
                                )
                            )
                        )
                    }),
                    None => None
                };
                Ok(PrimitiveValue::Float {
                    value,
                    suffix
                })
            }
            Err(_) => Err(
                InnerParseErr::with_skip(
                    make_simple_diag!(
                        error; "unable-to-parse-float-number",
                        ( span.src_id => span.lo..span.hi ),
                    )
                )
            ),
        }
    }

    fn parse_primitive(token: SpannedToken<'src>) -> InnerParserResult<'diag, Option<PrimitiveValue<'src>>> {
        Ok(Some(match token.node {
            Token::IntLiteral { digits, radix, suffix } => {
                Self::parse_int(digits, radix, suffix, token.span)?
            },
            Token::FloatLiteral { literal, suffix } => {
                Self::parse_float(literal, suffix, token.span)?
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
            let stmt_res = self.parse_stmt();
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

