use runec_abi::RuntimeFunctionId;
use runec_mir::{MirFunctionId, MirLocalId, MirTy};
use runec_source::span::Span;

pub type CodegenResult<T> = Result<T, CodegenError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub span: Option<Span>,
    pub kind: CodegenErrorKind,
}

impl CodegenError {
    pub fn new(kind: CodegenErrorKind) -> Self {
        Self { span: None, kind }
    }

    pub fn at(span: Span, kind: CodegenErrorKind) -> Self {
        Self {
            span: Some(span),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenErrorKind {
    MissingEntry,
    UnsupportedType(MirTy),
    UnsupportedFunction(MirFunctionId),
    UnsupportedRuntimeFunction(RuntimeFunctionId),
    UnknownLocal(MirLocalId),
}
