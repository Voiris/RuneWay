#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunctionId(u32);

impl RuntimeFunctionId {
    pub const fn from_index(index: usize) -> Self {
        assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

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
    Usize,
    Unit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RuntimeFunctionDecl {
    pub symbol: &'static str,
    pub params: &'static [AbiType],
    pub ret: AbiType,
}

pub const RUNTIME_PRINT: RuntimeFunctionId = RuntimeFunctionId::from_index(0);
pub const RUNTIME_PRINTLN: RuntimeFunctionId = RuntimeFunctionId::from_index(1);

const STRING_PARAMS: &[AbiType] = &[AbiType::Pointer, AbiType::Usize];

pub const RUNTIME_FUNCTIONS: &[RuntimeFunctionDecl] = &[
    RuntimeFunctionDecl {
        symbol: "__runeway_print",
        params: STRING_PARAMS,
        ret: AbiType::Unit,
    },
    RuntimeFunctionDecl {
        symbol: "__runeway_println",
        params: STRING_PARAMS,
        ret: AbiType::Unit,
    },
];

pub fn runtime_function(id: RuntimeFunctionId) -> Option<&'static RuntimeFunctionDecl> {
    RUNTIME_FUNCTIONS.get(id.index())
}

#[cfg(test)]
mod tests {
    use super::{AbiType, RUNTIME_PRINTLN, runtime_function};

    #[test]
    fn exposes_stable_runtime_declarations() {
        let declaration = runtime_function(RUNTIME_PRINTLN).expect("println declaration");
        assert_eq!(declaration.symbol, "__runeway_println");
        assert_eq!(declaration.params, &[AbiType::Pointer, AbiType::Usize]);
        assert_eq!(declaration.ret, AbiType::Unit);
    }
}
