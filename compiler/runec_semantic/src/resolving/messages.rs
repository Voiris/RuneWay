use runec_utils::define_messages;

define_messages! {
    DUPLICATE_ITEM => "item `{name}` is defined multiple times",
    DUPLICATE_LOCAL => "local `{name}` is defined multiple times",
    UNRESOLVED_NAME => "cannot resolve value `{name}`",
    UNRESOLVED_TYPE => "cannot resolve type `{name}`",
}
