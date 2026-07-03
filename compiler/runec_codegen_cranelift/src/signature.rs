pub use runec_abi::AbiType;

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
