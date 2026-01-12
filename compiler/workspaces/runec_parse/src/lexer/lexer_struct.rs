use std::borrow::Cow;
use fluent::FluentValue;
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::{DiagHelp, DiagLabel};
use runec_errors::message::DiagMessage;
use runec_source::byte_pos::BytePos;
use runec_source::source_map::{SourceFile, SourceId, SourceMap};
use runec_source::span::Span;
use crate::lexer::cursor::Cursor;
use crate::lexer::token::{SpannedToken, Token};

type LexerResult<'diag, T> = Result<T, Box<Diagnostic<'diag>>>;

pub struct Lexer<'src> {
    cursor: Cursor<'src>,
    source_id: SourceId,
    source_file: &'src SourceFile,
}

impl<'src, 'diag> Lexer<'src> {
    pub fn new(source_id: SourceId, source_map: &'src SourceMap) -> Self {
        let source_file = source_map.get_file(&source_id).unwrap();
        Self {
            cursor: Cursor::new(source_file.src.as_ref()),
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
        Diagnostic::error(DiagMessage::new("duplicated-string-literal-prefix", Some(runec_utils::hashmap!(
            "char" => FluentValue::String(Cow::Owned(prefix.to_string())),
        ))))
            .add_label(
                DiagLabel::silent_primary(Span::new(
                    lo,
                    hi,
                    self.source_id
                ))
            )
    }

    fn invalid_escape_sequence_error(&self, lo: BytePos, hi: BytePos, sequence: String) -> Box<Diagnostic<'diag>> {
        Diagnostic::error(DiagMessage::new("invalid-escape-sequence", Some(runec_utils::hashmap!(
            "sequence" => FluentValue::String(Cow::Owned(sequence)),
        ))))
            .add_label(
                DiagLabel::silent_primary(Span::new(
                    lo,
                    hi,
                    self.source_id,
                ))
            )
    }

    fn lex_identifier(&mut self) -> SpannedToken<'src> {
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
        let ident = &self.source_file.src[lo.to_usize()..hi.to_usize()];
        SpannedToken::new(Token::Ident(ident), Span::new(lo, hi, self.source_id))
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
                    Diagnostic::error(DiagMessage::new("invalid-char", Some(runec_utils::hashmap!(
                                "char" => FluentValue::String(Cow::Owned(c.to_string())),
                            ))))
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
                                    runec_errors::make_simple_diag!(
                                        error;
                                        "out-of-range-hex-escape",
                                        (: "out-of-range-hex-escape-label" : self.source_id => escape_lo..escape_hi)
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
                            runec_errors::make_simple_diag!(
                                error;
                                "invalid-unicode-escape",
                                (self.source_id => escape_lo..escape_hi),
                                {help = "unicode-escape-sequence-format"}
                            )
                        )
                    }
                    let hex_lo = self.cursor.pos();
                    self.cursor.skip_until_char_counted('}', 6);
                    if !matches!(self.cursor.peek_char(), Some('}')) {
                        let escape_hi = self.cursor.pos();
                        return Err(
                            runec_errors::make_simple_diag!(
                                error;
                                "invalid-unicode-escape",
                                (self.source_id => escape_lo..escape_hi),
                                {help = "unicode-must-have-at-most-6-hex-digits"}
                            )
                        )
                    }
                    let hex_hi = self.cursor.pos();
                    self.cursor.next();
                    let hex_str = &self.source_file.src[hex_lo.to_usize()..hex_hi.to_usize()];
                    let hex_opt = u32::from_str_radix(hex_str, 16);
                    if let Ok(hex) = hex_opt {
                        match hex {
                            0xD800..=0xDFFF => {
                                Err(
                                    runec_errors::make_simple_diag!(
                                        error;
                                        "invalid-unicode-escape",
                                        (self.source_id => hex_lo..hex_hi),
                                        {help = "unicode-escape-must-not-be-surrogate"}
                                    )
                                )
                            }
                            0x110000.. => {
                                Err(
                                    runec_errors::make_simple_diag!(
                                        error;
                                        "invalid-unicode-escape",
                                        (self.source_id => hex_lo..hex_hi),
                                        {help = "unicode-escape-must-be-in-range"}
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
                            runec_errors::make_simple_diag!(
                                error;
                                "invalid-unicode-escape",
                                (self.source_id => escape_lo..escape_hi),
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
                runec_errors::make_simple_diag!(
                    error;
                    "unterminated-escape-sequence",
                    (self.source_id => escape_lo..escape_hi)
                )
            )
        }
    }

    fn lex_string_literal(&mut self, is_raw: bool, is_format: bool, consume_starter_quote: bool) -> LexerResult<'diag, SpannedToken<'src>> {
        let lo = self.cursor.pos();

        if consume_starter_quote {
            self.cursor.next();
        }

        let raw_str_lo = self.cursor.pos();

        let mut string_opt = Option::<String>::None;
        let mut is_terminated = false;
        let mut cursor_clone = self.cursor.clone();
        let mut raw_chars_count = 0u32; // uses before first escape sequence and if is_raw == false
        while let Some(char) = self.cursor.peek_char() {
            match char {
                '"' => {
                    is_terminated = true;
                    break;
                }
                '{' if is_format => { unimplemented!() }
                '\\' if !is_raw => {
                    let string = string_opt.get_or_insert_with(
                        || cursor_clone.try_next_slice(raw_chars_count as usize).unwrap().to_string()
                    );
                    if let Some(char) = self.lex_escape_sequence()? {
                        string.push(char);
                    }
                }
                c => {
                    if let Some(ref mut string) = string_opt {
                        string.push(self.cursor.next_char().unwrap());
                    } else {
                        self.cursor.next();
                        raw_chars_count += 1;
                    }
                }
            }
        }

        let raw_str_hi = self.cursor.pos();

        self.cursor.next();

        let hi = self.cursor.pos();

        if !is_terminated {
            return Err(
                runec_errors::make_simple_diag!(
                    error;
                    "unterminated-string",
                    (self.source_id => lo..hi)
                )
            )
        }

        let token = if let Some(string) = string_opt {
            Token::StringLiteral(string)
        } else {
            Token::RawStringLiteral(&self.source_file.src[raw_str_lo.to_usize()..raw_str_hi.to_usize()])
        };

        Ok(SpannedToken::new(token, Span::new(lo, hi, self.source_id)))
    }

    fn lex_string(&mut self, is_raw: bool, is_format: bool) -> LexerResult<'diag, Vec<SpannedToken<'src>>> {
        unimplemented!()
    }

    pub fn lex(mut self) -> LexerResult<'diag, Vec<SpannedToken<'src>>> {
        let mut tokens = Vec::new();

        while self.cursor.peek().is_some() {
            while let Some(ch) = self.cursor.peek_char() {
                if ch.is_whitespace() {
                    self.cursor.next();
                } else {
                    break;
                }
            }

            if let Some(ch) = self.cursor.peek_char().cloned() {
                let new_token_opt = match ch {
                    '(' => self.span_one_char(Token::OpenParen),
                    ')' => self.span_one_char(Token::CloseParen),
                    '{' => self.span_one_char(Token::OpenBrace),
                    '}' => self.span_one_char(Token::CloseBrace),
                    '"' => {
                        Some(self.lex_string_literal(false, false, true)?)
                    }
                    'r' | 'f'
                    if self.cursor.lookahead_char(1) == Some('"')
                    || self.cursor.lookahead_char(2) == Some('"') => {
                        let (is_raw, is_format) = self.handle_string_prefix()?;
                        unimplemented!()
                    }
                    'A'..='Z' | 'a'..='z' | '_' => {
                        Some(self.lex_identifier())
                    }
                    _ => {
                        let lo = self.cursor.pos();
                        let hi = lo + ch.len_utf8();
                        return Err(
                            Diagnostic::error(DiagMessage::new("invalid-char", Some(runec_utils::hashmap!(
                                "char" => FluentValue::String(Cow::Owned(ch.to_string())),
                            ))))
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
                    tokens.push(new_token);
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use runec_source::source_map::FileName;
    use runec_source::byte_pos::BytePos;
    use super::*;

    fn generate_source(source: &str) -> (SourceMap, SourceId) {
        let mut source_map = SourceMap::new();
        let source_id = source_map.add_file(
            SourceFile::new(FileName::Real(PathBuf::from("/home/user/main.rnw")), source.to_string()),
        );
        (source_map, source_id)
    }

    fn span(lo: usize, hi: usize, src_id: SourceId) -> Span {
        Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), src_id)
    }

    #[test]
    fn one_char_tokens_test() {
        let (source_map, source_id) = generate_source("(){}");

        let expected_tokens = [
            SpannedToken::new(Token::OpenParen, span(0, 1, source_id)),
            SpannedToken::new(Token::CloseParen, span(1, 2, source_id)),
            SpannedToken::new(Token::OpenBrace, span(2, 3, source_id)),
            SpannedToken::new(Token::CloseBrace, span(3, 4, source_id)),
        ];

        let lexer = Lexer::new(source_id, &source_map);
        let real_tokens = lexer.lex().unwrap();

        assert_eq!(real_tokens, expected_tokens);
    }

    #[test]
    fn ident_lex_test() {
        let source = "main r DaDa r9_ r_9 _";
        let (source_map, source_id) = generate_source(source);

        let expected_tokens = [
            SpannedToken::new(Token::Ident(&source[0..4]), Span::new(BytePos::from_usize(0), BytePos::from_usize(4), source_id)),
            SpannedToken::new(Token::Ident(&source[5..6]), Span::new(BytePos::from_usize(5), BytePos::from_usize(6), source_id)),
            SpannedToken::new(Token::Ident(&source[7..11]), Span::new(BytePos::from_usize(7), BytePos::from_usize(11), source_id)),
            SpannedToken::new(Token::Ident(&source[12..15]), Span::new(BytePos::from_usize(12), BytePos::from_usize(15), source_id)),
            SpannedToken::new(Token::Ident(&source[16..19]), Span::new(BytePos::from_usize(16), BytePos::from_usize(19), source_id)),
            SpannedToken::new(Token::Ident(&source[20..21]), Span::new(BytePos::from_usize(20), BytePos::from_usize(21), source_id)),
        ];

        let lexer = Lexer::new(source_id, &source_map);
        let real_tokens = lexer.lex().unwrap();

        assert_eq!(real_tokens, expected_tokens);
    }

    #[test]
    fn basic_string_literal_test() {
        let source = "\"string\" \"\"";
        let (source_map, source_id) = generate_source(source);

        let expected_tokens = [
            SpannedToken::new(Token::RawStringLiteral(&source[1..7]), Span::new(BytePos::from_usize(0), BytePos::from_usize(8), source_id)),
            SpannedToken::new(Token::RawStringLiteral(&source[10..10]), Span::new(BytePos::from_usize(9), BytePos::from_usize(11), source_id)),
        ];

        let lexer = Lexer::new(source_id, &source_map);
        let real_tokens = lexer.lex().unwrap();

        assert_eq!(real_tokens, expected_tokens);
    }

    #[test]
    fn escape_sequence_test() {
        let source = "\"\\x01\\u{0012}\\u{FF}\\t\\r\\n\"";
        let (source_map, source_id) = generate_source(source);

        let expected_tokens = [
            SpannedToken::new(Token::StringLiteral("\x01\u{0012}\u{FF}\t\r\n".to_string()), Span::new(BytePos::from_usize(0), BytePos::from_usize(26), source_id)),
        ];

        let lexer = Lexer::new(source_id, &source_map);
        let real_tokens = lexer.lex().unwrap();

        assert_eq!(real_tokens, expected_tokens);
    }
}