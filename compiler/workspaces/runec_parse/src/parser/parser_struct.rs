use std::borrow::Cow;
use std::iter::Peekable;
use std::vec::IntoIter;
use fluent::FluentValue;
use runec_ast::ast_type::TypeAnnotation;
use runec_ast::expression::Expr;
use runec_ast::statement::{FunctionArg, Stmt, StmtBlock};
use runec_errors::diagnostics::Diagnostic;
use runec_errors::message::DiagMessage;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::{SourceFile, SourceId, SourceMap};
use crate::lexer::token::{ComplexLiteral, SpannedToken, Token};
use crate::parser::result::ParseResult;

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

pub struct Parser<'src> {
    tokens: Peekable<IntoIter<SpannedToken<'src>>>,
    source_id: SourceId,
    source_file: &'src SourceFile,
    source_hi: BytePos
}

impl<'src, 'diag> Parser<'src> {
    pub fn new(tokens: Vec<SpannedToken<'src>>, source_id: SourceId, source_map: &'src SourceMap) -> Self {
        let source_file = source_map.get_file(&source_id).unwrap();
        Self {
            tokens: tokens.into_iter().peekable(),
            source_id,
            source_hi:BytePos::from_usize(source_file.src.len()),
            source_file
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

    fn parse_statement(&mut self) -> InnerParserResult<'diag, Stmt<'src>> {
        let token = self.tokens.next().unwrap();
        match token.node {
            Token::Act => self.parse_act(),
            _ => {
                Err(InnerParseErr::with_skip(Self::unexpected_token(token.node.display())))
            }
        }
    }

    fn parse_act(&mut self) -> InnerParserResult<'diag, Stmt<'src>> {
        expect_token!(self, Token::Act, Token::Act.display())?;

        let ident = if let Some(token) = self.tokens.next() {
            match &token.node {
                Token::ComplexLiteral(literal) => {
                    match **literal {
                        ComplexLiteral::Ident(ident) => ident,
                        _ => return Err(unexpected_token!(token, "identifier")),
                    }
                }
                _ => return Err(unexpected_token!(token, "identifier")),
            }
        } else {
            return Err(self.unexpected_eof());
        };

        expect_token!(self, Token::OpenParen, Token::OpenParen.display())?;
        let mut args = Vec::new();
        let mut terminated = false;
        let mut lo_opt = None;
        while let Some(token) = self.tokens.next() {
            lo_opt.get_or_insert(token.span.lo);
            match &token.node {
                Token::ComplexLiteral(literal) => {
                    match literal.as_ref() {
                        ComplexLiteral::Ident(ident) => {
                            expect_token!(self, Token::Colon, Token::Colon.display())?;
                            let type_literal_token = expect_token!(self, Token::ComplexLiteral( .. ), "identifier")?;
                            match &type_literal_token.node {
                                Token::ComplexLiteral(type_literal) => {
                                    match type_literal.as_ref() {
                                        ComplexLiteral::Ident(ty) => {
                                            args.push(
                                                FunctionArg { ident, ty }
                                            );
                                        }
                                        _ => return Err(unexpected_token!(type_literal_token, "identifier")),
                                    }
                                }
                                _ => unreachable!()
                            }
                        },
                        _ => return Err(unexpected_token!(token, "identifier")),
                    }
                }
                _ => return Err(unexpected_token!(token, "identifier")),
            }
            let token = expect_token!(self, Token::CloseParen | Token::Comma, [Token::CloseParen.display(), Token::Comma.display()], *)?;
            if token.node == Token::CloseParen {
                terminated = true;
                break;
            } else if self.tokens.peek().is_some_and(|t| t.node == Token::CloseParen) {
                self.tokens.next();
                terminated = true;
                break;
            }
        }

        if !terminated {
            return if let Some(lo) = lo_opt {
                Err(
                    InnerParseErr::without_skip(
                        runec_errors::make_simple_diag!(
                            error;
                            "unterminated-args-block",
                            (self.source_id => lo..self.source_hi)
                        )
                    )
                )
            } else {
                Err(self.unexpected_eof())
            }
        }

        let ret_ty = if self.tokens.peek().is_some_and(|t| t.node == Token::Arrow) {
            self.tokens.next();

            self.parse_type_annotation()?
        } else { TypeAnnotation::Unit };

        let stmt_block = self.parse_stmt_block()?;

        Ok(Stmt::DefineFunction {
            ident,
            args: args.into_boxed_slice(),
            ret_ty,
            body: stmt_block
        })
    }

    fn parse_type_annotation(&mut self) -> InnerParserResult<'diag, TypeAnnotation<'src>> {
        if let Some(token) = self.tokens.next() {
            match &token.node {
                Token::ComplexLiteral(c_literal) => {
                    let ident = match **c_literal {
                        ComplexLiteral::Ident(ident) => ident,
                        _ => return Err(unexpected_token!(token, "identifier")),
                    };
                    Ok(TypeAnnotation::Ident(ident))
                }
                _ => todo!()
            }
        } else {
            Err(self.unexpected_eof())
        }
    }

    fn parse_stmt_block(&mut self) -> InnerParserResult<'diag, StmtBlock<'src>> {
        let mut stmts = Vec::new();
        let mut terminated = false;
        let mut lo_opt = None;

        while let Some(token) = self.tokens.peek() {
            lo_opt.get_or_insert(token.span.lo);
            match token.node {
                Token::CloseBrace => {
                    self.tokens.next();
                    terminated = true;
                    break;
                }
                _ => stmts.push(self.parse_statement()?)
            }
        }

        if !terminated {
            return if let Some(lo) = lo_opt {
                Err(
                    InnerParseErr::without_skip(
                        runec_errors::make_simple_diag!(
                            error;
                            "unterminated-code-block",
                            (self.source_id => lo..self.source_hi)
                        )
                    )
                )
            } else {
                Err(self.unexpected_eof())
            }
        }

        Ok(stmts.into_boxed_slice() as StmtBlock<'src>)
    }

    fn parse_expression(&mut self) -> InnerParserResult<'diag, Expr<'src>> {
        todo!()
    }

    pub fn parse_full(mut self) -> ParseResult<'src, 'diag> {
        let mut res = ParseResult::new();

        while self.tokens.peek().is_some() {
            let stmt_res = self.parse_statement();
            match stmt_res {
                Ok(stmt) => res.stmts.push(stmt),
                Err(err) => {
                    res.diags.push(*err.diag);
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

        res
    }
}

#[cfg(test)]
mod tests {
    use runec_source::source_map::SourceMap;
    use crate::generate_source;
    use crate::lexer::lexer_struct::Lexer;
    use super::*;

    fn lex_source(source_map: &SourceMap, source_id: SourceId) -> Vec<SpannedToken> {
        let lexer = Lexer::new(source_id, source_map);
        lexer.lex_full().unwrap()
    }

    #[test]
    fn act_parse_test() {
        let (source_map, source_id) = generate_source("");
        let tokens = lex_source(&source_map, source_id);
        let parse_result = Parser::new(tokens, source_id, &source_map).parse_full();

        assert_eq!(parse_result.diags.len(), 0);

        let expected_stmts = [

        ];

        assert_eq!(parse_result.stmts, expected_stmts);
    }
}
