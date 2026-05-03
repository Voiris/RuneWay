use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::{DiagHelp, DiagLabel};
use runec_errors::message::DiagMessage;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::{Source, SourceId, SourceMap};
use runec_source::span;
use runec_source::span::Span;
use crate::lexer::cursor::Cursor;
use crate::lexer::token::{Radix, SpannedToken, Token};

type LexerResult<'diag, T> = Result<T, Box<Diagnostic<'diag>>>;

macro_rules! handle_double_char_token {
    (
        $self:ident; $one_char_token:expr;
        $(
            $ch:pat => $double_char_token:expr
        ),*
        $(,)?
    ) => {
        match $self.cursor.peek_char() {
            $(
                Some($ch) => {
                    $self.cursor.next();
                    $double_char_token
                }
            ),*
            _ => $one_char_token
        }
    };

    (
        *;
        $self:ident; $one_char_token:expr;
        $(
            $ch:pat => $double_char_token:expr
        ),*
        $(,)?
    ) => {{
        handle_double_char_token!(@wrapper $self; token; {
            let token = handle_double_char_token!(
                $self; $one_char_token;
                $(
                    $ch => $double_char_token
                ),*
            )
        })
    }};

    (@wrapper $self:ident; $token:ident; {$($code:tt)*}) => {{
        let lo = $self.cursor.pos();
        $self.cursor.next();
        $($code)*;
        let hi = $self.cursor.pos();
        Some(SpannedToken::new($token, Span::new(lo, hi, $self.source_id)))
    }}
}

pub struct Lexer<'src> {
    cursor: Cursor<'src>,
    source_id: SourceId,
    source_file: &'src Source,
}

impl<'src, 'diag> Lexer<'src> {
    pub fn new(source_id: SourceId, source_map: &'src SourceMap) -> Self {
        let source_file = source_map.get_file(&source_id).unwrap();
        Self {
            cursor: Cursor::new(source_file.src()),
            source_id,
            source_file,
        }
    }

    fn span_one_char(&mut self, token: Token<'src>) -> Option<SpannedToken<'src>> {
        let lo = self.cursor.pos();
        self.cursor.next();
        let hi = self.cursor.pos();
        Some(SpannedToken::new(token, Span::new(lo, hi, self.source_id)))
    }

    fn duplicated_prefix_error(&self, prefix: char, lo: BytePos, hi: BytePos) -> Box<Diagnostic<'diag>> {
        Diagnostic::error(DiagMessage::new(super::messages::DUPLICATED_STRING_LITERAL_PREFIX, &[
            ("char", &prefix.to_string())
        ]))
            .add_label(
                DiagLabel::silent_primary(Span::new(
                    lo,
                    hi,
                    self.source_id
                ))
            )
    }

    fn invalid_escape_sequence_error(&self, lo: BytePos, hi: BytePos, sequence: String) -> Box<Diagnostic<'diag>> {
        Diagnostic::error(DiagMessage::new(super::messages::INVALID_ESCAPE_SEQUENCE, &[
            ("sequence", &sequence)
        ]))
            .add_label(
                DiagLabel::silent_primary(Span::new(
                    lo,
                    hi,
                    self.source_id,
                ))
            )
    }

    fn lex_identifier_or_keyword(&mut self) -> SpannedToken<'src> {
        let lo = self.cursor.pos();
        while let Some(char) = self.cursor.peek_char() {
            match char {
                // caller guarantees first char is [A-Za-z_]
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                    self.cursor.next();
                }
                _ => break,
            }
        }
        let hi = self.cursor.pos();
        let ident = &self.source_file.src()[lo.to_usize()..hi.to_usize()];

        let token = match ident {
            "act" => Token::Act,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "const" => Token::Const,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "as" => Token::As,
            "pub" => Token::Pub,
            "alias" => Token::Alias,
            "enum" => Token::Enum,
            "union" => Token::Union,
            "struct" => Token::Struct,
            "impl" => Token::Impl,
            "contract" => Token::Contract,
            "use" => Token::Use,
            "unsafe" => Token::Unsafe,
            _ => Token::Ident(ident)
        };

        SpannedToken::new(token, Span::new(lo, hi, self.source_id))
    }

    fn handle_string_prefix(&mut self) -> LexerResult<'diag, (bool, bool)> {
        let lo = self.cursor.pos();
        let first = self.cursor.next_char().unwrap();
        let second = *self.cursor.peek_char().unwrap();

        match (first, second) {
            ('r', 'f') | ('f', 'r') => {
                self.cursor.next();
                Ok((true, true))
            },
            ('f', 'f') | ('r', 'r') => {
                self.cursor.next();
                let hi = self.cursor.pos();
                Err(self.duplicated_prefix_error(first, lo, hi))
            },
            ('r', '"') => Ok((true, false)),
            ('f', '"') => Ok((false, true)),
            (_, c) => {
                self.cursor.next();
                let hi = self.cursor.pos();
                Err(
                    Diagnostic::error(DiagMessage::new(super::messages::INVALID_CHAR, &[
                        ("char", &c.to_string())
                    ]))
                        .add_label(
                            DiagLabel::silent_primary(Span::new(
                                lo,
                                hi,
                                self.source_id,
                            ))
                        )
                )
            }
        }
    }

    fn lex_escape_sequence(&mut self) -> LexerResult<'diag, Option<char>> {
        // expect `\` is not skipped
        let escape_lo = self.cursor.pos();
        self.cursor.next();
        if let Some(char) = self.cursor.next_char() {
            match char {
                'n' => Ok(Some('\n')),
                'r' => Ok(Some('\r')),
                't' => Ok(Some('\t')),
                '\\' => Ok(Some('\\')),
                '"' => Ok(Some('"')),
                '\'' => Ok(Some('\'')),
                '\n' => Ok(None),
                'x' => {
                    let hex_str_opt = self.cursor.try_next_slice(2);

                    if let Some(hex_str) = hex_str_opt {
                        let hex_opt = u8::from_str_radix(hex_str, 16);

                        if let Ok(hex) = hex_opt {
                            const MAX_HEX_ESCAPE: u8 = 0x7f;
                            if hex > MAX_HEX_ESCAPE {
                                let escape_hi = self.cursor.pos();
                                return Err(
                                    Diagnostic::error(
                                        DiagMessage::new(
                                            super::messages::OUT_OF_RANGE_HEX_ESCAPE,
                                            &[]
                                        )
                                    )
                                        .add_label(
                                            DiagLabel::simple_primary(super::messages::OUT_OF_RANGE_HEX_ESCAPE, span!(self.source_id => escape_lo..escape_hi))
                                        )
                                )
                            }
                            Ok(Some(hex as char))
                        } else {
                            let escape_hi = self.cursor.pos();
                            Err(self.invalid_escape_sequence_error(escape_lo, escape_hi, r"\x".to_string()))
                        }
                    } else {
                        let escape_hi = self.cursor.pos();
                        Err(self.invalid_escape_sequence_error(escape_lo, escape_hi, r"\x".to_string()))
                    }
                },
                'u' => {
                    if !matches!(self.cursor.next_char(), Some('{')) {
                        let escape_hi = self.cursor.pos();
                        return Err(
                            Diagnostic::error(
                                DiagMessage::new(
                                    super::messages::INVALID_UNICODE_ESCAPE,
                                    &[]
                                )
                            )
                                .add_label(
                                    DiagLabel::silent_primary(span!(self.source_id => escape_lo..escape_hi))
                                )
                                .set_help(
                                    DiagHelp::new(super::messages::UNICODE_ESCAPE_SEQUENCE_FORMAT, &[])
                                )
                        )
                    }
                    let hex_lo = self.cursor.pos();
                    self.cursor.skip_until_char_counted('}', 6);
                    if !matches!(self.cursor.peek_char(), Some('}')) {
                        let escape_hi = self.cursor.pos();
                        return Err(
                            Diagnostic::error(
                                DiagMessage::new(
                                    super::messages::INVALID_UNICODE_ESCAPE,
                                    &[]
                                )
                            )
                                .add_label(
                                    DiagLabel::silent_primary(span!(self.source_id => escape_lo..escape_hi))
                                )
                                .set_help(
                                    DiagHelp::new(super::messages::UNICODE_MUST_HAVE_AT_MOST_6_HEX_DIGITS, &[])
                                )
                        )
                    }
                    let hex_hi = self.cursor.pos();
                    self.cursor.next();
                    let hex_str = &self.source_file.src()[hex_lo.to_usize()..hex_hi.to_usize()];
                    let hex_opt = u32::from_str_radix(hex_str, 16);
                    if let Ok(hex) = hex_opt {
                        match hex {
                            0xD800..=0xDFFF => {
                                Err(
                                    Diagnostic::error(
                                        DiagMessage::new(
                                            super::messages::INVALID_UNICODE_ESCAPE,
                                            &[]
                                        )
                                    )
                                        .add_label(
                                            DiagLabel::silent_primary(span!(self.source_id => hex_lo..hex_hi))
                                        )
                                        .set_help(
                                            DiagHelp::new(super::messages::UNICODE_ESCAPE_MUST_NOT_BE_SURROGATE, &[])
                                        )
                                )
                            }
                            0x110000.. => {
                                Err(
                                    Diagnostic::error(
                                        DiagMessage::new(
                                            super::messages::INVALID_UNICODE_ESCAPE,
                                            &[]
                                        )
                                    )
                                        .add_label(
                                            DiagLabel::silent_primary(span!(self.source_id => hex_lo..hex_hi))
                                        )
                                        .set_help(
                                            DiagHelp::new(super::messages::UNICODE_ESCAPE_MUST_BE_IN_RANGE, &[])
                                        )
                                )
                            }
                            hex => {
                                // SAFETY: `hex` is guaranteed to be a valid Unicode scalar value
                                Ok(Some(unsafe { char::from_u32_unchecked(hex) }))
                            }
                        }
                    } else {
                        let escape_hi = self.cursor.pos();
                        Err(
                            Diagnostic::error(
                                DiagMessage::new(
                                    super::messages::INVALID_UNICODE_ESCAPE,
                                    &[]
                                )
                            )
                                .add_label(
                                    DiagLabel::silent_primary(span!(self.source_id => escape_lo..escape_hi))
                                )
                        )
                    }
                }
                _ => {
                    let escape_hi = self.cursor.pos();
                    Err(
                        self.invalid_escape_sequence_error(escape_lo, escape_hi, format!("\\{}", char))
                    )
                }
            }
        } else {
            let escape_hi = self.cursor.pos();
            Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::UNTERMINATED_ESCAPE_SEQUENCE,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => escape_lo..escape_hi))
                    )
            )
        }
    }

    fn lex_string_literal(&mut self, is_raw: bool, is_format: bool, consume_starter_quote: bool) -> LexerResult<'diag, (SpannedToken<'src>, bool)> {
        let lo = self.cursor.pos();

        if consume_starter_quote {
            self.cursor.next();
        }

        let raw_str_lo = self.cursor.pos();

        let mut string_opt = Option::<String>::None;
        let mut is_terminated = false;
        let mut is_quote_terminated = false;
        while let Some(char) = self.cursor.peek_char() {
            match char {
                '"' => {
                    is_terminated = true;
                    is_quote_terminated = true;
                    break;
                }
                '{' if is_format => {
                    is_terminated = true;
                    break;
                }
                '\\' if !is_raw => {
                    let string = string_opt.get_or_insert_with(
                        || self.source_file.src()[raw_str_lo.to_usize()..self.cursor.pos().to_usize()].to_string()
                    );
                    if let Some(char) = self.lex_escape_sequence()? {
                        string.push(char);
                    }
                }
                _ => {
                    if let Some(ref mut string) = string_opt {
                        string.push(self.cursor.next_char().unwrap());
                    } else {
                        self.cursor.next();
                    }
                }
            }
        }

        let raw_str_hi = self.cursor.pos();

        if is_quote_terminated {
            self.cursor.next();
        }

        let hi = self.cursor.pos();

        if !is_terminated {
            return Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::UNTERMINATED_STRING,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                    )
            )
        }

        let token = if let Some(string) = string_opt {
            Token::StringLiteral(string)
        } else {
            Token::RawStringLiteral(&self.source_file.src()[raw_str_lo.to_usize()..raw_str_hi.to_usize()])
        };

        Ok((SpannedToken::new(token, Span::new(lo, hi, self.source_id)), is_quote_terminated))
    }

    fn lex_string(&mut self, is_raw: bool, is_format: bool) -> LexerResult<'diag, Vec<SpannedToken<'src>>> {
        let lo = self.cursor.pos();

        self.cursor.next();

        let mut tokens = Vec::new();

        if is_format {
            tokens.push(SpannedToken::new(Token::FormatStringStart, Span::new(lo, lo, self.source_id)))
        }

        let mut is_terminated = false;
        let mut brace_level = 0;

        while let Some(char) = self.cursor.peek_char() {
            match char {
                '{' => {
                    brace_level += 1;
                    tokens.push(self.span_one_char(Token::FormatCodeBlockStart).unwrap());
                    while let Some(char) = { self.cursor.consume_while(|c| c.is_whitespace()); self.cursor.peek_char().cloned() } {
                        match char {
                            '}' if brace_level == 1 => {
                                brace_level -= 1;
                                tokens.push(self.span_one_char(Token::FormatCodeBlockEnd).unwrap());
                                break;
                            }
                            '}' => {
                                brace_level -= 1;
                                tokens.push(self.span_one_char(Token::CloseBrace).unwrap());
                            }
                            '{' => {
                                brace_level += 1;
                                tokens.push(self.span_one_char(Token::OpenBrace).unwrap());
                            }
                            '"' => {
                                break;
                            }
                            _ => {
                                tokens.extend(self.lex()?)
                            }
                        }
                    }
                }
                '"' => {
                    is_terminated = true;
                    self.cursor.next();
                    break;
                }
                _ => {
                    let (literal, is_quote_terminated) = self.lex_string_literal(is_raw, is_format, false)?;

                    if is_quote_terminated {
                        // Not count ending quote
                        tokens.push(SpannedToken::new(literal.node, Span::new(literal.span.lo, literal.span.hi - 1, literal.span.src_id)));
                        is_terminated = true;
                        break;
                    } else {
                        tokens.push(literal);
                    }
                }
            }
        }

        if brace_level > 0 {
            let hi = self.cursor.pos();
            return Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::UNTERMINATED_F_STRING_CODE_BLOCK,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                    )
            )
        }

        if !is_terminated {
            let hi = self.cursor.pos();
            return Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::UNTERMINATED_STRING,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                    )
            )
        }

        if is_format {
            let hi = self.cursor.pos();
            tokens.push(SpannedToken::new(Token::FormatStringEnd, Span::new(hi, hi, self.source_id)))
        }

        Ok(tokens)
    }

    fn lex_char_literal(&mut self) -> LexerResult<'diag, SpannedToken<'src>> {
        let lo = self.cursor.pos();
        // Consume `'` (guaranteed by caller)
        self.cursor.next();

        let char = match self.cursor.peek_char() {
            Some('\\') => {
                let lo = self.cursor.pos();
                let escape_seq_opt = self.lex_escape_sequence()?;
                let hi = self.cursor.pos();
                match escape_seq_opt {
                    Some(escape_seq) => {
                        escape_seq
                    }
                    None => {
                        return Err(
                            Diagnostic::error(
                                DiagMessage::new(
                                    super::messages::UNTERMINATED_CHAR_LITERAL,
                                    &[]
                                )
                            )
                                .add_label(
                                    DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                                )
                        )
                    }
                }
            }
            Some('\'') => {
                self.cursor.next();
                let hi = self.cursor.pos();
                return Err(
                    Diagnostic::error(
                        DiagMessage::new(
                            super::messages::EMPTY_CHAR_LITERAL,
                            &[]
                        )
                    )
                        .add_label(
                            DiagLabel::simple_primary(super::messages::EMPTY_CHAR_LITERAL, span!(self.source_id => lo..hi))
                        )
                )
            }
            Some(_) => { self.cursor.next_char().unwrap() },
            None => {
                let hi = lo + '\''.len_utf8();
                return Err(
                    Diagnostic::error(
                        DiagMessage::new(
                            super::messages::UNTERMINATED_CHAR_LITERAL,
                            &[]
                        )
                    )
                        .add_label(
                            DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                        )
                )
            }
        };

        if self.cursor.next_char() != Some('\'') {
            let hi = self.cursor.pos();
            return Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::UNTERMINATED_CHAR_LITERAL,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => lo..hi))
                    )
            )
        }

        let hi = self.cursor.pos();

        Ok(SpannedToken::new(Token::CharLiteral(char), Span::new(lo, hi, self.source_id)))
    }

    fn lex_number(&mut self) -> LexerResult<'diag, SpannedToken<'src>> {
        let lo = self.cursor.pos();

        let mut is_float = false;
        let mut is_exponent = false;

        let radix: Radix = {
            let lookahead = self.cursor.lookahead_char(1);
            match (self.cursor.peek_char(), lookahead) {
                (Some('0'), Some('b' | 'o' | 'x')) => {
                    self.cursor.next(); // 0
                    self.cursor.next(); // b | o | x
                    match lookahead {
                        Some('b') => Radix::Binary,
                        Some('o') => Radix::Octal,
                        Some('x') => Radix::Hex,
                        _ => unreachable!()
                    }
                }
                _ => Radix::Decimal
            }
        };

        let digits_lo = self.cursor.pos();
        let mut has_digits = false;

        // Digits
        while let Some(&char) = self.cursor.peek_char() {
            match char {
                '0' | '1' if radix == Radix::Binary => {
                    has_digits = true;
                    self.cursor.next();
                }
                '0'..='7' if radix == Radix::Octal => {
                    has_digits = true;
                    self.cursor.next();
                }
                '0'..='9' if radix == Radix::Decimal => {
                    has_digits = true;
                    self.cursor.next();
                }
                '0'..='9' | 'A'..='F' | 'a'..='f' if radix == Radix::Hex => {
                    has_digits = true;
                    self.cursor.next();
                }
                '_' => {
                    self.cursor.next();
                }
                '.' if !is_float && radix == Radix::Decimal
                    && matches!(self.cursor.lookahead_char(1), Some('0'..='9')) => {
                    self.cursor.next();
                    is_float = true;
                }
                'e' | 'E' if !is_exponent && radix == Radix::Decimal => {
                    match (
                        self.cursor.lookahead_char(1),
                        self.cursor.lookahead_char(2),
                        ) {
                        (Some('0'..='9'), _) => {
                            self.cursor.next(); // e | E
                            self.cursor.next(); // digit
                            is_exponent = true;
                        }
                        (Some('+') | Some('-'), Some('0'..='9')) => {
                            self.cursor.next(); // e | E
                            self.cursor.next(); // + | -
                            self.cursor.next(); // digit
                            is_exponent = true;
                        }
                        _ => break
                    }
                }
                _ => break
            }
        }

        let digits_hi = self.cursor.pos();

        if !has_digits {
            return Err(
                Diagnostic::error(
                    DiagMessage::new(
                        super::messages::NO_VALID_DIGITS,
                        &[]
                    )
                )
                    .add_label(
                        DiagLabel::silent_primary(span!(self.source_id => lo..digits_hi))
                    )
            )
        }

        // Suffix
        while let Some(char) = self.cursor.peek_char() {
            match char {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    self.cursor.next();
                }
                _ => break
            }
        }

        let hi = self.cursor.pos();

        let suffix = {
            if digits_hi == hi {
                None
            } else {
                Some(&self.source_file.src()[digits_hi.to_usize()..hi.to_usize()])
            }
        };

        let slice = &self.source_file.src()[digits_lo.to_usize()..digits_hi.to_usize()];
        let span = Span::new(lo, hi, self.source_id);

        if is_float || is_exponent {
            Ok(SpannedToken::new(Token::FloatLiteral {
                literal: slice,
                suffix
            }, span))
        } else {
            Ok(SpannedToken::new(Token::IntLiteral {
                digits: slice,
                radix,
                suffix
            }, span))
        }
    }

    pub fn lex(&mut self) -> LexerResult<'diag, Vec<SpannedToken<'src>>> {
        while let Some(ch) = self.cursor.peek_char() {
            if ch.is_whitespace() {
                self.cursor.next();
            } else {
                break;
            }
        }

        if let Some(ch) = self.cursor.peek_char().cloned() {
            let new_token_opt = match ch {
                // One char lexing
                '(' => self.span_one_char(Token::OpenParen),
                ')' => self.span_one_char(Token::CloseParen),
                '{' => self.span_one_char(Token::OpenBrace),
                '}' => self.span_one_char(Token::CloseBrace),
                '[' => self.span_one_char(Token::OpenBracket),
                ']' => self.span_one_char(Token::CloseBracket),
                ',' => self.span_one_char(Token::Comma),
                ';' => self.span_one_char(Token::Semicolon),
                // Multichar lexing
                '=' => {
                    handle_double_char_token!(
                        *; self; Token::Eq;
                        '=' => Token::EqEq,
                        '>' => Token::DArrow,
                    )
                }
                '+' => {
                    handle_double_char_token!(
                        *; self; Token::Plus;
                        '=' => Token::PlusEq,
                        '+' => Token::PlusPlus
                    )
                }
                '-' => {
                    handle_double_char_token!(
                        *; self; Token::Minus;
                        '=' => Token::MinusEq,
                        '-' => Token::MinusMinus,
                        '>' => Token::Arrow,
                    )
                }
                '*' => {
                    handle_double_char_token!(
                        *; self; Token::Star;
                        '=' => Token::StarEq
                    )
                }
                '/' => {
                    let lo = self.cursor.pos();
                    self.cursor.next();
                    let token = match self.cursor.peek_char() {
                        Some('=') => {
                            self.cursor.next();
                            Some(Token::SlashEq)
                        }
                        Some('/') => {
                            self.cursor.skip_until_char('\n');
                            None
                        }
                        Some('*') => {
                            loop {
                                self.cursor.skip_until_char('*');
                                self.cursor.next();
                                match self.cursor.next_char() {
                                    Some('/') => {
                                        break;
                                    }
                                    None => {
                                        return Err(
                                            Diagnostic::error(
                                                DiagMessage::new(
                                                    super::messages::UNTERMINATED_COMMENT_BLOCK,
                                                    &[]
                                                )
                                            )
                                        )
                                    }
                                    _ => {}
                                }
                            }
                            None
                        }
                        _ => Some(Token::Slash)
                    };
                    let hi = self.cursor.pos();

                    token.map(|x| SpannedToken::new(x, Span::new(lo, hi, self.source_id)))
                }
                '%' => {
                    handle_double_char_token!(
                        *; self; Token::Percent;
                        '=' => Token::PercentEq
                    )
                }
                '^' => {
                    handle_double_char_token!(
                        *; self; Token::Caret;
                        '=' => Token::CaretEq
                    )
                }
                '&' => {
                    handle_double_char_token!(
                        *; self; Token::And;
                        '=' => Token::AndEq
                    )
                }
                '|' => {
                    handle_double_char_token!(
                        *; self; Token::Or;
                        '=' => Token::OrEq
                    )
                }
                '<' => {
                    handle_double_char_token!(
                        *; self; Token::Lt;
                        '<' => handle_double_char_token!(
                            self; Token::Shl;
                            '=' => Token::ShlEq
                        ),
                        '=' => Token::Le
                    )
                }
                '>' => {
                    handle_double_char_token!(
                        *; self; Token::Gt;
                        '>' => handle_double_char_token!(
                            self; Token::Shr;
                            '=' => Token::ShrEq
                        ),
                        '=' => Token::Ge
                    )
                }
                '.' => {
                    handle_double_char_token!(
                        *; self; Token::Dot;
                        '.' => handle_double_char_token!(
                            self; Token::Range;
                            '=' => Token::RangeInclusive
                        )
                    )
                }
                ':' => {
                    handle_double_char_token!(
                        *; self; Token::Colon;
                        ':' => Token::DColon
                    )
                }

                // Complex lexing
                '"' => {
                    Some(self.lex_string_literal(false, false, true)?.0)
                }
                'r' | 'f'
                if self.cursor.lookahead_char(1) == Some('"')
                    || self.cursor.lookahead_char(2) == Some('"') => {
                    let (is_raw, is_format) = self.handle_string_prefix()?;
                    return self.lex_string(is_raw, is_format);
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    Some(self.lex_identifier_or_keyword())
                }
                '0'..='9' => {
                    Some(self.lex_number()?)
                }
                '\'' => {
                    Some(self.lex_char_literal()?)
                }
                _ => {
                    let lo = self.cursor.pos();
                    let hi = lo + ch.len_utf8();
                    return Err(
                        Diagnostic::error(DiagMessage::new(super::messages::INVALID_CHAR, &[
                            ("char", &ch.to_string()),
                        ]))
                            .add_label(
                                DiagLabel::silent_primary(Span::new(
                                    lo,
                                    hi,
                                    self.source_id
                                ))
                            )
                    )
                }
            };

            if let Some(new_token) = new_token_opt {
                Ok(vec![new_token])
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    pub fn lex_full(mut self) -> LexerResult<'diag, Vec<SpannedToken<'src>>> {
        let mut tokens = Vec::new();

        while self.cursor.peek().is_some() {
            tokens.extend(self.lex()?);
        }

        Ok(tokens)
    }
}
