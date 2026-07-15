use runec_utils::define_messages;

define_messages! {
    MISSING_FUNCTION_SIGNATURE => "missing function signature during MIR lowering",
    MISSING_LOCAL_ID => "missing HIR local ID during MIR lowering",
    MISSING_LOCAL_INFO => "missing type information for local { local } during MIR lowering",
    UNKNOWN_BUILTIN => "unknown builtin { builtin } during MIR lowering",
    UNKNOWN_LOCAL => "unknown local { local } during MIR lowering",
    UNSUPPORTED_EXPRESSION => "unsupported { expression } in MIR lowering",
    UNSUPPORTED_TYPE => "unsupported type { ty } in MIR lowering",
}
