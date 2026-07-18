use runec_errors::diagnostics::Diagnostic;
use runec_errors::labels::DiagLabel;
use runec_errors::message::DiagMessage;
use runec_source::span::Span;

pub type CodegenResult<T> = Result<T, Box<Diagnostic<'static>>>;

pub(crate) mod messages;

pub(crate) fn error(
    message: &'static str,
    replacements: &[(&str, &str)],
    span: Option<Span>,
) -> Box<Diagnostic<'static>> {
    let diagnostic = Diagnostic::error(DiagMessage::new(message, replacements));
    match span {
        Some(span) => diagnostic.add_label(DiagLabel::silent_primary(span)),
        None => diagnostic,
    }
}

pub(crate) fn backend(error_value: impl std::fmt::Display) -> Box<Diagnostic<'static>> {
    let error_value = error_value.to_string();
    error(messages::BACKEND_FAILURE, &[("error", &error_value)], None)
}
