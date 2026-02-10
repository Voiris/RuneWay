use crate::lexer::token::Token;

pub const fn lbp(tok: &Token) -> u8 {
    match tok {
        Token::OrOr => 40,
        Token::AndAnd => 50,
        Token::Or => 90,
        Token::Caret => 100,
        Token::And => 110,
        Token::Shl | Token::Shr => 120,
        Token::Plus | Token::Minus => 130,
        Token::Star | Token::Slash | Token::Percent => 140,
        Token::Lt | Token::Le | Token::Gt | Token::Ge => 70,
        Token::EqEq | Token::Ne => 60,
        // postfix
        Token::PlusPlus | Token::MinusMinus => 160,
        Token::OpenParen | Token::OpenBracket | Token::OpenBrace => 170,
        _ => 0,
    }
}

pub const fn rbp(tok: &Token) -> u8 {
    match tok {
        Token::Bang | Token::Tilde | Token::Plus | Token::Minus | Token::PlusPlus | Token::MinusMinus => 150,
        _ => 0,
    }
}
