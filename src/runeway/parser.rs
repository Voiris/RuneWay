use crate::runeway::ast_structure::{Module, Statement, Expr};
use crate::runeway::lexer::Token;

struct ParserProcess {
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
        if std::mem::discriminant(self.peek().unwrap()) == std::mem::discriminant(expected)  {
            self.forward();
            true
        } else {
            false
        }
    }

    pub fn parse_module(&mut self) -> Module {
        let mut statements: Vec<Statement> = Vec::new();

        while !matches!(self.peek(), Some(&Token::EOF)) {
            // println!("Handling module statement: {:?}", self.peek());
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                panic!("Cannot parse incorrect statement")
            }
        }

        Module { statements }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.peek().unwrap() {
            Token::Let => self.parse_let(),
            Token::Act => self.parse_act(),
            Token::Return => self.parse_return(),
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
        println!("{}", name);

        self.forward();

        if !self.consume(&Token::LParen) {
            return None;
        }

        let mut parameters = Vec::new();

        loop {
            match self.peek().unwrap() {
                Token::Identifier(param) => {
                    println!("{}", param);
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

        println!("{:?}", parameters);

        if !self.consume(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();

        while !self.consume(&Token::RBrace) {
            body.push(Box::new(self.parse_statement()?));
        }

        println!("body: {:?}", body);

        if !self.consume(&Token::Semicolon) {
            return None;
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

    fn parse_expr_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expr()?;

        match &expr {
            Expr::Variable(name) => {
                if self.consume(&Token::Equal) {
                    let assignment = self.parse_expr()?;

                    if !self.consume(&Token::Semicolon) {
                        return None;
                    }

                    return Some(Statement::Assign { name: name.clone(), value: assignment });
                }
            }
            _ => {}
        }

        if !self.consume(&Token::Semicolon) {
            return None;
        }
        Some(Statement::Expr(expr.clone()))
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Option<Expr> {
        let mut left = self.parse_primary().unwrap();

        while let Some(operator_token) = self.peek().unwrap().to_binary_operator() {
            let precedence = operator_token.get_precedence();

            if precedence < min_precedence {
                break;
            }

            let operator = operator_token;
            self.forward(); // consume operator

            let right = self.parse_binary_expr(precedence + 1)?;

            left = Expr::BinaryOperation {
                left_operand: Box::new(left),
                operator,
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
            Token::Null => {
                self.forward();
                Some(Expr::Null)
            },
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
            _ => None
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Module {
    let mut process = ParserProcess::new(tokens);
    process.parse_module()
}