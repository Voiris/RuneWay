use std::borrow::Cow;
use std::iter::Peekable;
use std::vec::IntoIter;
use fluent::FluentValue;
use runec_ast::expression::Expr;
use runec_ast::statement::Stmt;
use runec_errors::diagnostics::Diagnostic;
use runec_errors::message::DiagMessage;
use runec_source::source_map::{SourceFile, SourceId};
use crate::lexer::token::{SpannedToken, Token};
use crate::parser::result::ParseResult;

macro_rules! expect_token {
    ($self:expr, $expected:pat, $expected_str:expr) => {{
        if let Some(token) = $self.tokens.next() {
            match token.node {
                $expected => Ok(()),
                token => Err(
                    InnerParseErr::with_skip(Diagnostic::error(
                        DiagMessage::new_with_args(
                            "unexpected-token",
                            runec_utils::hashmap!(
                                "expected" => FluentValue::String(Cow::Borrowed($expected_str)),
                                "got" => FluentValue::String(Cow::Borrowed(token.display())),
                            )
                        )
                    ))
                ),
            }
        } else {
            Err(InnerParseErr::with_skip(Diagnostic::error(
                DiagMessage::new_with_args(
                    "unexpected-eof",
                    runec_utils::hashmap!(
                        "path" => FluentValue::String(
                            Cow::Owned($self.source_file.file_name.to_string())
                        )
                    )
                )
            )))
        }
    }};
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
}

impl<'src, 'diag> Parser<'src> {
    pub fn new(tokens: Vec<SpannedToken<'src>>, source_id: SourceId, source_file: &'src SourceFile) -> Self {
        Self { tokens: tokens.into_iter().peekable(), source_id, source_file }
    }

    fn unexpected_token(got: &'static str) -> Box<Diagnostic<'diag>> {
        Diagnostic::error(
            DiagMessage::new_with_args("unexpected-token", runec_utils::hashmap!(
                "token" => FluentValue::String(Cow::Owned(got.to_string())),
            ))
        )
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

        unimplemented!()
    }

    fn parse_expression(&mut self) -> InnerParserResult<'diag, Expr<'src>> {
        unimplemented!()
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
