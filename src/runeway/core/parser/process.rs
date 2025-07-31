use super::super::ast::expression::{Expr, FStringExpr};
use super::super::ast::statement::Statement;
use super::super::lexer::token::{FStringPart, Token};
use crate::runeway::core::ast::expression::SpannedExpr;
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::ast::statement::{
    AnnotatedParameter, ImportItem, ImportSymbol, SpannedStatement,
};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::lexer::token::SpannedToken;
use crate::runeway::core::spanned::Spanned;

pub struct ParserProcess {
    tokens: Vec<SpannedToken>,
    filename: String,
    pos: usize,
}

impl ParserProcess {
    pub fn new(tokens: Vec<SpannedToken>, filename: String) -> Self {
        ParserProcess {
            tokens,
            filename,
            pos: 0,
        }
    }

    fn peek_offset(&self, offset: isize) -> RWResult<&SpannedToken> {
        match self.tokens.get(match self.pos.checked_add_signed(offset) {
            Some(i) => i,
            None => panic!("Tokens position [usize] is overflow"),
        }) {
            Some(token) => Ok(token),
            None => {
                Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected EOF"))
            }
        }
    }

    fn peek(&self) -> RWResult<&SpannedToken> {
        match self.tokens.get(self.pos) {
            Some(val) => Ok(val),
            None => {
                Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected EOF"))
            }
        }
    }

    fn forward(&mut self) -> RWResult<&SpannedToken> {
        self.pos += 1;
        // println!("NOW TOKEN: {:?}", self.tokens.get(self.pos));
        Ok(self.peek()?)
    }

    fn consume(&mut self, expected: &Token) -> RWResult<bool> {
        Ok(if self.peek_is(expected)? {
            self.forward()?;
            true
        } else {
            false
        })
    }

    fn consume_get(&mut self, expected: &Token) -> RWResult<Option<SpannedToken>> {
        let peek = self.peek()?.clone();
        if self.consume(expected)? {
            Ok(Some(peek))
        } else {
            Ok(None)
        }
    }

    fn consume_statement_end(&mut self) -> RWResult<SpannedToken> {
        let peek = self.peek()?.clone();
        if self.consume(&Token::Semicolon)? {
            Ok(peek)
        } else {
            let peek = self.peek()?;
            Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                .with_message("Unexpected token")
                .with_label("Expected `;`. Got this", &peek.span, &self.filename))
        }
    }

    fn peek_is(&mut self, expected: &Token) -> RWResult<bool> {
        Ok(
            if std::mem::discriminant(&self.peek()?.node) == std::mem::discriminant(expected) {
                true
            } else {
                false
            },
        )
    }

    fn peek_offset_is(&mut self, expected: &Token, offset: isize) -> RWResult<bool> {
        Ok(
            if std::mem::discriminant(&self.peek_offset(offset)?.node)
                == std::mem::discriminant(expected)
            {
                true
            } else {
                false
            },
        )
    }

    pub fn parse_full(&mut self) -> RWResult<Vec<SpannedStatement>> {
        let mut statements: Vec<SpannedStatement> = Vec::new();

        while !self.peek_is(&Token::EOF)? {
            // println!("Handling module statement: {:?}", self.peek()?);
            match self.parse_statement()? {
                Some(statement) => statements.push(statement),
                None => (),
            }
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> RWResult<Option<SpannedStatement>> {
        // println!("Handling statement: {:?}", self.peek()?);
        match self.peek()?.node {
            Token::Let => Ok(Some(self.parse_let()?)),
            Token::Act => Ok(Some(self.parse_act()?)),
            Token::Return => Ok(Some(self.parse_return()?)),
            Token::If => Ok(Some(self.parse_if()?)),
            Token::While => Ok(Some(self.parse_while()?)),
            Token::For => Ok(Some(self.parse_for()?)),
            Token::Break => {
                let token = self.consume_get(&Token::Break)?.unwrap();

                self.consume_statement_end()?;

                Ok(Some(SpannedStatement::new(Statement::Break, token.span)))
            }
            Token::Continue => {
                let token = self.consume_get(&Token::Continue)?.unwrap();

                self.consume_statement_end()?;

                Ok(Some(SpannedStatement::new(Statement::Continue, token.span)))
            }
            Token::Assert => {
                let token = self.consume_get(&Token::Assert)?.unwrap();

                let expr = self.parse_expr()?.clone();

                self.consume_statement_end()?;

                Ok(Some(SpannedStatement::new(
                    Statement::Assert(expr.clone()),
                    token.span.start..expr.span.end,
                )))
            }
            Token::Identifier(_) => Ok(Some(self.parse_expr_statement()?)),
            Token::Semicolon => {
                self.forward()?;
                Ok(None)
            }
            Token::Import => Ok(Some(self.parse_import()?)),
            Token::Class => Ok(Some(self.parse_class()?)),
            _ => {
                let expr = self.parse_expr()?;
                let stmt = Statement::Expr(expr.clone());

                self.consume_statement_end()?;
                Ok(Some(SpannedStatement::new(stmt, expr.span)))
            }
        }
    }

    fn parse_class(&mut self) -> RWResult<SpannedStatement> {
        let class_token = self.consume_get(&Token::Class)?.unwrap();

        let peek = self.peek()?.clone();
        let name = match peek.node {
            Token::Identifier(name) => {
                self.forward()?;
                name
            }
            _ => {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token")
                    .with_label("Expected identifier. Got this", &peek.span, &self.filename));
            }
        };

        {
            let peek = self.peek()?;
            match &peek.node {
                Token::LBrace => (),
                _ => {
                    return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                        .with_message("Unexpected token")
                        .with_label("Expected identifier. Got this", &peek.span, &self.filename));
                }
            }
        }
        self.forward()?;

        let mut statements = Vec::new();

        let rbrace = loop {
            let peek = self.peek()?;
            match peek.node {
                Token::RBrace => break peek.clone(),
                _ => match self.parse_statement()? {
                    Some(statement) => statements.push(Box::new(statement)),
                    None => (),
                },
            }
        };
        self.forward()?;

        Ok(SpannedStatement::new(
            Statement::Class {
                name,
                body: statements,
            },
            class_token.span.start..rbrace.span.end,
        ))
    }

    fn parse_import(&mut self) -> RWResult<SpannedStatement> {
        let import_token = self.consume_get(&Token::Import)?.unwrap();

        let peek = self.peek()?;
        let path = match &peek.node {
            Token::StringLiteral(val) => val.clone(),
            _ => {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token")
                    .with_label(
                        "Expected string literal. Got this",
                        &peek.span,
                        &self.filename,
                    ));
            }
        };

        self.forward()?;

        let result = match &self.peek()?.node {
            Token::As => {
                self.forward()?;

                let peek = self.peek()?.clone();
                let result = match &peek.node {
                    Token::Identifier(alias) => Statement::Import {
                        path,
                        item: ImportItem::Alias(alias.clone()),
                    },
                    _ => {
                        return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                            .with_message("Unexpected token")
                            .with_label(
                                "Expected alias identifier. Got this",
                                &peek.span,
                                &self.filename,
                            ));
                    }
                };

                self.forward()?;

                result
            }
            Token::Get => {
                self.forward()?;

                let mut selective = Vec::new();

                loop {
                    match self.peek()?.node.clone() {
                        Token::Identifier(original) => {
                            self.forward()?;
                            let alias: Option<String> = if self.consume(&Token::As)? {
                                match self.peek()?.node.clone() {
                                    Token::Identifier(alias) => {
                                        self.forward()?;
                                        Some(alias)
                                    }
                                    token => panic!("Expected identifier, got {:?}", token),
                                }
                            } else {
                                None
                            };
                            selective.push(ImportSymbol { original, alias })
                        }
                        Token::Asterisk => {
                            self.forward()?;
                            let semicolon = self.consume_statement_end()?;
                            return Ok(SpannedStatement::new(
                                Statement::Import {
                                    path,
                                    item: ImportItem::All,
                                },
                                import_token.span.start..semicolon.span.start,
                            ));
                        }
                        token => panic!("Expected identifier or `*`, got {:?}", token),
                    }

                    if !self.consume(&Token::Comma)? {
                        break;
                    }
                }

                Statement::Import {
                    path,
                    item: ImportItem::Selective(selective),
                }
            }
            token => panic!("Expected `as` or `get`, got {:?}", token),
        };

        let semicolon = self.consume_statement_end()?;

        Ok(SpannedStatement::new(
            result,
            import_token.span.start..semicolon.span.start,
        ))
    }

    fn parse_let(&mut self) -> RWResult<SpannedStatement> {
        let let_token = self.consume_get(&Token::Let)?.unwrap();

        let name = if let Token::Identifier(name) = &self.peek()?.node {
            let name = name.clone();
            self.forward()?;
            name
        } else {
            return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                .with_message("Unexpected token")
                .with_label(
                    "Expected identifier. Got this",
                    &self.peek()?.span,
                    &self.filename,
                ));
        };

        let annotation = self.parse_annotation(&Token::Colon)?;

        let value: SpannedExpr = if !self.consume(&Token::Equal)? {
            let semicolon = self.consume_statement_end()?;
            return Ok(SpannedStatement::new(
                Statement::LetVoid { name, annotation },
                let_token.span.start..semicolon.span.start,
            ));
        } else {
            let expr = self.parse_expr()?;
            self.consume_statement_end()?;
            expr
        };

        Ok(SpannedStatement::new(
            Statement::Let {
                name,
                value: value.clone(),
                annotation,
            },
            let_token.span.start..value.span.end,
        ))
    }

    //noinspection DuplicatedCode
    fn parse_act(&mut self) -> RWResult<SpannedStatement> {
        let act_token = self.consume_get(&Token::Act)?.unwrap();

        let name = match &self.peek()?.node {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token")
                    .with_label(
                        "Expected identifier. Got this",
                        &self.peek()?.span,
                        &self.filename,
                    ));
            }
        };

        self.forward()?;

        if !self.consume(&Token::LParen)? {
            return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                .with_message("Unexpected token")
                .with_label("Expected `(`. Got this", &self.peek()?.span, &self.filename));
        }

        let mut parameters = Vec::new();

        loop {
            match &self.peek()?.node {
                Token::Identifier(param) => {
                    let param_name = param.clone();
                    self.forward()?;

                    let annotation = self.parse_annotation(&Token::Colon)?;

                    parameters.push(AnnotatedParameter {
                        name: param_name,
                        annotation,
                    });

                    if self.consume(&Token::Comma)? {
                        continue;
                    } else if self.consume(&Token::RParen)? {
                        break;
                    } else {
                        return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                            .with_message("Founded not closed bracket")
                            .with_label(
                                "Expected `)` or `,` or `:`. Got this",
                                &self.peek()?.span,
                                &self.filename,
                            ));
                    }
                }
                Token::RParen => {
                    self.forward()?;
                    break;
                }
                _ => {
                    return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                        .with_message("Founded not closed bracket")
                        .with_label("Expected `)`. Got this", &self.peek()?.span, &self.filename));
                }
            }
        }

        let return_annotation = self.parse_annotation(&Token::Arrow)?;

        let body = self.parse_body()?;

        Ok(SpannedStatement::new(
            Statement::Act {
                name,
                parameters,
                return_annotation,
                body: body.node,
            },
            act_token.span.start..body.span.end,
        ))
    }

    fn parse_annotation(&mut self, annotation_token: &Token) -> RWResult<Option<Spanned<String>>> {
        if self.consume(annotation_token)? {
            let peek = self.peek()?.clone();
            if let Token::Identifier(annotation) = peek.node {
                let annotation = annotation.clone();
                let span = peek.span;
                self.forward()?;
                Ok(Some(Spanned::new(annotation, span)))
            } else {
                Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token")
                    .with_label(
                        "Expected identifier. Got this",
                        &self.peek()?.span,
                        &self.filename,
                    ))
            }
        } else {
            Ok(None)
        }
    }

    fn parse_return(&mut self) -> RWResult<SpannedStatement> {
        let return_token = self.consume_get(&Token::Return)?.unwrap();

        let expr = self.parse_expr()?;

        self.consume_statement_end()?;

        Ok(SpannedStatement::new(
            Statement::Return(expr.clone()),
            return_token.span.start..expr.span.end,
        ))
    }

    fn parse_if(&mut self) -> RWResult<SpannedStatement> {
        let if_token = self.consume_get(&Token::If)?.unwrap();

        let condition = self.parse_expr()?;

        if !self.consume(&Token::LBrace)? {
            return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                .with_message("Founded not closed bracket")
                .with_label("Expected `}`. Got this", &self.peek()?.span, &self.filename));
        }

        let mut then_branch = Vec::new();

        let rbrace = loop {
            match self.consume_get(&Token::RBrace)? {
                Some(brace) => break brace,
                None => (),
            }
            match self.parse_statement()? {
                Some(statement) => then_branch.push(Box::new(statement)),
                None => (),
            }
        };

        Ok(if self.consume(&Token::Else)? {
            let mut else_branch = Vec::new();

            let end = if self.peek_is(&Token::If)? {
                else_branch.push(Box::new(self.parse_if()?));
                else_branch.last().unwrap().span.end
            } else {
                let body = self.parse_body()?;

                else_branch.extend(body.node);

                body.span.end
            };

            SpannedStatement::new(
                Statement::If {
                    condition,
                    then_branch,
                    else_branch: Some(else_branch),
                },
                if_token.span.start..end,
            )
        } else {
            SpannedStatement::new(
                Statement::If {
                    condition,
                    then_branch,
                    else_branch: None,
                },
                if_token.span.start..rbrace.span.end,
            )
        })
    }

    fn parse_while(&mut self) -> RWResult<SpannedStatement> {
        let while_token = self.consume_get(&Token::While)?.unwrap();

        let condition = self.parse_expr()?;

        let body = self.parse_body()?;

        Ok(SpannedStatement::new(
            Statement::While {
                condition,
                body: body.node,
            },
            while_token.span.start..body.span.end,
        ))
    }

    //noinspection DuplicatedCode
    fn parse_for(&mut self) -> RWResult<SpannedStatement> {
        // Fix
        /* for i in [1, 2, 3] { } */
        let for_token = self.consume_get(&Token::For)?.unwrap();

        match &self.peek()?.node {
            Token::Identifier(variable) => {
                let variable = variable.clone();

                self.forward()?;
                if !self.consume(&Token::In)? {
                    return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                        .with_message("Unexpected token")
                        .with_label(
                            "Expected keyword `in`. Got this",
                            &self.peek()?.span,
                            &self.filename,
                        ));
                }

                let iterable = self.parse_expr()?;

                let body = self.parse_body()?;

                Ok(SpannedStatement::new(
                    Statement::For {
                        variable,
                        iterable,
                        body: body.node,
                    },
                    for_token.span.start..body.span.end,
                ))
            }
            _ => Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                .with_message("Unexpected token as target variable in `for` statement")
                .with_label(
                    "Expected identifier. Got this",
                    &self.peek()?.span,
                    &self.filename,
                )),
        }
    }

    fn parse_iterator(&mut self, start: SpannedExpr) -> RWResult<SpannedExpr> {
        let end = self.parse_expr()?;

        let step = if self.consume(&Token::DoubleColon)? {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        let span = start.span.start..end.span.end;
        Ok(SpannedExpr::new(
            Expr::Iterator {
                start: Box::new(start),
                end: Box::new(end),
                step,
            },
            span,
        ))
    }

    fn parse_set_attr(&mut self, obj: SpannedExpr) -> RWResult<SpannedExpr> {
        let value = self.parse_expr()?;

        let span = obj.span.start..value.span.end;
        Ok(SpannedExpr::new(
            Expr::SetAttr {
                object: Box::new(obj),
                value: Box::new(value),
            },
            span,
        ))
    }

    fn parse_body(&mut self) -> RWResult<Spanned<Vec<Box<SpannedStatement>>>> {
        let start = match self.consume_get(&Token::LBrace)? {
            Some(lbrace) => lbrace.span.start,
            None => {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token")
                    .with_label("Expected `{`. Got this", &self.peek()?.span, &self.filename));
            }
        };

        let mut statements = Vec::new();

        let end = loop {
            let peek = self.peek()?.clone();
            match peek.node {
                Token::RBrace => {
                    self.forward()?;
                    break peek.span.end;
                }
                Token::EOF => {
                    let span_point = statements
                        .last()
                        .cloned()
                        .map_or_else(|| start, |x: Box<SpannedStatement>| (*x).span.end + 1);
                    return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                        .with_message("Founded not closed bracket")
                        .with_label("Expected `}`", &(span_point..span_point), &self.filename));
                }
                _ => match self.parse_statement()? {
                    Some(statement) => statements.push(Box::new(statement)),
                    None => (),
                },
            }
        };

        Ok(Spanned::new(statements, start..end))
    }

    fn parse_expr_statement(&mut self) -> RWResult<SpannedStatement> {
        let expr = self.parse_expr()?;

        let statement = match &expr.node {
            Expr::Variable(name) => {
                let token = self.peek()?.clone();
                match token.node {
                    Token::Equal => {
                        self.forward()?;
                        let value = self.parse_expr()?;
                        let end = value.span.end;
                        SpannedStatement::new(
                            Statement::Assign {
                                name: name.clone(),
                                value,
                            },
                            expr.span.start..end,
                        )
                    }
                    Token::PlusEqual
                    | Token::MinusEqual
                    | Token::AsteriskEqual
                    | Token::SlashEqual
                    | Token::PercentEqual => {
                        let (operator, op_span) = match token.node {
                            Token::PlusEqual => (BinaryOperator::Add, token.span),
                            Token::MinusEqual => (BinaryOperator::Sub, token.span),
                            Token::AsteriskEqual => (BinaryOperator::Mul, token.span),
                            Token::SlashEqual => (BinaryOperator::Div, token.span),
                            Token::PercentEqual => (BinaryOperator::Mod, token.span),
                            _ => unreachable!(),
                        };
                        self.forward()?;
                        let value = self.parse_expr()?;
                        let start = expr.span.start;
                        let end = value.span.end;
                        SpannedStatement::new(
                            Statement::Assign {
                                name: name.clone(),
                                value: SpannedExpr::new(
                                    Expr::BinaryOperation {
                                        left_operand: Box::new(expr),
                                        right_operand: Box::new(value),
                                        operator,
                                    },
                                    op_span,
                                ),
                            },
                            start..end,
                        )
                    }
                    _ => SpannedStatement::new(Statement::Expr(expr.clone()), expr.span),
                }
            }
            _ => SpannedStatement::new(Statement::Expr(expr.clone()), expr.span),
        };

        self.consume_statement_end()?;

        Ok(statement)
    }

    fn parse_expr(&mut self) -> RWResult<SpannedExpr> {
        Ok(self.parse_binary_expr(0)?)
    }

    //noinspection DuplicatedCode
    fn parse_binary_expr(&mut self, min_precedence: u8) -> RWResult<SpannedExpr> {
        let unary_operator_token = self.peek()?.clone();
        let unary_operator = unary_operator_token.node.to_unary_operator();

        if unary_operator.is_some() {
            self.forward()?;
            let operand = self.parse_expr()?;
            let end = operand.span.end;

            return Ok(SpannedExpr::new(
                Expr::UnaryOperation {
                    operator: unary_operator.unwrap(),
                    operand: Box::new(operand),
                },
                unary_operator_token.span.start..end,
            ));
        }

        let mut left = self.parse_primary()?;

        if self.peek_is(&Token::Dot)? && self.peek_offset_is(&Token::Dot, 1)? {
            self.forward()?;
            self.forward()?;
            return self.parse_iterator(left.clone());
        } else if matches!(left.node, Expr::AttributeAccess { .. })
            && self.peek_is(&Token::Equal)?
        {
            self.forward()?;
            return self.parse_set_attr(left.clone());
        }

        while let Some(operator_token) = self.peek()?.node.to_binary_operator() {
            let precedence = operator_token.get_precedence();

            if precedence < min_precedence {
                break;
            }

            let binary_operator = operator_token;
            self.forward()?; // consume operator

            let right = self.parse_binary_expr(precedence + 1)?;
            let span = left.span.start..right.span.end;
            left = SpannedExpr::new(
                Expr::BinaryOperation {
                    left_operand: Box::new(left),
                    operator: binary_operator,
                    right_operand: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    //noinspection DuplicatedCode
    fn parse_arguments(&mut self) -> RWResult<Vec<SpannedExpr>> {
        let mut arguments = Vec::new();

        if self.consume(&Token::RParen)? {
            return Ok(arguments); // пустые аргументы ()
        }

        loop {
            let mut expr = self.parse_expr()?;
            arguments.push(expr);

            if self.consume(&Token::Comma)? {
                continue;
            } else if self.consume(&Token::RParen)? {
                break;
            } else {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Founded not closed bracket")
                    .with_label("Expected `)`. Got this", &self.peek()?.span, &self.filename));
            }
        }

        Ok(arguments)
    }

    fn parse_postfix(&mut self, mut expr: SpannedExpr) -> RWResult<SpannedExpr> {
        loop {
            match &self.peek()?.node {
                Token::Dot => {
                    self.forward()?;

                    let peek = self.peek()?;
                    match &peek.node {
                        Token::Dot => {
                            self.forward()?;
                            expr = self.parse_iterator(expr.clone())?;
                        }
                        Token::Identifier(field) => {
                            let start = expr.span.start;
                            expr = SpannedExpr::new(
                                Expr::AttributeAccess {
                                    object: Box::new(expr),
                                    field: field.clone(),
                                },
                                start..peek.span.end,
                            );
                            self.forward()?;
                        }
                        _ => {
                            return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                                .with_message("Unexpected token after `.`")
                                .with_label(
                                    "Expected `.` or identifier. Got this",
                                    &self.peek()?.span,
                                    &self.filename,
                                ));
                        }
                    }
                }
                Token::LParen => {
                    self.forward()?;

                    let args = self.parse_arguments()?;
                    let span = expr.span.start..self.peek_offset(-1)?.span.end;
                    expr = SpannedExpr::new(
                        Expr::Call {
                            callee: Box::new(expr),
                            arguments: args,
                        },
                        span,
                    );
                }
                Token::LBracket => {
                    self.forward()?;
                    let in_bracket_expr = self.parse_primary()?;
                    let end = {
                        let peek = self.peek()?;
                        match &peek.node {
                            Token::RBracket => {
                                let end = peek.span.end.clone();
                                self.forward()?;
                                end
                            }
                            _ => {
                                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                                    .with_message("Founded not closed bracket")
                                    .with_label(
                                        "Expected `]`. Got this",
                                        &self.peek()?.span,
                                        &self.filename,
                                    ));
                            }
                        }
                    };
                    let span = expr.span.start..end;
                    return Ok(SpannedExpr::new(
                        Expr::Slice {
                            object: Box::new(expr),
                            index: Box::new(in_bracket_expr),
                        },
                        span,
                    ));
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_primary(&mut self) -> RWResult<SpannedExpr> {
        let peek = self.peek()?.clone();
        let expr = match &peek.node {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.forward()?;
                Ok::<SpannedExpr, RuneWayError>(SpannedExpr::new(
                    Expr::String(s.to_string()),
                    peek.span,
                ))
            }
            Token::IntegerLiteral(i) => {
                let i = i.clone();
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Integer(i), peek.span))
            }
            Token::UIntegerLiteral(u) => {
                let u = u.clone();
                self.forward()?;
                Ok(SpannedExpr::new(Expr::UInteger(u), peek.span))
            }
            Token::FloatLiteral(f) => {
                let f = f.clone();
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Float(f), peek.span))
            }
            Token::FString(parts) => {
                let mut format_string = Vec::new();

                for part in parts {
                    match part.clone() {
                        FStringPart::StringLiteral(string) => {
                            format_string.push(FStringExpr::String(string.clone()));
                        }
                        FStringPart::Expr(mut expr, ..) => {
                            expr.push(SpannedToken::new(Token::EOF, 0..0));
                            format_string.push(FStringExpr::Expr(
                                ParserProcess::new(expr, self.filename.clone()).parse_expr()?,
                            ));
                        }
                    }
                }

                self.forward()?;

                Ok(SpannedExpr::new(Expr::FString(format_string), peek.span))
            }
            Token::True => {
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Boolean(true), peek.span))
            }
            Token::False => {
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Boolean(false), peek.span))
            }
            Token::Null => {
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Null, peek.span))
            }
            Token::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.forward()?;
                Ok(SpannedExpr::new(Expr::Variable(identifier), peek.span))
            }
            Token::LParen => {
                let start = self.peek()?.span.start.clone();
                self.forward()?;

                let null_end = self.peek()?.span.end.clone();
                if self.consume(&Token::RParen)? {
                    return Ok(self.parse_postfix(SpannedExpr::new(Expr::Null, start..null_end))?);
                }

                let mut expr = self.parse_expr()?;
                let peek = self.peek()?;
                match &peek.node {
                    Token::RParen => {
                        expr.span.start = start;
                        expr.span.end = peek.span.end.clone();
                        self.forward()?;
                        return Ok(self.parse_postfix(expr)?);
                    }
                    _ => (),
                }

                let mut exprs = vec![Box::new(expr)];
                let end = loop {
                    let peek = self.peek()?;
                    match peek.node {
                        Token::RParen => {
                            let end = self.peek()?.span.end.clone();
                            self.forward()?;
                            break end;
                        }
                        Token::Comma => {
                            self.forward()?;
                            let peek = self.peek()?;
                            if peek.node == Token::RParen {
                                break peek.span.end.clone();
                            }
                            let expr = self.parse_expr()?;
                            exprs.push(Box::new(expr));
                        }
                        _ => {
                            return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                                .with_message("Unexpected token")
                                .with_label(
                                    "Expected `)` or `,`. Got this",
                                    &self.peek()?.span,
                                    &self.filename,
                                ));
                        }
                    }
                };

                Ok(SpannedExpr::new(Expr::Tuple(exprs), start..end))
            }
            Token::LBrace => {
                let start = peek.span.start.clone();
                self.forward()?;

                let mut vec = Vec::new();

                let end = loop {
                    let peek = self.peek()?;
                    match peek.node {
                        Token::RBrace => {
                            let end = peek.span.end.clone();
                            self.forward()?;
                            break end;
                        }
                        _ => {
                            if matches!(peek.node, Token::Comma) && !vec.is_empty() {
                                self.forward()?;
                            }
                            let key_expr = self.parse_expr()?;
                            if !self.consume(&Token::Colon)? {
                                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                                    .with_message("Unexpected token")
                                    .with_label(
                                        "Expected `:`. Got this",
                                        &self.peek()?.span,
                                        &self.filename,
                                    ));
                            }
                            let value_expr = self.parse_expr()?;
                            vec.push((Box::new(key_expr), Box::new(value_expr)));
                        }
                    }
                };

                Ok(SpannedExpr::new(Expr::Dict(vec), start..end))
            }
            Token::LBracket => {
                self.forward()?;

                let mut list = Vec::new();

                let rbracket = loop {
                    match self.consume_get(&Token::RBracket)? {
                        Some(bracket) => break bracket,
                        None => (),
                    }
                    list.push(Box::new(self.parse_expr()?));

                    self.consume(&Token::Comma)?;
                };

                Ok(SpannedExpr::new(
                    Expr::List(list),
                    peek.span.start..rbracket.span.end,
                ))
            }
            _ => {
                return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Unexpected token in primary expression")
                    .with_label("Got this", &self.peek()?.span, &self.filename));
            }
        };
        self.parse_postfix(expr?)
    }
}
