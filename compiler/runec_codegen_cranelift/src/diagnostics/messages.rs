use runec_utils::define_messages;

define_messages! {
    MISSING_ENTRY => "missing entry function during code generation",
    UNSUPPORTED_TYPE => "unsupported type { type } during code generation",
    UNSUPPORTED_FUNCTION => "unsupported function { function } during code generation",
    UNSUPPORTED_RUNTIME_FUNCTION => "unsupported runtime function { function } during code generation",
    UNKNOWN_LOCAL => "unknown local { local } during code generation",
    MISSING_ENTRY_BLOCK => "missing MIR entry block during code generation",
    UNKNOWN_FUNCTION => "unknown function { function } during code generation",
    ABI_ARITY_MISMATCH => "assignment ABI arity mismatch during code generation",
    BACKEND_FAILURE => "code generation backend failed: { error }",
}
