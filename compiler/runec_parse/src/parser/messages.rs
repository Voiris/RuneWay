use runec_utils::define_message;

define_message! {
    UNEXPECTED_TOKEN => "unexpected token: `{ token }`",
    EXPECTED_TOKEN_GOT => "expected token: `{ expected }`. Got: `{ got }`",
    UNTERMINATED_ARGS_BLOCK => "unterminated arguments block. Expect: `)`",
    UNTERMINATED_CODE_BLOCK => "unterminated code block. Expect: `\\u007D`",
    UNTERMINATED_TUPLE_TYPE_ANNOTATION => "unterminated tuple type annotation. Expect: `)`",
    UNTERMINATED_TUPLE => "unterminated tuple. Expect: `)`",
    UNTERMINATED_ARRAY => "unterminated array. Expect: `]`",
    UNTERMINATED_GENERIC => "unterminated generic. Expect: `>`",
    EXPECT_CODE_BLOCK => "expect code block",
    INTEGER_LITERAL_IS_TOO_LARGE => "integer literal is too large",
    INTEGER_LITERAL_VALUE_EXCEEDS_LIMIT => "value exceeds limit `340282366920938463463374607431768211455`",
    UNSUPPORTED_SUFFIX => "unsupported suffix",
    SUPPORTED_SUFFIXES_INT => "supported suffixes: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64",
    SUPPORTED_SUFFIXES_FLOAT => "supported float suffixes: f32, f64",
    UNABLE_TO_PARSE_FLOAT_NUMBER => "unable to parse float number"
}
