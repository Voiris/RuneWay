use super::super::lexer::token::Token;
use super::super::ast::statement::Statement;
use super::super::ast::expression::{Expr, FStringExpr};

pub struct ParserProcess {
    tokens: Vec<Token>,
    pos: usize,
}

impl ParserProcess {
    pub fn new(tokens: Vec<Token>) -> ParserProcess {
        ParserProcess {
            tokens,
            pos: 0
        }
    }

    fn peek_offset(&self, offset: isize) -> Option<&Token> {
        self.tokens.get(self.pos.checked_add_signed(offset)?)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn forward(&mut self) -> Option<&Token> {
        self.pos += 1;
        self.peek()
    }

    fn backward(&mut self) -> Option<&Token> {
        self.pos -= 1;
        self.peek()
    }

    fn consume(&mut self, expected: &Token) -> bool {
        if self.peek_is(expected)  {
            self.forward();
            true
        } else {
            false
        }
    }

    fn peek_is(&mut self, expected: &Token) -> bool {
        if std::mem::discriminant(self.peek().unwrap()) == std::mem::discriminant(expected) {
            true
        } else {
            false
        }
    }

    fn peek_offset_is(&mut self, expected: &Token, offset: isize) -> bool {
        if std::mem::discriminant(self.peek_offset(offset).unwrap()) == std::mem::discriminant(expected) {
            true
        } else {
            false
        }
    }

    /*
    pub fn parse_module(&mut self) -> Module {
        let mut statements: Vec<Statement> = Vec::new();

        while !matches!(self.peek(), Some(&Token::EOF)) {
            println!("Handling module statement: {:?}", self.peek().unwrap());
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                panic!("Cannot parse incorrect statement")
            }
        }

        Module { statements }
    }
     */
    pub fn parse_full(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();

        while !matches!(self.peek(), Some(&Token::EOF)) {
            println!("Handling module statement: {:?}", self.peek().unwrap());
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                panic!("Cannot parse incorrect statement")
            }
        }

        statements
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        println!("Handling statement: {:?}", self.peek().unwrap());
        match self.peek().unwrap() {
            Token::Let => self.parse_let(),
            Token::Act => self.parse_act(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Break => {
                self.consume(&Token::Break);
                self.consume(&Token::Semicolon);

                Some(Statement::Break)
            },
            Token::Identifier(_) => self.parse_expr_statement(),
            _ => None
        }
    }

    fn parse_let(&mut self) -> Option<Statement> {
        self.consume(&Token::Let);

        let name = if let Token::Identifier(name) = self.peek().unwrap() {
            let name = name.clone();
            self.forward();
            name
        } else {
            return None;
        };

        let value: Expr = if !self.consume(&Token::Equal) {
            Expr::Null
        } else {
            self.parse_expr()?
        };

        if !self.consume(&Token::Semicolon) {
            return None;
        }

        Some(Statement::Let { name, value })
    }

    fn parse_act(&mut self) -> Option<Statement> {
        self.consume(&Token::Act);

        let name = match self.peek().unwrap() {
            Token::Identifier(name) => name.clone(),
            _ => return None,
        };

        self.forward();

        if !self.consume(&Token::LParen) {
            return None;
        }

        let mut parameters = Vec::new();

        loop {
            match self.peek().unwrap() {
                Token::Identifier(param) => {
                    parameters.push(param.clone());
                    self.forward();
                    if self.consume(&Token::Comma) {
                        continue;
                    } else if self.consume(&Token::RParen) {
                        break;
                    } else {
                        return None;
                    }
                },
                Token::RParen => {
                    self.forward();
                    break;
                }
                _ => return None
            }
        }

        if !self.consume(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();

        while !self.consume(&Token::RBrace) {
            body.push(Box::new(self.parse_statement()?));
        }

        Some(Statement::Act { name, parameters, body })
    }

    fn parse_return(&mut self) -> Option<Statement> {
        self.consume(&Token::Return);

        let expr = self.parse_expr();

        if !self.consume(&Token::Semicolon) {
            return None;
        }

        Some(Statement::Return(expr?))
    }

    fn parse_if(&mut self) -> Option<Statement> {
        self.consume(&Token::If);

        let condition = self.parse_expr()?;

        println!("condition: {:?}", condition);

        if !self.consume(&Token::LBrace) {
            return None;
        }

        let mut then_branch = Vec::new();

        while !self.consume(&Token::RBrace) {
            then_branch.push(Box::new(self.parse_statement()?));
        }

        println!("then_branch: {:?}", then_branch);

        let statement = if self.consume(&Token::Else) {
            let mut else_branch = Vec::new();

            if self.peek() == Some(&Token::If) {
                else_branch.push(Box::new(self.parse_if()?));
            } else {
                if !self.consume(&Token::LBrace) {
                    return None;
                }

                while !self.consume(&Token::RBrace) {
                    else_branch.push(Box::new(self.parse_statement()?));
                }
            }

            println!("else_branch: {:?}", else_branch);

            Some(Statement::If { condition, then_branch, else_branch: Some(else_branch) } )
        } else {
            Some(Statement::If { condition, then_branch, else_branch: None })
        };

        statement
    }

    fn parse_while(&mut self) -> Option<Statement> {
        self.consume(&Token::While);

        let condition = self.parse_expr()?;

        println!("condition: {:?}", condition);

        if !self.consume(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();

        while !self.consume(&Token::RBrace) {
            body.push(Box::new(self.parse_statement()?));
        }

        println!("body: {:?}", body);
        Some(Statement::While { condition, body })
    }

    fn parse_for(&mut self) -> Option<Statement> {
        self.consume(&Token::For);

        match self.peek().unwrap() {
            Token::Identifier(variable) => {
                let variable = variable.clone();

                println!("variable: {:?}", variable);

                self.forward();
                if !self.consume(&Token::In) {
                    return None
                }

                let iterable = self.parse_expr()?;

                println!("iterable: {:?}", iterable);

                if !self.consume(&Token::LBrace) {
                    return None
                }

                let mut body = Vec::new();

                while !self.consume(&Token::RBrace) {
                    body.push(Box::new(self.parse_statement()?));
                }

                println!("body: {:?}", body);

                Some(Statement::For {
                    variable,
                    iterable,
                    body,
                })
            }
            _ => None,
        }
    }

    fn parse_iterator(&mut self, start: Expr) -> Option<Expr> {
        let end = self.parse_expr()?;

        let step = if self.consume(&Token::DoubleColon) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Some(Expr::Iterator {
            start: Box::new(start),
            end: Box::new(end),
            step,
        })
    }

    fn parse_expr_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expr()?;

        let statement: Option<Statement> = match &expr {
            Expr::Variable(name) => {
                if self.consume(&Token::Equal) {
                    let assignment = self.parse_expr()?;

                    Some(Statement::Assign { name: name.clone(), value: assignment })
                } else {
                    Some(Statement::Expr(expr.clone()))
                }
            }
            _ => Some(Statement::Expr(expr.clone()))
        };

        if !self.consume(&Token::Semicolon) {
            return None;
        }

        statement
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Option<Expr> {
        let unary_operator = self.peek().unwrap().to_unary_operator();

        if unary_operator.is_some() {
            self.forward();
            let operand = self.parse_expr()?;

            return Some(Expr::UnaryOperation { operator: unary_operator.unwrap(), operand: Box::new(operand) })
        }

        let mut left = self.parse_primary().unwrap();

        if self.peek_is(&Token::Dot) && self.peek_offset_is(&Token::Dot, 1) {
            self.forward(); self.forward();
            let iter = self.parse_iterator(left.clone());

            if let Some(iter) = iter {
                return Some(iter)
            }
        }

        while let Some(operator_token) = self.peek().unwrap().to_binary_operator() {
            let precedence = operator_token.get_precedence();

            if precedence < min_precedence {
                break;
            }

            let binary_operator = operator_token;
            self.forward(); // consume operator

            let right = self.parse_binary_expr(precedence + 1)?;

            left = Expr::BinaryOperation {
                left_operand: Box::new(left),
                operator: binary_operator,
                right_operand: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek().unwrap() {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.forward();
                Some(Expr::String(s.to_string()))
            }
            Token::IntegerLiteral(i) => {
                let i = i.clone();
                self.forward();
                Some(Expr::Integer(i))
            }
            Token::FloatLiteral(f) => {
                let f = f.clone();
                self.forward();
                Some(Expr::Float(f))
            }
            Token::FStringStart => {
                self.forward();
                let mut format_string = Vec::new();

                while !self.consume(&Token::FStringEnd) {
                    match self.peek().unwrap() {
                        Token::FStringLiteral(s) => {
                            format_string.push(FStringExpr::String(s.clone()));
                            self.forward();
                        },
                        Token::FStringExprStart => {
                            self.forward();
                            format_string.push(FStringExpr::Expr(self.parse_expr()?));
                            if !self.consume(&Token::FStringExprEnd) {
                                panic!("Unclosed format string expression or multiple expressions");
                            }
                        }
                        _ => println!("Invalid format string: {:?}", self.peek().unwrap())
                    }
                }

                println!("format_string: {:?}", format_string);

                Some(Expr::FString(format_string))
            }
            Token::True => {
                self.forward();
                Some(Expr::Boolean(true))
            }
            Token::False => {
                self.forward();
                Some(Expr::Boolean(false))
            }
            Token::Null => {
                self.forward();
                Some(Expr::Null)
            }
            Token::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.forward();
                if self.consume(&Token::LParen) {
                    let mut arguments: Vec<Expr> = Vec::new();

                    while !self.consume(&Token::RParen) {
                        if let Some(arg) = self.parse_expr() {
                            arguments.push(arg);

                            self.consume(&Token::Comma);
                        }
                    }

                    Some(Expr::Call {
                        act: identifier,
                        arguments,
                    })
                } else {
                    Some(Expr::Variable(identifier.clone()))
                }
            }
            Token::LParen => {
                self.forward();

                let expr = self.parse_expr();

                if !self.consume(&Token::RParen) {
                    return None;
                }

                expr
            }
            Token::LBracket => {
                self.forward();

                let mut list = Vec::new();

                while !self.consume(&Token::RBracket) {
                    if let Some(expr) = self.parse_expr() {
                        list.push(Box::new(expr));

                        self.consume(&Token::Comma);
                    }
                }

                Some(Expr::List(list))
            }
            token => {
                println!("Unexpected token: {:?}", token);
                None
            },
        }
    }
}