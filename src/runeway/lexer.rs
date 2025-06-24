use crate::runeway::ast_structure::{BinaryOperator, UnaryOperator};
use crate::runeway::lexer::Token::EOF;


#[derive(PartialEq, Debug)]
pub enum Token {
    // Code structure
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),

    // FStrings
    FStringStart,                       // f"
    FStringLiteral(String),
    FStringExprStart,                   // {
    FStringExprSubcommand(Vec<Token>),  // :TOKENS}
    FStringExprEnd,                     // }
    FStringEnd,                         // "

    // Keywords
    Let,
    In,
    Null,

    // Classes
    Class,
    Property,
    Set,
    Get,

    // Functions
    Act,
    Return,

    // Logic
    If,
    Else,

    And,
    Or,
    Not,
    Bang, // !

    True,
    False,

    // Loops
    For,
    While,
    Break,
    Continue,

    // Equalising and comparison
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=

    // Mathematics
    Plus,           // +
    Minus,          // -
    Asterisk,       // *
    DoubleAsterisk, // **
    Slash,          // /
    // TildeSlash,     // ~/
    Percent,        // %

    // Compound assignment operators
    PlusEqual,           // +=
    MinusEqual,          // -=
    AsteriskEqual,       // *=
    DoubleAsteriskEqual, // **=
    SlashEqual,          // /=
    PercentEqual,        // %=

    // Arrows
    Arrow,        // ->
    DoubleArrow,  // =>

    // Brackets
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]

    // Other
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    DoubleColon,  // ::
    Semicolon,    // ;
    AtSymbol,     // @

    // Comments
    // DoubleSlash, // //
    // SlashStar,   // /*
    // StarSlash,   // */

    // Quotes
    // Quote,             // '
    // TripleQuote,       // '''
    // DoubleQuote,       // "
    // TripleDoubleQuote, // """

    // Special
    EOF
}

impl Token {
    pub fn to_binary_operator(&self) -> Option<BinaryOperator> {
        match self {
            Token::Plus => Some(BinaryOperator::Add),
            Token::Minus => Some(BinaryOperator::Sub),
            Token::Asterisk => Some(BinaryOperator::Mul),
            Token::DoubleAsterisk => Some(BinaryOperator::Pow),
            Token::Slash => Some(BinaryOperator::Div),
            Token::Percent => Some(BinaryOperator::Mod),

            Token::EqualEqual => Some(BinaryOperator::Eq),
            Token::NotEqual => Some(BinaryOperator::NotEq),
            Token::Greater => Some(BinaryOperator::Gt),
            Token::GreaterEqual => Some(BinaryOperator::GtEq),
            Token::Less => Some(BinaryOperator::Lt),
            Token::LessEqual => Some(BinaryOperator::LtEq),

            Token::And => Some(BinaryOperator::And),
            Token::Or => Some(BinaryOperator::Or),

            _ => None,
        }
    }

    pub fn to_unary_operator(&self) -> Option<UnaryOperator> {
        match self {
            Token::Not | Token::Bang => Some(UnaryOperator::Not),
            Token::Minus => Some(UnaryOperator::Neg),
            _ => None,
        }
    }
}

struct LexerProcess {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl LexerProcess {
    pub fn new(input: String) -> LexerProcess {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn forward(&mut self) -> Option<&char> {
        self.pos += 1;
        self.peek()
    }

    fn backward(&mut self) -> Option<&char> {
        self.pos -= 1;
        self.peek()
    }

    fn peek_offset(&self, offset: isize) -> Option<&char> {
        self.chars.get(self.pos.checked_add_signed(offset)?)
    }
    
    fn peek(&self) -> Option<&char> {
        self.chars.get(self.pos)
    }

    fn peek_is(&self, c: char) -> bool {
        self.peek() == Some(&c)
    }

    fn peek_offset_is(&self, c: char, offset: isize) -> bool {
        self.peek_offset(offset) == Some(&c)
    }

    fn has_forward(&self) -> bool {
        self.pos + 1 < self.chars.len()
    }

    // fn extract_substring(&self, start_offset: isize, end_offset: isize) -> Option<&str> {
    //     self.chars[self.pos.checked_add_signed(start_offset)?..=self.pos.checked_add_signed(end_offset)?].iter().collect()
    // }

    fn lex_string_literal(&mut self, is_format_string: bool) -> Token {
        let mut value = String::new();
        let mut terminated = false;

        if let Some(&quote) = self.peek() {
            if is_format_string {
                value.push(quote)
            }

            self.forward();

            while let Some(&char) = self.peek() {
                if !is_format_string && quote == char {
                    terminated = true;
                    self.forward();
                    break
                } else if is_format_string && (char == '{' || char == '"') {
                    terminated = true;
                    break;
                } else {
                    value.push(char);
                    self.forward();
                }
            }
        } else {
            panic!("Founded unexpected character")
        };

        if !terminated {
            panic!("Founded unterminated string")
        }

        Token::StringLiteral(value)
    }

    //noinspection DuplicatedCode
    fn lex_number_literal(&mut self, is_negative: bool) -> Token {
        let mut string_number = String::new();
        let mut is_float = false;

        if is_negative {
            string_number.push('-')
        }

        // Number before dot or integer
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                string_number.push(c);
                self.forward();
            } else {
                break;
            }
        }

        // Dot and float part
        if self.peek() == Some(&'.') {
            if let Some(&next_after_dot) = self.peek_offset(1) {
                if next_after_dot.is_ascii_digit() {
                    is_float = true;
                    string_number.push('.');
                    self.forward();

                    while let Some(&c) = self.peek() {
                        if c.is_ascii_digit() {
                            string_number.push(c);
                            self.forward();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Exponent: e+10, E-5, e3
        if let Some(&c) = self.peek() {
            if c == 'e' || c == 'E' {
                is_float = true;
                string_number.push(c);
                self.forward();

                if let Some(&sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        string_number.push(sign);
                        self.forward();
                    }
                }

                // Number after exponent
                let mut has_digits = false;
                while let Some(&c) = self.peek() {
                    if c.is_ascii_digit() {
                        string_number.push(c);
                        self.forward();
                        has_digits = true;
                    } else {
                        break;
                    }
                }

                if !has_digits {
                    panic!("Founded invalid float exponent")
                }
            }
        }

        if is_float {
            Token::FloatLiteral(string_number.parse::<f64>().unwrap())
        } else {
            Token::IntegerLiteral(string_number.parse::<i64>().unwrap())
        }
    }

    fn lex_ident_and_keyword(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.forward();
            } else {
                break;
            }
        }

        let token = match ident.as_str() {
            "let" => Token::Let,
            "act" => Token::Act,
            "return" => Token::Return,
            "class" => Token::Class,
            "null" => Token::Null,
            "true" => Token::True,
            "false" => Token::False,
            "and" => Token::And,
            "not" => Token::Not,
            "or" => Token::Or,
            "in" => Token::In,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "continue" => Token::Continue,
            "break" => Token::Break,

            _ => Token::Identifier(ident)
        };

        token
    }

    fn lex_format_string(&mut self) -> Vec<Token> {
        self.forward(); self.forward();

        let mut tokens = vec![Token::FStringStart];

        while !self.peek_is('"') {
            if self.peek_is('{') {
                self.forward();
                tokens.push(Token::FStringExprStart);
                while !self.peek_is('}') {
                    if self.peek_is(':') {
                        self.forward();
                        let mut subcommand = Vec::new();
                        while !self.peek_is('}') {
                            subcommand.extend(self.lex_primary());
                        }
                        tokens.push(Token::FStringExprSubcommand(subcommand));
                    } else {
                        tokens.extend(self.lex_primary());
                    }
                }

                self.forward();
                tokens.push(Token::FStringExprEnd);
            } else {
                let Token::StringLiteral(string) = self.lex_string_literal(true) else {
                    panic!("Expected string literal")
                };

                tokens.push(Token::FStringLiteral(string));
            }
        }
        tokens.push(Token::FStringEnd);
        self.forward();

        tokens
    }

    fn lex_primary(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        match self.peek().unwrap() {
            '=' => {
                self.forward();
                match self.peek().unwrap() {
                    '>' => {
                        self.forward();
                        tokens.push(Token::DoubleArrow);
                    }
                    '=' => {
                        self.forward();
                        tokens.push(Token::EqualEqual);
                    }

                    _ => tokens.push(Token::Equal),
                }
            }
            '>' => {
                self.forward();
                match self.peek().unwrap() {
                    '=' => {
                        self.forward();
                        tokens.push(Token::GreaterEqual);
                    }

                    _ => tokens.push(Token::Greater),
                }
            }
            '<' => {
                self.forward();
                match self.peek().unwrap() {
                    '=' => {
                        self.forward();
                        tokens.push(Token::LessEqual);
                    }

                    _ => tokens.push(Token::Less),
                }
            }
            '-' => {
                self.forward();
                match self.peek().unwrap() {
                    '>' => {
                        self.forward();
                        tokens.push(Token::Arrow)
                    }
                    '=' => {
                        self.forward();
                        tokens.push(Token::MinusEqual)
                    }
                    '0'..='9' => {
                        tokens.push(self.lex_number_literal(true))
                    }

                    _ => tokens.push(Token::Minus)
                }
            }
            '+' => {
                self.forward();
                match self.peek().unwrap() {
                    '=' => {
                        self.forward();
                        tokens.push(Token::PlusEqual)
                    }

                    _ => tokens.push(Token::Plus)
                }
            }
            '*' => {
                self.forward();
                match self.peek().unwrap() {
                    '*' => {
                        self.forward();
                        match self.peek().unwrap() {
                            '=' => {
                                self.forward();
                                tokens.push(Token::DoubleAsteriskEqual)
                            }

                            _ => tokens.push(Token::DoubleAsterisk)
                        }
                    }
                    '=' => {
                        self.forward();
                        tokens.push(Token::AsteriskEqual)
                    }

                    _ => tokens.push(Token::Asterisk)
                }
            }
            '/' => {
                self.forward();
                match self.peek().unwrap() {
                    '/' => {
                        while self.has_forward() {
                            self.forward();
                            if self.peek_is('\n') {
                                break;
                            }
                        }
                    }
                    '*' => {
                        while self.has_forward() {
                            self.forward();
                            if self.peek_is('*') && self.peek_offset_is('/', 1) {
                                self.forward(); self.forward();
                                break;
                            }
                        }
                    }
                    '=' => {
                        self.forward();
                        tokens.push(Token::SlashEqual)
                    }

                    _ => tokens.push(Token::Slash)
                }
            }
            '.' => {
                self.forward();
                tokens.push(Token::Dot);
            },
            '0'..='9' => {
                tokens.push(self.lex_number_literal(false))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                if self.peek_offset_is('"', 1) {
                    tokens.extend(self.lex_format_string())
                } else {
                    tokens.push(self.lex_ident_and_keyword())
                }
            }
            '[' => {
                tokens.push(Token::LBracket);
                self.forward();
            },
            ']' => {
                tokens.push(Token::RBracket);
                self.forward();
            },
            '!' => {
                self.forward();
                if self.peek_is('=') {
                    self.forward();
                    tokens.push(Token::NotEqual);
                } else {
                    tokens.push(Token::Bang);
                }
            },
            '%' => {
                tokens.push(Token::Percent);
                self.forward();
            },
            // Code structure
            '"' => {
                tokens.push(self.lex_string_literal(false));
            }
            _ => panic!("Lexer founded unexpected character: {}", self.peek().unwrap().to_string())
        }

        tokens
    }

    //noinspection DuplicatedCode
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(&char) = self.peek() {
            match char {
                // Space and escape sequence
                ' ' | '\t' | '\r' => {
                    self.forward();
                }
                '\n' => {
                    self.line += 1;
                    self.forward();
                }

                // Multiple char tokens
                ':' => {
                    self.forward();
                    if self.peek_is(':') {
                        self.forward();
                        tokens.push(Token::DoubleColon);
                    } else {
                        tokens.push(Token::Colon);
                    }
                }

                // Single char tokens
                '(' => {
                    tokens.push(Token::LParen);
                    self.forward();
                },
                ')' => {
                    tokens.push(Token::RParen);
                    self.forward();
                },
                '{' => {
                    tokens.push(Token::LBrace);
                    self.forward();
                },
                '}' => {
                    tokens.push(Token::RBrace);
                    self.forward();
                },
                ',' => {
                    tokens.push(Token::Comma);
                    self.forward();
                },
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.forward();
                },
                '@' => {
                    tokens.push(Token::AtSymbol);
                    self.forward();
                },

                // UnexpectedCharacter
                _ => {
                    tokens.extend(self.lex_primary());
                }
            }
        }

        tokens.push(EOF);

        tokens
    }
}

pub fn tokenize(input: String) -> Vec<Token> {
    LexerProcess::new(input).tokenize()
}
