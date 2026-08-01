use runec_utils::define_messages;

define_messages! {
    MISSING_LOCAL_ID => "local binding is missing a resolved id",
    UNKNOWN_LOCAL => "cannot find type information for local `{local}`",
    UNRESOLVED_EXPRESSION => "expression was not resolved before type checking",
    UNRESOLVED_TYPE => "type was not resolved before type checking",
    NOT_CALLABLE => "value of type `{actual}` is not callable",
    ARGUMENT_COUNT_MISMATCH => "expected {expected} arguments, found {actual}",
    TYPE_MISMATCH => "expected type `{expected}`, found `{actual}`",
    CONTRACT_NOT_IMPLEMENTED => "type `{actual}` does not implement `{contract}`",
}
