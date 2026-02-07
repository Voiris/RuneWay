pub enum TypeAnnotation<'src> {
    Ident(&'src str),
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
