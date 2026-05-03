use runec_utils::define_message;

define_message! {
    INVALID_CHAR => "invalid character: `{ char }`",
    INVALID_NUMERIC_LITERAL => "invalid number literal",
    DUPLICATED_STRING_LITERAL_PREFIX => "string literal cannot have duplicated prefix '{ prefix }'",
    UNTERMINATED_STRING => "unterminated string",
    UNTERMINATED_F_STRING_CODE_BLOCK => "unterminated code block in format string",
    UNTERMINATED_CHAR_LITERAL => "unterminated character literal",
    UNTERMINATED_COMMENT_BLOCK => "unterminated comment block",
    UNTERMINATED_ESCAPE_SEQUENCE => "unterminated escape sequence",
    INVALID_ESCAPE_SEQUENCE => "invalid escape sequence: { sequence }",
    OUT_OF_RANGE_HEX_ESCAPE => "hex escape out of range",
    OUT_OF_RANGE_HEX_ESCAPE_LABEL => "must be in range: [\\x00-\\x7f]",
    INVALID_UNICODE_ESCAPE => "invalid unicode character escape",
    UNICODE_ESCAPE_MUST_BE_IN_RANGE => "must be in range: [\\U000000-\\U10FFFF]",
    UNICODE_ESCAPE_MUST_NOT_BE_SURROGATE => "must not be a surrogate",
    UNICODE_ESCAPE_SEQUENCE_FORMAT => "format of unicode escape sequences is `\\u{{...}}`",
    UNICODE_MUST_BE_HEX => "must be hexadecimal",
    UNICODE_MUST_HAVE_AT_MOST_6_HEX_DIGITS => "must have at most 6 hex digits",
    NO_VALID_DIGITS => "no valid digits found",
    EMPTY_CHAR_LITERAL => "empty char literal",
}
