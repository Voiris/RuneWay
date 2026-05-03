use runec_source::span::Spanned;
use crate::expression::SpannedExpr;

#[derive(Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    Unit,
    Ident(&'src str),
    Tuple(Box<[SpannedTypeAnnotation<'src>]>),
    Path {
        from_root: bool,
        path: Box<[SpannedTypeAnnotation<'src>]>
    },
    Array {
        item: Box<SpannedTypeAnnotation<'src>>,
        length: SpannedExpr<'src>
    },
    Generic {
        ty: Box<SpannedTypeAnnotation<'src>>,
        args: Box<[GenericArgument<'src>]>,
    },
}

#[derive(Debug, PartialEq)]
pub enum GenericArgument<'src> {
    Type(SpannedTypeAnnotation<'src>),
    Expr(SpannedExpr<'src>),
}

pub type SpannedTypeAnnotation<'src> = Spanned<TypeAnnotation<'src>>;
