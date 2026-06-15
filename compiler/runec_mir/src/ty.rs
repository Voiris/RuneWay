use runec_builtins::TypeBits;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MirTy {
    Unit,
    Bool,
    Int(MirIntTy),
    Float(MirFloatTy),
    Char,
    Str,
    Bytes,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirIntTy {
    pub signed: bool,
    pub bits: TypeBits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirFloatTy {
    pub bits: TypeBits,
}
