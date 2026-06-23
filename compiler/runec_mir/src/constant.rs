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
