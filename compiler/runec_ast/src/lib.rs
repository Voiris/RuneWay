use runec_source::span::Spanned;

pub mod ast_type;
pub mod expression;
pub mod operators;
pub mod statement;

pub type SpannedStr<'src> = Spanned<&'src str>;
