use std::borrow::Cow;
use fluent::FluentValue;
use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::DiagLabel;
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
}