use runec_source::byte_pos::BytePos;
use runec_source::source_map::SourceId;
use runec_source::span::Span;
use crate::generate_source;
use super::lexer_struct::*;
use super::token::*;

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
    let real_tokens = lexer.lex_full().unwrap();

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
    let real_tokens = lexer.lex_full().unwrap();

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
    let real_tokens = lexer.lex_full().unwrap();

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
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn format_string_test() {
    let source = "f\"{var}str{some}ing\\n\"";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::FormatStringStart, Span::new(BytePos::from_usize(1), BytePos::from_usize(1), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(2), BytePos::from_usize(3), source_id)),
        SpannedToken::new(Token::Ident(&source[3..6]), Span::new(BytePos::from_usize(3), BytePos::from_usize(6), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(6), BytePos::from_usize(7), source_id)),
        SpannedToken::new(Token::RawStringLiteral(&source[7..10]), Span::new(BytePos::from_usize(7), BytePos::from_usize(10), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(10), BytePos::from_usize(11), source_id)),
        SpannedToken::new(Token::Ident(&source[11..15]), Span::new(BytePos::from_usize(11), BytePos::from_usize(15), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(15), BytePos::from_usize(16), source_id)),
        SpannedToken::new(Token::StringLiteral("ing\n".to_string()), Span::new(BytePos::from_usize(16), BytePos::from_usize(21), source_id)),
        SpannedToken::new(Token::FormatStringEnd, Span::new(BytePos::from_usize(22), BytePos::from_usize(22), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn format_string_edge_cases_test() {
    let source = "f\"{}{  }{  var  }{ {v} }\"";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::FormatStringStart, Span::new(BytePos::from_usize(1), BytePos::from_usize(1), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(2), BytePos::from_usize(3), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(3), BytePos::from_usize(4), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(7), BytePos::from_usize(8), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(8), BytePos::from_usize(9), source_id)),
        SpannedToken::new(Token::Ident("var"), Span::new(BytePos::from_usize(11), BytePos::from_usize(14), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(16), BytePos::from_usize(17), source_id)),
        SpannedToken::new(Token::FormatCodeBlockStart, Span::new(BytePos::from_usize(17), BytePos::from_usize(18), source_id)),
        SpannedToken::new(Token::OpenBrace, Span::new(BytePos::from_usize(19), BytePos::from_usize(20), source_id)),
        SpannedToken::new(Token::Ident("v"), Span::new(BytePos::from_usize(20), BytePos::from_usize(21), source_id)),
        SpannedToken::new(Token::CloseBrace, Span::new(BytePos::from_usize(21), BytePos::from_usize(22), source_id)),
        SpannedToken::new(Token::FormatCodeBlockEnd, Span::new(BytePos::from_usize(23), BytePos::from_usize(24), source_id)),
        SpannedToken::new(Token::FormatStringEnd, Span::new(BytePos::from_usize(25), BytePos::from_usize(25), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn basic_int_literal_test() {
    let source = "123 0 999999999999999 0b0 0o0 0x0";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::IntLiteral { digits: "123", radix: Radix::Decimal, suffix: None }, Span::new(BytePos::from_usize(0), BytePos::from_usize(3), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Decimal, suffix: None }, Span::new(BytePos::from_usize(4), BytePos::from_usize(5), source_id)),
        SpannedToken::new(
            Token::IntLiteral { digits: "999999999999999", radix: Radix::Decimal, suffix: None },
            Span::new(BytePos::from_usize(6), BytePos::from_usize(21), source_id)
        ),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Binary, suffix: None }, Span::new(BytePos::from_usize(22), BytePos::from_usize(25), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Octal, suffix: None }, Span::new(BytePos::from_usize(26), BytePos::from_usize(29), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Hex, suffix: None }, Span::new(BytePos::from_usize(30), BytePos::from_usize(33), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn int_literal_with_suffix_test() {
    let source = "123u8 0i8 999999999999999f32 0b0u64 0o0isize 0x0suffix";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::IntLiteral { digits: "123", radix: Radix::Decimal, suffix: Some("u8") }, Span::new(BytePos::from_usize(0), BytePos::from_usize(5), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Decimal, suffix: Some("i8") }, Span::new(BytePos::from_usize(6), BytePos::from_usize(9), source_id)),
        SpannedToken::new(
            Token::IntLiteral { digits: "999999999999999", radix: Radix::Decimal, suffix: Some("f32") },
            Span::new(BytePos::from_usize(10), BytePos::from_usize(28), source_id)
        ),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Binary, suffix: Some("u64") }, Span::new(BytePos::from_usize(29), BytePos::from_usize(35), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Octal, suffix: Some("isize") }, Span::new(BytePos::from_usize(36), BytePos::from_usize(44), source_id)),
        SpannedToken::new(Token::IntLiteral { digits: "0", radix: Radix::Hex, suffix: Some("suffix") }, Span::new(BytePos::from_usize(45), BytePos::from_usize(54), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn float_literal_test() {
    let source = "3.14 0.0f32 0.0e1 0e+1 0e-1 0e1f64";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::FloatLiteral { literal: "3.14", suffix: None }, Span::new(BytePos::from_usize(0), BytePos::from_usize(4), source_id)),
        SpannedToken::new(Token::FloatLiteral { literal: "0.0", suffix: Some("f32") }, Span::new(BytePos::from_usize(5), BytePos::from_usize(11), source_id)),
        SpannedToken::new(Token::FloatLiteral { literal: "0.0e1", suffix: None }, Span::new(BytePos::from_usize(12), BytePos::from_usize(17), source_id)),
        SpannedToken::new(Token::FloatLiteral { literal: "0e+1", suffix: None }, Span::new(BytePos::from_usize(18), BytePos::from_usize(22), source_id)),
        SpannedToken::new(Token::FloatLiteral { literal: "0e-1", suffix: None }, Span::new(BytePos::from_usize(23), BytePos::from_usize(27), source_id)),
        SpannedToken::new(Token::FloatLiteral { literal: "0e1", suffix: Some("f64") }, Span::new(BytePos::from_usize(28), BytePos::from_usize(34), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn multichar_tokens_test() {
    let source = "= == + += ++ - -= -- -> * *= / /= % %= ^ ^= & &= | |= < << <<= <= > >> >>= >= . .. ..= : :: =>";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::Eq, Span::new(BytePos::from_usize(0), BytePos::from_usize(1), source_id)),
        SpannedToken::new(Token::EqEq, Span::new(BytePos::from_usize(2), BytePos::from_usize(4), source_id)),
        SpannedToken::new(Token::Plus, Span::new(BytePos::from_usize(5), BytePos::from_usize(6), source_id)),
        SpannedToken::new(Token::PlusEq, Span::new(BytePos::from_usize(7), BytePos::from_usize(9), source_id)),
        SpannedToken::new(Token::PlusPlus, Span::new(BytePos::from_usize(10), BytePos::from_usize(12), source_id)),
        SpannedToken::new(Token::Minus, Span::new(BytePos::from_usize(13), BytePos::from_usize(14), source_id)),
        SpannedToken::new(Token::MinusEq, Span::new(BytePos::from_usize(15), BytePos::from_usize(17), source_id)),
        SpannedToken::new(Token::MinusMinus, Span::new(BytePos::from_usize(18), BytePos::from_usize(20), source_id)),
        SpannedToken::new(Token::Arrow, Span::new(BytePos::from_usize(21), BytePos::from_usize(23), source_id)),
        SpannedToken::new(Token::Star, Span::new(BytePos::from_usize(24), BytePos::from_usize(25), source_id)),
        SpannedToken::new(Token::StarEq, Span::new(BytePos::from_usize(26), BytePos::from_usize(28), source_id)),
        SpannedToken::new(Token::Slash, Span::new(BytePos::from_usize(29), BytePos::from_usize(30), source_id)),
        SpannedToken::new(Token::SlashEq, Span::new(BytePos::from_usize(31), BytePos::from_usize(33), source_id)),
        SpannedToken::new(Token::Percent, Span::new(BytePos::from_usize(34), BytePos::from_usize(35), source_id)),
        SpannedToken::new(Token::PercentEq, Span::new(BytePos::from_usize(36), BytePos::from_usize(38), source_id)),
        SpannedToken::new(Token::Caret, Span::new(BytePos::from_usize(39), BytePos::from_usize(40), source_id)),
        SpannedToken::new(Token::CaretEq, Span::new(BytePos::from_usize(41), BytePos::from_usize(43), source_id)),
        SpannedToken::new(Token::And, Span::new(BytePos::from_usize(44), BytePos::from_usize(45), source_id)),
        SpannedToken::new(Token::AndEq, Span::new(BytePos::from_usize(46), BytePos::from_usize(48), source_id)),
        SpannedToken::new(Token::Or, Span::new(BytePos::from_usize(49), BytePos::from_usize(50), source_id)),
        SpannedToken::new(Token::OrEq, Span::new(BytePos::from_usize(51), BytePos::from_usize(53), source_id)),
        SpannedToken::new(Token::Lt, Span::new(BytePos::from_usize(54), BytePos::from_usize(55), source_id)),
        SpannedToken::new(Token::Shl, Span::new(BytePos::from_usize(56), BytePos::from_usize(58), source_id)),
        SpannedToken::new(Token::ShlEq, Span::new(BytePos::from_usize(59), BytePos::from_usize(62), source_id)),
        SpannedToken::new(Token::Le, Span::new(BytePos::from_usize(63), BytePos::from_usize(65), source_id)),
        SpannedToken::new(Token::Gt, Span::new(BytePos::from_usize(66), BytePos::from_usize(67), source_id)),
        SpannedToken::new(Token::Shr, Span::new(BytePos::from_usize(68), BytePos::from_usize(70), source_id)),
        SpannedToken::new(Token::ShrEq, Span::new(BytePos::from_usize(71), BytePos::from_usize(74), source_id)),
        SpannedToken::new(Token::Ge, Span::new(BytePos::from_usize(75), BytePos::from_usize(77), source_id)),
        SpannedToken::new(Token::Dot, Span::new(BytePos::from_usize(78), BytePos::from_usize(79), source_id)),
        SpannedToken::new(Token::Range, Span::new(BytePos::from_usize(80), BytePos::from_usize(82), source_id)),
        SpannedToken::new(Token::RangeInclusive, Span::new(BytePos::from_usize(83), BytePos::from_usize(86), source_id)),
        SpannedToken::new(Token::Colon, Span::new(BytePos::from_usize(87), BytePos::from_usize(88), source_id)),
        SpannedToken::new(Token::DColon, Span::new(BytePos::from_usize(89), BytePos::from_usize(91), source_id)),
        SpannedToken::new(Token::DArrow, Span::new(BytePos::from_usize(92), BytePos::from_usize(94), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn char_literal_test() {
    let source = r"'a' '\x30' '\u{30}' '\n' '\''";
    let (source_map, source_id) = generate_source(source);

    let expected_tokens = [
        SpannedToken::new(Token::CharLiteral('a'), Span::new(BytePos::from_usize(0), BytePos::from_usize(3), source_id)),
        SpannedToken::new(Token::CharLiteral('\x30'), Span::new(BytePos::from_usize(4), BytePos::from_usize(10), source_id)),
        SpannedToken::new(Token::CharLiteral('\u{30}'), Span::new(BytePos::from_usize(11), BytePos::from_usize(19), source_id)),
        SpannedToken::new(Token::CharLiteral('\n'), Span::new(BytePos::from_usize(20), BytePos::from_usize(24), source_id)),
        SpannedToken::new(Token::CharLiteral('\''), Span::new(BytePos::from_usize(25), BytePos::from_usize(29), source_id)),
    ];

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, expected_tokens);
}

#[test]
fn comment_test() {
    let source = r"/* comment block*/  // comment  \n   //comment 2";
    let (source_map, source_id) = generate_source(source);

    let lexer = Lexer::new(source_id, &source_map);
    let real_tokens = lexer.lex_full().unwrap();

    assert_eq!(real_tokens, []);
}
