use runec_source::span::Spanned;
use crate::expression::SpannedExpr;

#[derive(Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    Unit,
    Ident(&'src str),
    Tuple(Box<[SpannedTypeAnnotation<'src>]>),
    Array { 
        item: Box<SpannedTypeAnnotation<'src>>,
        length: SpannedExpr<'src>
    },
    Generic {
        ty: Box<SpannedTypeAnnotation<'src>>,
        params: Box<[GenericParameter<'src>]>,
    },
}

#[derive(Debug, PartialEq)]
pub enum GenericParameter<'src> {
    Type(SpannedTypeAnnotation<'src>),
    Expr(SpannedExpr<'src>),
}

pub type SpannedTypeAnnotation<'src> = Spanned<TypeAnnotation<'src>>;
