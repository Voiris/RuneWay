use std::borrow::Cow;

use crate::ty::MirTy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirConstant<'src> {
    Str(Cow<'src, str>),
    Bytes(Cow<'src, [u8]>),
}

impl MirConstant<'_> {
    pub fn ty(&self) -> MirTy {
        match self {
            MirConstant::Str(_) => MirTy::Str,
            MirConstant::Bytes(_) => MirTy::Bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::MirConstant;
    use crate::ty::MirTy;

    #[test]
    fn reports_string_and_byte_types() {
        let string = MirConstant::Str(Cow::Borrowed("value"));
        let bytes = MirConstant::Bytes(Cow::Borrowed(b"value"));

        assert_eq!(string.ty(), MirTy::Str);
        assert_eq!(bytes.ty(), MirTy::Bytes);
    }
}
