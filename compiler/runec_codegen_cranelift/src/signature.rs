#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AbiType {
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Pointer,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub params: Vec<AbiType>,
    pub returns: Vec<AbiType>,
}

impl FunctionSignature {
    pub fn new(params: impl Into<Vec<AbiType>>, returns: impl Into<Vec<AbiType>>) -> Self {
        Self {
            params: params.into(),
            returns: returns.into(),
        }
    }
}
