use runec_abi::RuntimeFunctionId;
use runec_mir::MirFunctionId;

pub type CodegenResult<T> = Result<T, CodegenError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub kind: CodegenErrorKind,
}

impl CodegenError {
    pub fn new(kind: CodegenErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenErrorKind {
    MissingEntry,
    UnsupportedFunction(MirFunctionId),
    UnsupportedRuntimeFunction(RuntimeFunctionId),
}
