use crate::ty::MirTy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirConstant {
    Str(Box<str>),
    Bytes(Box<[u8]>),
}

impl MirConstant {
    pub fn ty(&self) -> MirTy {
        match self {
            MirConstant::Str(_) => MirTy::Str,
            MirConstant::Bytes(_) => MirTy::Bytes,
        }
    }
}
