//! Operator binding powers for Pratt parser.

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
        Token::PlusPlus | Token::MinusMinus => 160, // postfix
        Token::OpenParen | Token::OpenBracket | Token::OpenBrace => 170,
        _ => 0,
    }
}

pub const fn rbp(tok: &Token) -> u8 {
    match tok {
        Token::Bang | Token::Tilde | Token::Plus | Token::Minus => 150,
        Token::PlusPlus | Token::MinusMinus => 160, // prefix
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{lbp, rbp};
    use crate::lexer::token::Token;

    #[test]
    fn reports_operator_binding_powers() {
        assert!(lbp(&Token::Star) > lbp(&Token::Plus));
        assert!(lbp(&Token::Plus) > lbp(&Token::OrOr));
        assert_eq!(rbp(&Token::Bang), 150);
        assert_eq!(lbp(&Token::PlusPlus), 160);
        assert_eq!(rbp(&Token::PlusPlus), 160);
        assert_eq!(lbp(&Token::OpenParen), 170);
        assert_eq!(lbp(&Token::Eq), 0);
    }
}
