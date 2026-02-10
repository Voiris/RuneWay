use runec_source::span::Spanned;

pub mod expression;
pub mod statement;
pub mod ast_type;
pub mod operators;

pub type SpannedStr<'src> = Spanned<&'src str>;
