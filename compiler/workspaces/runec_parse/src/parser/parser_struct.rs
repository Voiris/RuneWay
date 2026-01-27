use std::iter::Peekable;
use std::vec::IntoIter;
use runec_ast::expression::Expr;
use runec_ast::statement::Stmt;
use runec_errors::diagnostics::Diagnostic;
use crate::lexer::token::SpannedToken;
use crate::parser::result::ParseResult;

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
    tokens: Peekable<IntoIter<SpannedToken<'src>>>
}

impl<'src, 'diag> Parser<'src> {
    pub fn new(tokens: Vec<SpannedToken<'src>>) -> Self {
        Self { tokens: tokens.into_iter().peekable() }
    }

    fn parse_expression(&mut self) -> InnerParserResult<'diag, Expr<'src>> {
        unimplemented!()
    }

    fn parse_statement(&mut self) -> InnerParserResult<'diag, Stmt<'src>> {
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

                    }
                }
            }
        }

        res
    }
}
