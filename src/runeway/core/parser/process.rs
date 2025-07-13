use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::ast::statement::{ImportItem, ImportSymbol};
use crate::runeway::core::lexer::token::SpannedToken;
use crate::runeway::core::spanned::Span;
use super::super::lexer::token::{Token, FStringPart};
use super::super::ast::statement::Statement;
use super::super::ast::expression::{Expr, FStringExpr};

pub struct ParserProcess {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl ParserProcess {
    pub fn new(tokens: Vec<SpannedToken>) -> ParserProcess {
        ParserProcess {
            tokens,
            pos: 0
        }
    }

    fn peek_offset(&self, offset: isize) -> Result<&SpannedToken, String> {
        match self.tokens.get(
            match self.pos.checked_add_signed(offset) {
                Some(i) => i,
                None => Err("Tokens position [usize] is overflow".to_owned())?,
            }
        ) {
            Some(token) => Ok(token),
            None => Err("Unexpected EOF".to_owned())?,
        }
    }

    fn peek(&self) -> Result<&SpannedToken, String> {
        match self.tokens.get(self.pos) {
            Some(val) => Ok(val),
            None => Err("Unexpected EOF".to_owned())
        }
    }

    fn forward(&mut self) -> Result<&SpannedToken, String> {
        self.pos += 1;
        // println!("NOW TOKEN: {:?}", self.tokens.get(self.pos));
        Ok(self.peek()?)
    }

    fn consume(&mut self, expected: &Token) -> Result<bool, String> {
        Ok(if self.peek_is(expected)? {
            self.forward()?;
            true
        } else {
            false
        })
    }

    fn consume_statement_end(&mut self) -> Result<(), String> {
        if self.consume(&Token::Semicolon)? {
            Ok(())
        } else {
            Err(format!("Expected `;`. Got: {:?}", self.peek()? ))
        }
    }

    fn peek_is(&mut self, expected: &Token) -> Result<bool, String> {
        Ok(if std::mem::discriminant(&self.peek()?.node) == std::mem::discriminant(expected) {
            true
        } else {
            false
        })
    }

    fn peek_offset_is(&mut self, expected: &Token, offset: isize) -> Result<bool, String> {
        Ok(if std::mem::discriminant(&self.peek_offset(offset)?.node) == std::mem::discriminant(expected) {
            true
        } else {
            false
        })
    }

    pub fn parse_full(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.peek_is(&Token::EOF)? {
            // println!("Handling module statement: {:?}", self.peek()?);
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        // println!("Handling statement: {:?}", self.peek()?);
        match self.peek()?.node {
            Token::Let => self.parse_let(),
            Token::Act => self.parse_act(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Break => {
                self.consume(&Token::Break)?;

                self.consume_statement_end()?;

                Ok(Statement::Break)
            },
            Token::Continue => {
                self.consume(&Token::Continue)?;

                self.consume_statement_end()?;

                Ok(Statement::Continue)
            },
            Token::Identifier(_) => self.parse_expr_statement(),
            Token::Semicolon => { self.forward()?; self.parse_statement() },
            Token::Import => self.parse_import(),
            _ => {
                let stmt = Statement::Expr(self.parse_expr()?);
                self.consume_statement_end()?;
                Ok(stmt)
            },
        }
    }

    fn parse_import(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Import)?;

        let path = match &self.peek()?.node {
            Token::StringLiteral(val) => val.clone(),
            token => panic!("Expected string literal, got {:?}", token),
        };

        self.forward()?;

        let result = match &self.peek()?.node {
            Token::As => {
                self.forward()?;

                let result = match &self.peek()?.node {
                    Token::Identifier(alias) => Ok(Statement::Import {
                        path,
                        item: ImportItem::Alias(alias.clone())
                    }),
                    token => panic ! ("Expected identifier, got {:?}", token),
                };

                self.forward()?;

                result
            },
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
                                    },
                                    token => panic!("Expected identifier, got {:?}", token),
                                }
                            } else {
                                None
                            };
                            selective.push(ImportSymbol { original, alias })
                        }
                        Token::Asterisk => {
                            self.forward()?;
                            self.consume_statement_end()?;
                            return Ok(Statement::Import { path, item: ImportItem::All })
                        },
                        token => panic!("Expected identifier or `*`, got {:?}", token),
                    }

                    if !self.consume(&Token::Comma)? {
                        break;
                    }
                }

                Ok(Statement::Import { path, item: ImportItem::Selective(selective) })
            },
            token => panic!("Expected `as` or `get`, got {:?}", token),
        };

        self.consume_statement_end()?;

        result
    }

    fn parse_let(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Let)?;

        let name = if let Token::Identifier(name) = &self.peek()?.node {
            let name = name.clone();
            self.forward()?;
            name
        } else {
            return Err(format!("Expected identifier. Got {:?}", self.peek()?));
        };

        let value: Expr = if !self.consume(&Token::Equal)? {
            Expr::Null
        } else {
            self.parse_expr()?
        };

        self.consume_statement_end()?;

        Ok(Statement::Let { name, value })
    }

    fn parse_act(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Act)?;

        let name = match &self.peek()?.node {
            Token::Identifier(name) => name.clone(),
            _ => return Err(format!("Expected identifier. Got {:?}", self.peek()?)),
        };

        self.forward()?;

        if !self.consume(&Token::LParen)? {
            return Err(format!("Expected ')'. Got {:?}", self.peek()?));
        }

        let mut parameters = Vec::new();

        loop {
            match &self.peek()?.node {
                Token::Identifier(param) => {
                    parameters.push(param.clone());
                    self.forward()?;
                    if self.consume(&Token::Comma)? {
                        continue;
                    } else if self.consume(&Token::RParen)? {
                        break;
                    } else {
                        Err(format!("Founded not closed `(`. Unexpected token: {:?}", self.peek()?))?;
                    }
                },
                Token::RParen => {
                    self.forward()?;
                    break;
                }
                _ => Err(format!("Founded not closed `(`. Unexpected token: {:?}", self.peek()?))?,
            }
        }

        if !self.consume(&Token::LBrace)? {
            Err(format!("Founded not closed `{{`. Unexpected token: {:?}", self.peek()?))?;
        }

        let mut body = Vec::new();

        while !self.consume(&Token::RBrace)? {
            body.push(Box::new(self.parse_statement()?));
        }

        Ok(Statement::Act { name, parameters, body })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Return)?;

        let expr = self.parse_expr();

        self.consume_statement_end()?;

        Ok(Statement::Return(expr?))
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.consume(&Token::If)?;

        let condition = self.parse_expr()?;

        if !self.consume(&Token::LBrace)? {
            Err(format!("Founded not closed `{{`. Unexpected token: {:?}", self.peek()?))?;
        }

        let mut then_branch = Vec::new();

        while !self.consume(&Token::RBrace)? {
            then_branch.push(Box::new(self.parse_statement()?));
        }

        Ok(if self.consume(&Token::Else)? {
            let mut else_branch = Vec::new();

            if self.peek_is(&Token::If)? {
                else_branch.push(Box::new(self.parse_if()?));
            } else {
                if !self.consume(&Token::LBrace)? {
                    Err(format!("Founded not closed `{{`. Unexpected token: {:?}", self.peek()?))?;
                }

                while !self.consume(&Token::RBrace)? {
                    else_branch.push(Box::new(self.parse_statement()?));
                }
            }

            Statement::If { condition, then_branch, else_branch: Some(else_branch) }
        } else {
            Statement::If { condition, then_branch, else_branch: None }
        })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.consume(&Token::While)?;

        let condition = self.parse_expr()?;

        if !self.consume(&Token::LBrace)? {
            Err(format!("Founded not closed `{{`. Unexpected token: {:?}", self.peek()?))?;
        }

        let mut body = Vec::new();

        while !self.consume(&Token::RBrace)? {
            body.push(Box::new(self.parse_statement()?));
        }

        Ok(Statement::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Statement, String> {
        self.consume(&Token::For)?;

        match &self.peek()?.node {
            Token::Identifier(variable) => {
                let variable = variable.clone();

                self.forward()?;
                if !self.consume(&Token::In)? {
                    Err(format!("Expected key-word `in`. Got {:?}", self.peek()?))?;
                }

                let iterable = self.parse_expr()?;

                if !self.consume(&Token::LBrace)? {
                    Err(format!("Expected `{{`, not `{:?}`", self.peek()?))?;
                }

                let mut body = Vec::new();

                while !self.consume(&Token::RBrace)? {
                    body.push(Box::new(self.parse_statement()?));
                }

                Ok(Statement::For {
                    variable,
                    iterable,
                    body,
                })
            }
            _ => Err(format!("For value target token: {:?}", self.peek()?))?,
        }
    }

    fn parse_iterator(&mut self, start: Expr) -> Result<Expr, String> {
        let end = self.parse_expr()?;

        let step = if self.consume(&Token::DoubleColon)? {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(Expr::Iterator {
            start: Box::new(start),
            end: Box::new(end),
            step,
        })
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expr()?;

        let statement: Result<Statement, String> = match &expr {
            Expr::Variable(name) => {
                let token = self.peek()?.clone();
                match token.node {
                    Token::Equal => {
                        self.forward()?;
                        Ok(Statement::Assign {
                            name: name.clone(),
                            value: self.parse_expr()?,
                        })
                    },
                    Token::PlusEqual | Token::MinusEqual | Token::AsteriskEqual |
                    Token::SlashEqual | Token::PercentEqual => {
                        let operator = match token.node {
                            Token::PlusEqual => BinaryOperator::Add,
                            Token::MinusEqual => BinaryOperator::Sub,
                            Token::AsteriskEqual => BinaryOperator::Mul,
                            Token::SlashEqual => BinaryOperator::Div,
                            Token::PercentEqual => BinaryOperator::Mod,
                            _ => unreachable!()
                        };
                        self.forward()?;
                        Ok(Statement::Assign { name: name.clone(), value: Expr::BinaryOperation {
                            left_operand: Box::new(expr.clone()),
                            right_operand: Box::new(self.parse_expr()?),
                            operator,
                        }})
                    },
                    _ => Ok(Statement::Expr(expr.clone()))
                }
            }
            _ => Ok(Statement::Expr(expr.clone()))
        };

        // println!("{:?}", expr);

        self.consume_statement_end()?;

        Ok(statement?)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        Ok(self.parse_binary_expr(0)?)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let unary_operator = self.peek()?.node.to_unary_operator();

        if unary_operator.is_some() {
            self.forward()?;
            let operand = self.parse_expr()?;

            return Ok(Expr::UnaryOperation { operator: unary_operator.unwrap(), operand: Box::new(operand) })
        }

        let mut left = self.parse_primary()?;

        if self.peek_is(&Token::Dot)? && self.peek_offset_is(&Token::Dot, 1)? {
            self.forward()?; self.forward()?;
            let iter = self.parse_iterator(left.clone());

            if let Ok(iter) = iter {
                return Ok(iter)
            }
        }

        if self.peek_is(&Token::LBracket)? {
            self.forward()?;
            let expr = self.parse_primary()?;
            if !self.consume(&Token::RBracket)? {
                Err("Founded not closed slice brackets")?
            }
            return Ok(Expr::Slice {
                object: Box::new(left),
                index: Box::new(expr)
            })
        }

        while let Some(operator_token) = self.peek()?.node.to_binary_operator() {
            let precedence = operator_token.get_precedence();

            if precedence < min_precedence {
                break;
            }

            let binary_operator = operator_token;
            self.forward()?; // consume operator

            let right = self.parse_binary_expr(precedence + 1)?;

            left = Expr::BinaryOperation {
                left_operand: Box::new(left),
                operator: binary_operator,
                right_operand: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        self.consume(&Token::LParen)?;

        let mut arguments = Vec::new();

        if self.consume(&Token::RParen)? {
            return Ok(arguments); // пустые аргументы ()
        }

        loop {
            let mut expr = self.parse_expr()?;
            expr = self.parse_postfix(expr)?;
            arguments.push(expr);

            if self.consume(&Token::Comma)? {
                continue;
            } else if self.consume(&Token::RParen)? {
                break;
            } else {
                Err(format!("Founded not closed `(`, unexpected: {:?}", self.peek()?))?;
            }
        }

        Ok(arguments)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            match &self.peek()?.node {
                Token::Dot => {
                    self.forward()?;

                    if let Token::Identifier(field) = &self.peek()?.node {
                        expr = Expr::GetAttr {
                            object: Box::new(expr),
                            field: field.clone()
                        };
                        self.forward()?;
                    } else {
                        Err(format!("Expected identifier after `.`, founded: {:?}", self.peek()?))?
                    }
                },
                Token::LParen => {
                    self.forward()?;

                    let args = self.parse_arguments()?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        arguments: args
                    };
                },
                _ => return Ok(expr)
            }
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match &self.peek()?.node {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.forward()?;
                Ok(Expr::String(s.to_string()))
            }
            Token::IntegerLiteral(i) => {
                let i = i.clone();
                self.forward()?;
                Ok(Expr::Integer(i))
            }
            Token::FloatLiteral(f) => {
                let f = f.clone();
                self.forward()?;
                Ok(Expr::Float(f))
            }
            Token::FString(parts) => {
                let mut format_string = Vec::new();

                for part in parts {
                    match part.clone() {
                        FStringPart::StringLiteral(string) => {
                            format_string.push(FStringExpr::String(string.clone()));
                        },
                        FStringPart::Expr(mut expr, .. ) => {
                            expr.push(SpannedToken::new(Token::EOF, Span::new_point(0, 0)));
                            println!("{:?}", expr);
                            format_string.push(FStringExpr::Expr(ParserProcess::new(expr).parse_expr()?));
                        }
                    }
                }

                self.forward()?;

                Ok(Expr::FString(format_string))
            }
            Token::True => {
                self.forward()?;
                Ok(Expr::Boolean(true))
            }
            Token::False => {
                self.forward()?;
                Ok(Expr::Boolean(false))
            }
            Token::Null => {
                self.forward()?;
                Ok(Expr::Null)
            }
            Token::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.forward()?;
                Ok(self.parse_postfix(Expr::Variable(identifier))?)
            }
            Token::LParen => {
                self.forward()?;

                let expr = self.parse_expr()?;

                if !self.consume(&Token::RParen)? {
                    Err(format!("Founded not closed `(`. Unexpected token: {:?}", self.peek()?))?
                }

                Ok(expr)
            }
            Token::LBracket => {
                self.forward()?;

                let mut list = Vec::new();

                while !self.consume(&Token::RBracket)? {
                    list.push(Box::new(self.parse_expr()?));

                    self.consume(&Token::Comma)?;
                }

                Ok(Expr::List(list))
            }
            token => Err(format!("Unexpected token: {:?}", token))?
        }
    }
}