use runec_source::span::Spanned;

#[derive(Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    Unit,
    Ident(&'src str),
    Tuple(Box<[SpannedTypeAnnotation<'src>]>)
    /*Generic {
        ty: &'src str,
        params: Box<[GenericParameter<'src>]>,
    },
     */
}

/*
pub enum GenericParameter<'src> {
    Type(TypeAnnotation<'src>),
    // TODO: ConstValue(&'src str),
}
 */

pub type SpannedTypeAnnotation<'src> = Spanned<TypeAnnotation<'src>>;
