use super::token::{FStringPart, SpannedToken, Token};
use std::ops::Range;

pub struct LexerProcess {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl LexerProcess {
    pub fn new(input: String) -> LexerProcess {
        Self {
            chars: input.clone().chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn forward(&mut self) -> Option<char> {
        self.pos += 1;
        self.column += 1;
        let peek = self.peek().cloned();
        if peek == Some('\n') {
            self.line += 1;
            self.column = 0;
        }
        peek
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

    fn current_point(&self) -> Range<usize> {
        self.pos..(self.pos + 1)
    }

    // fn extract_substring(&self, start_offset: isize, end_offset: isize) -> Option<&str> {
    //     self.chars[self.pos.checked_add_signed(start_offset)?..=self.pos.checked_add_signed(end_offset)?].iter().collect()
    // }

    fn lex_string_literal(&mut self, is_format_string: bool, is_raw: bool) -> SpannedToken {
        let start = self.pos;
        let mut end = 0;
        let mut value = String::new();
        let mut terminated = false;

        if let Some(&quote) = self.peek() {
            if !is_format_string {
                self.forward();
            }

            while let Some(&char) = self.peek() {
                if !is_format_string && quote == char {
                    terminated = true;
                    end = self.pos;
                    self.forward();
                    break;
                } else if is_format_string && (char == '{' || char == '"') {
                    terminated = true;
                    end = self.pos;
                    break;
                } else {
                    if char == '\\' && !is_raw {
                        self.forward();
                        if let Some(c) = self.peek() {
                            match c {
                                'n' => value.push('\n'),
                                'r' => value.push('\r'),
                                't' => value.push('\t'),
                                '\\' => value.push('\\'),
                                '\'' => value.push('\''),
                                '\"' => value.push('\"'),
                                '0' => value.push('\0'),
                                // UNICODE \uXXXX
                                'u' | 'U' => {
                                    let escape_type = c.clone();
                                    let hex_len = if escape_type == 'u' { 4 } else { 8 };

                                    let mut hex = String::new();
                                    for _ in 0..hex_len {
                                        self.forward();
                                        match self.peek() {
                                            Some(c) if c.is_ascii_hexdigit() => hex.push(*c),
                                            Some(c) => panic!(
                                                "Invalid Unicode escape hex digit in \\{} escape: '{}'",
                                                escape_type, c
                                            ),
                                            None => panic!(
                                                "Unexpected end of input in \\{} escape",
                                                escape_type
                                            ),
                                        }
                                    }

                                    let code_point =
                                        u32::from_str_radix(&hex, 16).unwrap_or_else(|_| {
                                            panic!(
                                                "Failed to parse \\{}{} unicode escape",
                                                escape_type, hex
                                            )
                                        });

                                    if let Some(ch) = std::char::from_u32(code_point) {
                                        value.push(ch);
                                    } else {
                                        panic!(
                                            "Invalid Unicode code point: \\{}{}",
                                            escape_type, hex
                                        )
                                    }
                                }
                                other => {
                                    // Panic is temporary
                                    panic!("Invalid escape sequence '\\{}'", other)
                                }
                            }
                        }
                    } else {
                        value.push(char);
                    }
                    self.forward();
                }
            }
        } else {
            panic!("Founded unexpected character")
        }

        if !terminated {
            panic!("Founded unterminated string")
        }

        SpannedToken::new(Token::StringLiteral(value), start..end)
    }

    //noinspection DuplicatedCode
    fn lex_number_literal(&mut self, is_negative: bool) -> SpannedToken {
        let start = self.pos;

        let mut string_number = String::new();
        let mut is_float = false;
        let mut suffix: Option<char> = None;

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

        if let Some(&c) = self.peek() {
            if matches!(c, 'i' | 'u' | 'f') {
                suffix = Some(c);
                self.forward();
            }
        }

        // Exponent: e+10, E-5, e3
        if suffix.is_none() {
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
        }

        let node = if let Some(suffix) = suffix {
            match suffix {
                'i' => Token::IntegerLiteral(string_number.parse::<i64>().unwrap()),
                'f' => Token::FloatLiteral(string_number.parse::<f64>().unwrap()),
                'u' => Token::UIntegerLiteral(string_number.parse::<u64>().unwrap()),
                _ => unreachable!(),
            }
        } else {
            if is_float {
                Token::FloatLiteral(string_number.parse::<f64>().unwrap())
            } else {
                Token::IntegerLiteral(string_number.parse::<i64>().unwrap())
            }
        };

        SpannedToken::new(node, start..self.pos)
    }

    fn lex_ident_and_keyword(&mut self) -> SpannedToken {
        let start = self.pos;

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
            "import" => Token::Import,
            "get" => Token::Get,
            "as" => Token::As,
            "assert" => Token::Assert,

            _ => Token::Identifier(ident),
        };

        SpannedToken::new(token, start..self.pos)
    }

    fn lex_format_string(&mut self, is_raw: bool) -> SpannedToken {
        let start = self.pos;

        if is_raw {
            self.forward();
        }
        self.forward();
        self.forward();

        let mut parts = vec![];

        while !self.peek_is('"') {
            if self.peek_is('{') {
                self.forward();
                let mut expr = Vec::new();
                let mut subcommand = Vec::new();
                while !self.peek_is('}') {
                    if self.peek_is(':') {
                        self.forward();
                        while !self.peek_is('}') {
                            subcommand.extend(self.lex_primary());
                        }
                    } else {
                        expr.extend(self.lex_primary());
                    }
                }
                self.forward();
                parts.push(FStringPart::Expr(expr, subcommand));
            } else {
                let Token::StringLiteral(string) = self.lex_string_literal(true, is_raw).node
                else {
                    panic!("Expected string literal")
                };

                parts.push(FStringPart::StringLiteral(string));
            }
        }
        let end = self.pos;
        self.forward();

        SpannedToken::new(Token::FString(parts), start..end)
    }

    //noinspection DuplicatedCode
    fn lex_primary(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        match self.peek().unwrap() {
            // Space and escape sequence
            ' ' | '\t' | '\r' => {
                self.forward();
            }
            '\n' => {
                self.forward();
            }
            // Multiple char tokens
            ':' => {
                let start = self.pos;
                self.forward();
                if self.peek_is(':') {
                    tokens.push(SpannedToken::new(Token::DoubleColon, start..self.pos));
                    self.forward();
                } else {
                    tokens.push(SpannedToken::new(Token::Colon, start..start));
                }
            }

            // Single char tokens
            '(' => {
                tokens.push(SpannedToken::new(Token::LParen, self.current_point()));
                self.forward();
            }
            ')' => {
                tokens.push(SpannedToken::new(Token::RParen, self.current_point()));
                self.forward();
            }
            '{' => {
                tokens.push(SpannedToken::new(Token::LBrace, self.current_point()));
                self.forward();
            }
            '}' => {
                tokens.push(SpannedToken::new(Token::RBrace, self.current_point()));
                self.forward();
            }
            ',' => {
                tokens.push(SpannedToken::new(Token::Comma, self.current_point()));
                self.forward();
            }
            ';' => {
                tokens.push(SpannedToken::new(Token::Semicolon, self.current_point()));
                self.forward();
            }
            '@' => {
                tokens.push(SpannedToken::new(Token::AtSymbol, self.current_point()));
                self.forward();
            }
            // Other
            '=' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '>' => {
                        tokens.push(SpannedToken::new(Token::DoubleArrow, start..self.pos));
                        self.forward();
                    }
                    '=' => {
                        tokens.push(SpannedToken::new(Token::EqualEqual, start..self.pos));
                        self.forward();
                    }

                    _ => tokens.push(SpannedToken::new(Token::Equal, start..start)),
                }
            }
            '>' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '=' => {
                        tokens.push(SpannedToken::new(Token::GreaterEqual, start..self.pos));
                        self.forward();
                    }

                    _ => tokens.push(SpannedToken::new(Token::Greater, start..start)),
                }
            }
            '<' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '=' => {
                        tokens.push(SpannedToken::new(Token::LessEqual, start..self.pos));
                        self.forward();
                    }

                    _ => tokens.push(SpannedToken::new(Token::Less, start..start)),
                }
            }
            '-' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '>' => {
                        tokens.push(SpannedToken::new(Token::Arrow, start..self.pos));
                        self.forward();
                    }
                    '=' => {
                        tokens.push(SpannedToken::new(Token::MinusEqual, start..self.pos));
                        self.forward();
                    }
                    '0'..='9' => tokens.push(self.lex_number_literal(true)),

                    _ => tokens.push(SpannedToken::new(Token::Minus, start..start)),
                }
            }
            '+' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '=' => {
                        tokens.push(SpannedToken::new(Token::PlusEqual, start..self.pos));
                        self.forward();
                    }

                    _ => tokens.push(SpannedToken::new(Token::Plus, start..start)),
                }
            }
            '*' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '*' => {
                        let next = self.pos;
                        match self.forward().unwrap() {
                            '=' => {
                                tokens.push(SpannedToken::new(
                                    Token::DoubleAsteriskEqual,
                                    start..self.pos,
                                ));
                                self.forward();
                            }

                            _ => tokens.push(SpannedToken::new(Token::DoubleAsterisk, start..next)),
                        }
                    }
                    '=' => {
                        self.forward();
                        tokens.push(SpannedToken::new(Token::AsteriskEqual, start..self.pos))
                    }

                    _ => tokens.push(SpannedToken::new(Token::Asterisk, start..start)),
                }
            }
            '/' => {
                let start = self.pos;
                match self.forward().unwrap() {
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
                                self.forward();
                                self.forward();
                                break;
                            }
                        }
                    }
                    '=' => {
                        tokens.push(SpannedToken::new(Token::SlashEqual, start..self.pos));
                        self.forward();
                    }

                    _ => tokens.push(SpannedToken::new(Token::Slash, start..start)),
                }
            }
            '.' => {
                tokens.push(SpannedToken::new(Token::Dot, self.current_point()));
                self.forward();
            }
            '0'..='9' => tokens.push(self.lex_number_literal(false)),
            'a'..='z' | 'A'..='Z' | '_' => {
                if self.peek_is('f') && (self.peek_offset_is('"', 1) || self.peek_offset_is('"', 2))
                {
                    let is_raw = self.peek_offset_is('r', 1);
                    if is_raw {
                        self.forward();
                    }
                    tokens.push(self.lex_format_string(is_raw))
                } else if self.peek_is('r')
                    && (self.peek_offset_is('"', 1) || self.peek_offset_is('"', 2))
                {
                    let is_format_string = self.peek_offset_is('f', 1);
                    self.forward();
                    if is_format_string {
                        self.forward();
                        tokens.push(self.lex_format_string(true))
                    } else {
                        tokens.push(self.lex_string_literal(false, true))
                    }
                } else {
                    tokens.push(self.lex_ident_and_keyword())
                }
            }
            '[' => {
                tokens.push(SpannedToken::new(Token::LBracket, self.current_point()));
                self.forward();
            }
            ']' => {
                tokens.push(SpannedToken::new(Token::RBracket, self.current_point()));
                self.forward();
            }
            '!' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '=' => {
                        tokens.push(SpannedToken::new(Token::NotEqual, start..self.pos));
                        self.forward();
                    }
                    _ => tokens.push(SpannedToken::new(Token::Bang, start..start)),
                }
            }
            '%' => {
                let start = self.pos;
                match self.forward().unwrap() {
                    '=' => {
                        tokens.push(SpannedToken::new(Token::PercentEqual, start..self.pos));
                        self.forward();
                    }
                    _ => tokens.push(SpannedToken::new(Token::Percent, start..start)),
                }
            }
            // Code structure
            '"' => {
                tokens.push(self.lex_string_literal(false, false));
            }
            _ => panic!(
                "Lexer founded unexpected character: {} @ {}:{}",
                self.peek().unwrap().to_string(),
                self.line,
                self.column
            ),
        }

        tokens
    }

    //noinspection DuplicatedCode
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens: Vec<SpannedToken> = Vec::new();

        while let Some(_) = self.peek() {
            tokens.extend(self.lex_primary());
        }

        tokens.push(SpannedToken::new(Token::EOF, self.current_point()));

        tokens
    }
}
