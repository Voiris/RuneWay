pub use runec_abi::AbiType;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub params: Vec<AbiType>,
    pub returns: Vec<AbiType>,
}

impl FunctionSignature {
    pub fn new(params: impl Into<Vec<AbiType>>, returns: impl Into<Vec<AbiType>>) -> Self {
        Self { params: params.into(), returns: returns.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::{AbiType, FunctionSignature};

    #[test]
    fn preserves_parameters_and_returns() {
        let signature = FunctionSignature::new([AbiType::I32, AbiType::Pointer], [AbiType::I64]);

        assert_eq!(signature.params, [AbiType::I32, AbiType::Pointer]);
        assert_eq!(signature.returns, [AbiType::I64]);
    }
}
