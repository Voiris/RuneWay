# General
unexpected-eof = unexpected end of file `{ $path }`

# Lexer
invalid-char = invalid character: { $char }
invalid-numeric-literal = invalid number literal
duplicated-string-literal-prefix = string literal cannot have duplicated prefix '{ $prefix }'
unterminated-string = unterminated string
unterminated-f-string-code-block = unterminated code block in format string
unterminated-char-literal = unterminated character literal
unterminated-comment-block = unterminated comment block
unterminated-escape-sequence = unterminated escape sequence
invalid-escape-sequence = invalid escape sequence: { $sequence }
out-of-range-hex-escape = hex escape out of range
out-of-range-hex-escape-label = must be in range: [\\x00-\\x7f]
invalid-unicode-escape = invalid unicode character escape
unicode-escape-must-be-in-range = must be in range: [\\U000000-\\U10FFFF]
unicode-escape-must-not-be-surrogate = must not be a surrogate
unicode-escape-sequence-format = format of unicode escape sequences is {"`\\u{...}`"}
unicode-must-be-hex = must be hexadecimal
unicode-must-have-at-most-6-hex-digits = must have at most 6 hex digits
no-valid-digits = no valid digits found
empty-char-literal = empty char literal

# Parser
unexpected-token = unexpected token: `{ $token }`
expected-token-got = expected token: `{ $expected }`. Got: `{ $got }`
unterminated-args-block = unterminated arguments block. Expect: `)`
unterminated-code-block = unterminated code block. Expect: \u007D
expect-code-block = expect code block
integer-literal-is-too-large = integer literal is too large
integer-literal-value-exceeds-limit = value exceeds limit `340282366920938463463374607431768211455`
unsupported-suffix = unsupported suffix
supported-suffixes-int = supported suffixes: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
supported-suffixes-float = supported float suffixes: f32, f64
unable-to-parse-float-number = unable to parse float number
